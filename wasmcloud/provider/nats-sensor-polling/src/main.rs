//! Implementation for wasmcloud:messaging
//!
mod config;
mod nats;
mod sensor;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, UNIX_EPOCH};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::sync::{OwnedSemaphorePermit, RwLock};
use tracing::{debug, error, instrument};
use uuid::Uuid;
use wasmbus_rpc::core::HostData;
use wasmbus_rpc::{core::LinkDefinition, provider::prelude::*};
use wasmcloud_interface_polling::{
    AddPollTargetRequest, AddPollTargetResponse, PollRequest, PollResult, PollSubscriberSender,
    Polling, PollingReceiver, RemovePollTargetRequest, RemovePollTargetResponse,
};

use crate::nats::{ConnectionConfig, HeartbeatRx, HeartbeatTx, NatsClient, NatsClientBundle};
use crate::sensor::{PollInterval, Sensor};

type Sensors = Arc<RwLock<HashMap<Uuid, Sensor>>>;
type Schedule = Arc<RwLock<HashMap<PollInterval, HashSet<Uuid>>>>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // handle lattice control messages and forward rpc to the provider dispatch
    // returns when provider receives a shutdown control message
    let host_data = load_host_data()?;
    let provider = NatsSensorPollingProvider::new(host_data)?;

    provider_main(provider, Some("NATS-Sensor-Polling Provider".to_string()))?;

    eprintln!("NATS-Sensor-Polling provider exiting");
    Ok(())
}

// The heartbeat_sender needs to be stored here in case extra heartbeat subscriptions
// are added later.
struct ActorState {
    client: NatsClientBundle,
    heartbeat_sender: HeartbeatTx,
    sensors: Sensors,   //TODO: does this need to be stored here?
    schedule: Schedule, //TODO: does this need to be stored here?
    handles: Vec<tokio::task::JoinHandle<()>>,
    ld: LinkDefinition, //TODO: does this need to be stored here?
}

/// Abort all background tasks when dropped
impl Drop for ActorState {
    fn drop(&mut self) {
        for handle in &self.handles {
            handle.abort();
        }

        // TODO: will using blocking_write() here cause issues if other actors are still running?
        let mut write_sensors = self.sensors.blocking_write();
        write_sensors.clear();
        drop(write_sensors);

        let mut write_schedule = self.schedule.blocking_write();
        write_schedule.clear();
    }
}

/// Implementation for wasmcloud:polling
#[derive(Default, Clone, Provider)]
#[services(Polling)]
struct NatsSensorPollingProvider {
    actors: Arc<RwLock<HashMap<String, ActorState>>>,
    default_config: ConnectionConfig,
}

// use default implementations of provider message handlers
impl ProviderDispatch for NatsSensorPollingProvider {}

impl NatsSensorPollingProvider {
    fn new(host_data: HostData) -> Result<Self, anyhow::Error> {
        if let Some(config) = &host_data.config_json {
            if config.trim().is_empty() {
                Ok(Self::default())
            } else {
                let config: ConnectionConfig = serde_json::from_str(config)?;
                Ok(Self {
                    default_config: config,
                    ..Default::default()
                })
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Begin running tasks in the background, listening for heartbeats, updating discovered sensors,
    /// and regularly polling sensors so results can be sent to the actor given in the ld.
    ///
    /// Returns the handles for these background tasks.
    #[instrument(level = "info", skip_all, fields(actor_id = %ld.actor_id))]
    async fn run(
        ld: LinkDefinition,
        sensors: Sensors,
        schedule: Schedule,
        heartbeats: HeartbeatRx,
        client: NatsClient,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        let mut handles = vec![];

        //TODO: listen for heartbeats
        handles.push(tokio::task::spawn(Self::listen_heartbeats(
            ld.clone(),
            sensors.clone(),
            schedule.clone(),
            heartbeats,
            client,
        )));

        // //TODO: scheduled polling
        // handles.push(tokio::task::spawn(Self::scheduled_polling(
        //     ld,
        //     sensors.clone(),
        //     schedule.clone(),
        //     client,
        // )));

        handles
    }

    #[instrument(level = "info", skip_all, fields(actor_id = %ld.actor_id))]
    async fn listen_heartbeats(
        ld: LinkDefinition,
        sensors: Sensors,
        schedule: Schedule,
        mut heartbeats: HeartbeatRx,
        client: NatsClient,
    ) {
        loop {
            let (_, msg, _permit) = if let Some(heartbeat) = heartbeats.recv().await {
                heartbeat
            } else {
                break;
            };
            let sensor_info = match serde_json::from_slice::<Sensor>(&msg.payload) {
                Ok(sensor_info) => sensor_info,
                Err(e) => {
                    error!("Failed to deserialize sensor info: {e:?}");
                    continue;
                }
            };

            let read_sensors = sensors.read().await;
            if !read_sensors.contains_key(&sensor_info.id) {
                drop(read_sensors);

                let id = sensor_info.id.clone();
                let poll_interval = sensor_info.poll_interval;

                let mut write_sensors = sensors.write().await;
                write_sensors.insert(id, sensor_info.clone());
                drop(write_sensors);

                let mut write_schedule = schedule.write().await;
                if let Some(sensor_ids) = write_schedule.get_mut(&poll_interval) {
                    if sensor_ids.is_empty() {
                        sensor_ids.insert(id);
                        tokio::task::spawn(Self::scheduled_polling(
                            ld.clone(),
                            sensors.clone(),
                            schedule.clone(),
                            client.clone(),
                            poll_interval,
                        ));
                    } else {
                        sensor_ids.insert(id);
                    }
                } else {
                    let mut sensor_ids = HashSet::new();
                    sensor_ids.insert(id);
                    write_schedule.insert(poll_interval, sensor_ids);
                    tokio::task::spawn(Self::scheduled_polling(
                        ld.clone(),
                        sensors.clone(),
                        schedule.clone(),
                        client.clone(),
                        poll_interval,
                    ));
                }
                drop(write_schedule);

                // TODO: listen for disconnect message (not needed for my PoC)
            }
        }
    }

    #[instrument(level = "info", skip_all, fields(actor_id = %ld.actor_id))]
    async fn scheduled_polling(
        ld: LinkDefinition,
        sensors: Sensors,
        schedule: Schedule,
        client: NatsClient,
        poll_interval: PollInterval,
    ) {
        let mut poll_clock = tokio::time::interval(Duration::from_millis(poll_interval));
        loop {
            poll_clock.tick().await;

            // If there is no schedule for this poll interval anymore, end the task
            let sensor_ids = {
                let read_schedule = schedule.read().await;
                let sensor_ids = read_schedule.get(&poll_interval);
                if let Some(sensor_ids) = sensor_ids {
                    if sensor_ids.is_empty() {
                        break;
                    } else {
                        sensor_ids.clone()
                    }
                } else {
                    break;
                }
            };

            let sensors = {
                let read_sensors = sensors.read().await;
                // TODO: optimise this
                let mut sensors = Vec::with_capacity(sensor_ids.len());
                for sensor_id in sensor_ids.iter() {
                    sensors.push(read_sensors.get(sensor_id))
                }
                sensors
                    .into_iter()
                    .flatten()
                    .map(|s| s.clone())
                    .collect::<Vec<Sensor>>()
            };

            // TODO: create stream that does the following:
            //  - gets poll and read topics
            //  - subscribes to read topic
            //  - publishes to poll topic
            //  - waits with ~500ms(?) timeout for "reply"
            //  - COMM_ERROR if no response, json! with sensor id, name, location, timestamp otherwise


            // TODO: handle the collapse of the spacetime continuum
            let timestamp = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("If time has gone backwards then we have bigger problems than this error.")
                .as_secs();
        }
    }

    async fn poll(&self, actor_name: String) {
        todo!()
    }

    async fn connect(
        &self,
        cfg: ConnectionConfig,
        ld: &LinkDefinition,
        heartbeat_tx: HeartbeatTx,
    ) -> Result<ActorState, RpcError> {
        let nats_client_bundle = NatsClientBundle::connect(cfg, ld, heartbeat_tx.clone()).await?;

        Ok(ActorState {
            client: nats_client_bundle,
            heartbeat_sender: heartbeat_tx,
            ld: ld.clone(),
            sensors: Default::default(),
            schedule: Default::default(),
            handles: Default::default(),
        })
    }
}

/// Handle provider control commands
/// put_link (new actor link command), del_link (remove link command), and shutdown
#[async_trait]
impl ProviderHandler for NatsSensorPollingProvider {
    /// Provider should perform any operations needed for a new link,
    /// including setting up per-actor resources, and checking authorization.
    /// If the link is allowed, return true, otherwise return false to deny the link.
    #[instrument(level = "info", skip(self, ld), fields(actor_id = %ld.actor_id))]
    async fn put_link(&self, ld: &LinkDefinition) -> RpcResult<bool> {
        let (heartbeat_tx, heartbeat_rx) = unbounded_channel();
        debug!("putting link for actor {:?}", ld);
        let config = if ld.values.is_empty() {
            self.default_config.clone()
        } else {
            match ConnectionConfig::new_from(&ld.values) {
                Ok(config) => self.default_config.merge(&config),
                Err(e) => {
                    error!("Failed to build connection configuration: {e:?}");
                    return Ok(false);
                }
            }
        };

        let mut actor = self.connect(config, ld, heartbeat_tx).await?;
        let schedule = actor.schedule.clone();
        let sensors = actor.sensors.clone();
        let client = actor.client.client.clone();
        // Run the background tasks
        actor.handles = Self::run(ld.clone(), sensors, schedule, heartbeat_rx, client).await;

        {
            let mut write_actors = self.actors.write().await;
            write_actors.insert(ld.actor_id.to_string(), actor);
        }

        Ok(true)
    }

    /// Handle notification that a link is dropped: close the connection
    #[instrument(level = "info", skip(self))]
    async fn delete_link(&self, actor_id: &str) {
        debug!("deleting link for actor {}", actor_id);

        todo!()
    }

    /// Handle shutdown request with any cleanup necessary
    async fn shutdown(&self) -> Result<(), Infallible> {
        todo!();
    }
}

#[async_trait]
impl Polling for NatsSensorPollingProvider {
    async fn poll_tx(&self, ctx: &Context, arg: &PollRequest) -> RpcResult<PollResult> {
        todo!()
    }

    async fn add_poll_target(
        &self,
        ctx: &Context,
        arg: &AddPollTargetRequest,
    ) -> RpcResult<AddPollTargetResponse> {
        todo!()
    }

    async fn remove_poll_target(
        &self,
        ctx: &Context,
        arg: &RemovePollTargetRequest,
    ) -> RpcResult<RemovePollTargetResponse> {
        todo!()
    }
}

// /// Handle Messaging methods
// #[async_trait]
// impl Messaging for NatsSensorPollingProvider {
//     #[instrument(level = "debug", skip(self, msg), fields(subject = %msg.subject, reply_to = ?msg.reply_to, body_len = %msg.body.len()))]
//     async fn publish(&self, _ctx: &Context, msg: &PubMessage) -> RpcResult<()> {
//         debug!("Publishing message: {:?}", msg);
//         Err(RpcError::NotImplemented)
//     }
//
//     #[instrument(level = "debug", skip(self, msg), fields(subject = %msg.subject))]
//     async fn request(&self, _ctx: &Context, msg: &RequestMessage) -> RpcResult<ReplyMessage> {
//         debug!("Sending message request: {:?}", msg);
//         Err(RpcError::NotImplemented)
//     }
// }
