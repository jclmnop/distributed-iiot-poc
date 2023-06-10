use actor_interfaces::{LogEvent, SearchParams};
use once_cell::sync::Lazy;
use wasmbus_rpc::actor::prelude::Context;
use wasmbus_rpc::error::{RpcError, RpcResult};
use wasmcloud_interface_httpclient::{HeaderMap, HeaderValues, HttpRequest};
use wasmcloud_interface_keyvalue::{KeyValue, KeyValueSender};

const API_KEY: &str = "PANGEA_API_KEY";
const DOMAIN: &str = "aws.eu.pangea.cloud";
static AUDIT_URI: Lazy<String> = Lazy::new(|| {
    let s = format!("https://audit.{}", DOMAIN);
    s
});
static AUDIT_LOG_ENDPOINT: Lazy<String> = Lazy::new(|| {
    let s = format!("{}/v1/log", AUDIT_URI.as_str());
    s
});
static AUDIT_SEARCH_ENDPOINT: Lazy<String> = Lazy::new(|| {
    let s = format!("{}/v1/search", AUDIT_URI.as_str());
    s
});

pub fn build_log_request(
    api_token: &String,
    mut event: LogEvent,
) -> RpcResult<HttpRequest> {
    event.convert_timestamps();
    let headers = headers(api_token);
    let body = serde_json::json!({ "event": event });
    let body = serde_json::to_vec(&body).map_err(|e| RpcError::Ser(e.to_string()))?;
    Ok(HttpRequest {
        method: "POST".to_string(),
        url: AUDIT_LOG_ENDPOINT.to_string(),
        headers,
        body,
    })
}

//TODO: use a proper search query type
pub fn build_search_request(
    api_token: &String,
    mut query: SearchParams,
) -> RpcResult<HttpRequest> {
    query.convert_timestamps();
    let headers = headers(api_token);
    let body = serde_json::to_vec(&query).map_err(|e| RpcError::Ser(e.to_string()))?;
    Ok(HttpRequest {
        method: "POST".to_string(),
        url: AUDIT_SEARCH_ENDPOINT.to_string(),
        headers,
        body,
    })
}

fn headers(api_token: &String) -> HeaderMap {
    let headers = HeaderMap::from([
        (
            "Content-Type".to_string(),
            vec!["application/json".to_string()],
        ),
        (
            "Authorization".to_string(),
            vec![format!("Bearer {api_token}")],
        ),
    ]);
    headers
}

pub async fn get_api_token(ctx: &Context) -> RpcResult<String> {
    let kv = KeyValueSender::new();
    let api_token = kv.get(ctx, API_KEY).await?.value;
    Ok(api_token)
}

fn timestamp_to_datetime(timestamp: Option<String>) -> Option<String> {
    let timestamp = timestamp?;
    let timestamp = timestamp.parse::<i64>().ok()?;
    let dt = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0);
    if let Some(dt) = dt {
        let dt = dt.format("%Y-%m-%dT%H:%M:%S.%fZ").to_string();
        Some(dt)
    } else {
        None
    }
}

trait ConvertTimestamps {
    fn convert_timestamps(&mut self);
}

impl ConvertTimestamps for SearchParams {
    fn convert_timestamps(&mut self) {
        self.start = timestamp_to_datetime(self.start.clone());
        self.end = timestamp_to_datetime(self.end.clone());
        if let Some(restrictions) = &mut self.search_restriction {
            if let Some(timestamp) = &mut restrictions.timestamp {
                timestamp.iter_mut().for_each(|t| {
                    *t = timestamp_to_datetime(Some(t.to_string())).unwrap_or("".to_string())
                });
                restrictions.timestamp = Some(
                    timestamp
                        .into_iter()
                        .filter(|t| t != &&"".to_string())
                        .map(|t| t.clone())
                        .collect(),
                );
            }
            if let Some(received_at) = &mut restrictions.received_at {
                received_at.iter_mut().for_each(|t| {
                    *t = timestamp_to_datetime(Some(t.to_string())).unwrap_or("".to_string())
                });
                restrictions.received_at = Some(
                    received_at
                        .into_iter()
                        .filter(|t| t != &&"".to_string())
                        .map(|t| t.clone())
                        .collect(),
                );
            }
        }
    }
}

impl ConvertTimestamps for LogEvent {
    fn convert_timestamps(&mut self) {
        self.timestamp = timestamp_to_datetime(self.timestamp.clone());
    }
}
