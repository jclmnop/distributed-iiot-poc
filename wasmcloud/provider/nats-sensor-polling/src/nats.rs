// A lot of this is just copied from the official wasmcloud nats provider
// https://github.com/wasmCloud/capability-providers/blob/main/nats/src/main.rs#L136

use async_nats::Message;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::{OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, error, instrument, warn, Instrument};
use wascap::prelude::KeyPair;
use wasmbus_rpc::core::LinkDefinition;
use wasmbus_rpc::error::{RpcError, RpcResult};

pub type NatsClient = async_nats::Client;
pub type HeartbeatRx = UnboundedReceiver<(LinkDefinition, Message, OwnedSemaphorePermit)>;
pub type HeartbeatTx = UnboundedSender<(LinkDefinition, Message, OwnedSemaphorePermit)>;

const DEFAULT_NATS_URI: &str = "0.0.0.0:4222";
const ENV_NATS_SUBSCRIPTION: &str = "SUBSCRIPTION";
const ENV_NATS_URI: &str = "URI";
const ENV_NATS_CLIENT_JWT: &str = "CLIENT_JWT";
const ENV_NATS_CLIENT_SEED: &str = "CLIENT_SEED";

/// Configuration for connecting a nats client.
/// More options are available if you use the json than variables in the values string map.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    /// list of topics to subscribe to for sensor heartbeats
    #[serde(default)]
    subscriptions: Vec<String>,
    #[serde(default)]
    cluster_uris: Vec<String>,
    #[serde(default)]
    auth_jwt: Option<String>,
    #[serde(default)]
    auth_seed: Option<String>,

    /// ping interval in seconds
    #[serde(default)]
    ping_interval_sec: Option<u16>,
}

impl ConnectionConfig {
    pub fn merge(&self, extra: &ConnectionConfig) -> ConnectionConfig {
        let mut out = self.clone();
        if !extra.subscriptions.is_empty() {
            out.subscriptions = extra.subscriptions.clone();
        }
        // If the default configuration has a URL in it, and then the link definition
        // also provides a URL, the assumption is to replace/override rather than combine
        // the two into a potentially incompatible set of URIs
        if !extra.cluster_uris.is_empty() {
            out.cluster_uris = extra.cluster_uris.clone();
        }
        if extra.auth_jwt.is_some() {
            out.auth_jwt = extra.auth_jwt.clone()
        }
        if extra.auth_seed.is_some() {
            out.auth_seed = extra.auth_seed.clone()
        }
        if extra.ping_interval_sec.is_some() {
            out.ping_interval_sec = extra.ping_interval_sec.clone()
        }
        out
    }

    pub fn new_from(values: &HashMap<String, String>) -> RpcResult<ConnectionConfig> {
        let mut config = if let Some(config_b64) = values.get("config_b64") {
            let bytes = base64::decode(config_b64.as_bytes()).map_err(|e| {
                RpcError::InvalidParameter(format!("invalid base64 encoding: {}", e))
            })?;
            serde_json::from_slice::<ConnectionConfig>(&bytes)
                .map_err(|e| RpcError::InvalidParameter(format!("corrupt config_b64: {}", e)))?
        } else if let Some(config) = values.get("config_json") {
            serde_json::from_str::<ConnectionConfig>(config)
                .map_err(|e| RpcError::InvalidParameter(format!("corrupt config_json: {}", e)))?
        } else {
            ConnectionConfig::default()
        };

        if let Some(sub) = values.get(ENV_NATS_SUBSCRIPTION) {
            config
                .subscriptions
                .extend(sub.split(',').map(|s| s.to_string()));
        }
        if let Some(url) = values.get(ENV_NATS_URI) {
            config.cluster_uris = url.split(',').map(String::from).collect();
        }
        if let Some(jwt) = values.get(ENV_NATS_CLIENT_JWT) {
            config.auth_jwt = Some(jwt.clone());
        }
        if let Some(seed) = values.get(ENV_NATS_CLIENT_SEED) {
            config.auth_seed = Some(seed.clone());
        }
        if config.auth_jwt.is_some() && config.auth_seed.is_none() {
            return Err(RpcError::InvalidParameter(
                "if you specify jwt, you must also specify a seed".to_string(),
            ));
        }
        if config.cluster_uris.is_empty() {
            config.cluster_uris.push(DEFAULT_NATS_URI.to_string());
        }
        Ok(config)
    }
}

impl Default for ConnectionConfig {
    fn default() -> ConnectionConfig {
        ConnectionConfig {
            subscriptions: vec![],
            cluster_uris: vec![DEFAULT_NATS_URI.to_string()],
            auth_jwt: None,
            auth_seed: None,
            ping_interval_sec: None,
        }
    }
}

/// NatsClientBundles hold a NATS client and information (subscriptions)
/// related to it.
///
/// This struct is necessary because subscriptions are *not* automatically removed on client drop,
/// meaning that we must keep track of all subscriptions to close once the client is done
#[derive(Debug)]
pub struct NatsClientBundle {
    pub client: NatsClient,
    pub heartbeat_sub_handles: Vec<(String, JoinHandle<()>)>,
}

impl Drop for NatsClientBundle {
    fn drop(&mut self) {
        for handle in &self.heartbeat_sub_handles {
            handle.1.abort()
        }
    }
}

impl NatsClientBundle {
    /// Attempt to connect to nats url (with jwt credentials, if provided)
    pub async fn connect(
        cfg: ConnectionConfig,
        ld: &LinkDefinition,
        heartbeat_tx: HeartbeatTx,
    ) -> Result<NatsClientBundle, RpcError> {
        let opts = match (cfg.auth_jwt, cfg.auth_seed) {
            (Some(jwt), Some(seed)) => {
                let key_pair = Arc::new(
                    KeyPair::from_seed(&seed)
                        .map_err(|e| RpcError::ProviderInit(format!("key init: {}", e)))?,
                );
                async_nats::ConnectOptions::with_jwt(jwt, move |nonce| {
                    let key_pair = key_pair.clone();
                    async move { key_pair.sign(&nonce).map_err(async_nats::AuthError::new) }
                })
            }
            (None, None) => async_nats::ConnectOptions::default(),
            _ => {
                return Err(RpcError::InvalidParameter(
                    "must provide both jwt and seed for jwt authentication".into(),
                ));
            }
        };

        // Use the first visible cluster_uri
        let url = cfg.cluster_uris.get(0).unwrap();

        let client = opts
            .name("NATS Sensor Polling Provider") // allow this to show up uniquely in a NATS connection list
            .connect(url)
            .await
            .map_err(|e| {
                RpcError::ProviderInit(format!("NATS Sensor Polling connection to {}: {}", url, e))
            })?;

        let mut nats_client_bundle = NatsClientBundle {
            client,
            heartbeat_sub_handles: Vec::new(),
        };
        // Heartbeat subscriptions
        for sub in cfg.subscriptions.iter().filter(|s| !s.is_empty()) {
            let (sub, queue) = match sub.split_once('|') {
                Some((sub, queue)) => (sub, Some(queue.to_string())),
                None => (sub.as_str(), None),
            };

            nats_client_bundle.heartbeat_sub_handles.push((
                sub.to_string(),
                nats_client_bundle
                    .subscribe_to_heartbeat(ld, sub.to_string(), queue, heartbeat_tx.clone())
                    .await?,
            ));
        }

        Ok(nats_client_bundle)
    }

    /// Add a regular or queue subscription
    pub async fn subscribe_to_heartbeat(
        &self,
        ld: &LinkDefinition,
        sub: String,
        queue: Option<String>,
        heartbeat_tx: HeartbeatTx,
    ) -> RpcResult<JoinHandle<()>> {
        let mut subscriber = match queue {
            Some(queue) => self.client.queue_subscribe(sub.clone(), queue).await,
            None => self.client.subscribe(sub.clone()).await,
        }
        .map_err(|e| {
            error!(subject = %sub, error = %e, "error subscribing");
            RpcError::Nats(format!("subscription to {}: {}", sub, e))
        })?;

        let link_def = ld.to_owned();

        // Spawn a thread that listens for messages coming from NATS
        // this thread is expected to run the full duration that the provider is available
        let join_handle = tokio::spawn(async move {
            // MAGIC NUMBER: Based on our benchmark testing, this seems to be a good upper limit
            // where we start to get diminishing returns. We can consider making this
            // configurable down the line.
            // NOTE (thomastaylor312): It may be better to have a semaphore pool on the
            // NatsMessagingProvider struct that has a global limit of permits so that we don't end
            // up with 20 subscriptions all getting slammed with up to 75 tasks, but we should wait
            // to do anything until we see what happens with real world usage and benchmarking
            let semaphore = Arc::new(Semaphore::new(75));

            // Listen for heartbeat
            while let Some(msg) = subscriber.next().await {
                let permit = match semaphore.clone().acquire_owned().await {
                    Ok(p) => p,
                    Err(_) => {
                        warn!("Work pool has been closed, exiting queue subscribe");
                        break;
                    }
                };

                Self::send_heartbeat(&heartbeat_tx, link_def.clone(), msg, permit);
            }
        });

        Ok(join_handle)
    }

    #[instrument(level = "debug", skip_all, fields(actor_id = %ld.actor_id, subject = %msg.subject))]
    fn send_heartbeat(
        channel: &HeartbeatTx,
        ld: LinkDefinition,
        msg: Message,
        _permit: OwnedSemaphorePermit,
    ) {
        if let Err(e) = channel.send((ld, msg, _permit)) {
            error!(
                errpr = %e,
                "Unable to receive heartbeat"
            )
        }
    }
}
