use once_cell::sync::Lazy;
use wasmbus_rpc::actor::prelude::Context;
use wasmbus_rpc::error::{RpcError, RpcResult};
use wasmcloud_interface_logging::{error, info};
use wasmcloud_interface_keyvalue::{KeyValue, KeyValueSender};
use wasmcloud_interface_httpclient::{HeaderMap, HeaderValues, HttpRequest};
use actor_interfaces::LogEvent;

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

pub fn build_log_request(context: &Context, api_token: &String, event: LogEvent) -> RpcResult<HttpRequest> {
    let headers = headers(api_token);
    let body = serde_json::to_vec(&event).map_err(|e| RpcError::Ser(e.to_string()))?;
    Ok(HttpRequest {
        method: "POST".to_string(),
        url: AUDIT_LOG_ENDPOINT.to_string(),
        headers,
        body,
    })
}

//TODO: use a proper search query type
pub fn build_search_request(context: &Context, api_token: &String, query: String) -> RpcResult<HttpRequest> {
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
        ("Content-Type".to_string(), vec!["application/json".to_string()]),
        ("Authorization".to_string(), vec![format!("Bearer {api_token}")]),
    ]);
    headers
}

pub async fn get_api_token(ctx: &Context) -> RpcResult<String> {
    let kv = KeyValueSender::new();
    let api_token = kv.get(ctx, API_KEY).await?.value;
    Ok(api_token)
}