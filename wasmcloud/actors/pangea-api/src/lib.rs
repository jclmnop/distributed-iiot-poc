mod api;

use actor_interfaces::{LogEvent, PangeaApi, PangeaApiReceiver, SearchResponse, WriteResult};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpclient::{HttpClient, HttpClientSender};
use wasmcloud_interface_logging::{error, info, debug};
use api::build_log_request;
use crate::api::build_search_request;

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, PangeaApi)]
struct PangeaApiActor {}

// TODO: pangea API module
#[async_trait]
impl PangeaApi for PangeaApiActor {
    async fn write_audit_log(
        &self,
        ctx: &Context,
        events: &Vec<LogEvent>,
    ) -> RpcResult<WriteResult> {
        info!("Received {} events to write to audit log", events.len());
        let mut success = true;
        let mut reason = None;
        let client = HttpClientSender::new();
        let api_token = api::get_api_token(ctx).await?;

        for event in events {
            debug!("Event: {:?}", event);
            let req = build_log_request(ctx, &api_token, event.clone());
            match req {
                Ok(req) => {
                    match client.request(ctx, &req).await {
                        Ok(resp) => {
                            if resp.status_code == 200 {
                                info!("Successfully wrote event to audit log");
                            } else {
                                error!("Error writing event to audit log: {}", resp.status_code);
                                success = false;
                                reason = Some(format!("Error writing event to audit log: {}", resp.status_code));
                            }
                        }
                        Err(e) => {
                            error!("Error writing event to audit log: {}", e);
                            success = false;
                            reason = Some(format!("Error writing event to audit log: {}", e));
                        }
                    }
                }
                Err(e) => {
                    error!("Error building log request: {}", e);
                    success = false;
                    reason = Some(format!("Error building log request: {}", e));
                }
            }
        }

        Ok(WriteResult {
            success,
            reason,
        })
    }

    async fn search_audit_log<TS: ToString + ?Sized + Sync>(
        &self,
        ctx: &Context,
        query: &TS,
    ) -> RpcResult<SearchResponse> {
        info!("Received search query: {}", query.to_string());
        let client = HttpClientSender::new();
        let api_token = api::get_api_token(ctx).await?;
        let req = build_search_request(ctx, &api_token, query.to_string())?;
        let resp = client.request(ctx, &req).await?;
        if resp.status_code == 200 {
            info!("Successfully queried audit log");
            match serde_json::from_slice::<SearchResponse>(&resp.body) {
                Ok(search_response) => Ok(search_response),
                Err(e) => {
                    error!("Error deserializing search response: {}", e);
                    Err(RpcError::Other(format!("Error deserializing search response: {}", e)))
                }
            }
        } else {
            error!("Error querying audit log: {}", resp.status_code);
            Err(RpcError::Other(format!("Error querying audit log: {}", resp.status_code)))
        }
    }
}
