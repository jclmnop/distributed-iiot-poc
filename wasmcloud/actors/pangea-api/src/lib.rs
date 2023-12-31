mod api;

use crate::api::build_search_request;
use actor_interfaces::{
    LogEvent, PangeaApi, PangeaApiReceiver, SearchParams, SearchResponse, WriteResult,
};
use api::build_log_request;
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpclient::{HttpClient, HttpClientSender};
use wasmcloud_interface_logging::{debug, error, info};

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
        let api_token = match api::get_api_token(ctx).await {
            Ok(api_token) => api_token,
            Err(e) => {
                error!("Error getting API token: {}", e);
                success = false;
                reason = Some(format!("Error getting API token: {}", e));
                return Ok(WriteResult { success, reason });
            }
        };

        for event in events {
            debug!("Event: {:?}", event);
            let req = build_log_request(&api_token, event.to_owned());
            match req {
                Ok(req) => match client.request(ctx, &req).await {
                    Ok(resp) => {
                        if resp.status_code == 200 {
                            info!("Successfully wrote event to audit log");
                        } else {
                            error!(
                                "Error writing event to audit log: url: {} status: {}",
                                req.url, resp.status_code
                            );
                            success = false;
                            reason = Some(format!(
                                "Error writing event to audit log: {}",
                                resp.status_code
                            ));
                            let req_body = std::str::from_utf8(&req.body).unwrap_or("n/a");
                            let resp_body = std::str::from_utf8(&resp.body).unwrap_or("n/a");
                            info!("Request body: {}", req_body);
                            info!("Response body: {}", resp_body);
                        }
                    }
                    Err(e) => {
                        error!("Error writing event to audit log: {}", e);
                        success = false;
                        reason = Some(format!("Error writing event to audit log: {}", e));
                    }
                },
                Err(e) => {
                    error!("Error building log request: {}", e);
                    success = false;
                    reason = Some(format!("Error building log request: {}", e));
                }
            }
        }

        Ok(WriteResult { success, reason })
    }

    async fn search_audit_log(
        &self,
        ctx: &Context,
        query: &SearchParams,
    ) -> RpcResult<SearchResponse> {
        info!(
            "Received search query: {}",
            query.query.clone().unwrap_or("n/a".to_string())
        );
        debug!("Query: {:?}", query);
        let client = HttpClientSender::new();
        let api_token = api::get_api_token(ctx).await?;
        let req = build_search_request(&api_token, query.to_owned())?;
        info!("Sending search request to {}", req.url);
        let resp = client.request(ctx, &req).await?;
        if resp.status_code == 200 {
            info!("Successfully queried audit log");
            match serde_json::from_slice::<SearchResponse>(&resp.body) {
                Ok(search_response) => Ok(search_response),
                Err(e) => {
                    error!(
                        "Error deserializing search response: \
                    \n\tError:{}\
                    \n\tResponse body: {}",
                        e,
                        std::str::from_utf8(&resp.body).unwrap_or("Invalid UTF-8")
                    );
                    Err(RpcError::Other(format!(
                        "Error deserializing search response: {}",
                        e
                    )))
                }
            }
        } else {
            error!("Error querying audit log: {}", resp.status_code);
            Err(RpcError::Other(format!(
                "Error querying audit log: {}",
                resp.status_code
            )))
        }
    }
}
