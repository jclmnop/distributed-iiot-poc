use actor_interfaces::{PangeaApi, PangeaApiSender, SearchParams, SearchResponse, ui::{Ui, UiSender, GetAssetResponse}};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HeaderMap, HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::{debug, error, info};

const PANGEA_API_ACTOR: &str = "iiot/pangea_api";
const UI_ACTOR: &str = "iiot/ui";

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct HttpGatewayActor {}

/// Implementation of the HttpServer capability contract
#[async_trait]
impl HttpServer for HttpGatewayActor {
    async fn handle_request(&self, ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
        let path = req.path.trim_matches(|c| c == ' ' || c == '/');
        let method = req.method.to_ascii_uppercase();
        match (method.as_str(), path) {
            ("POST", "api/logs") => get_logs(ctx, req).await,
            ("POST", "api/results") => page_results(ctx, req).await,
            ("GET", _) => get_ui(ctx, req).await,
            _ => {
                error!("Received POST request for unknown path: {}", path);
                Ok(HttpResponse::not_found())
            }
        }
        .or(Ok(HttpResponse::internal_server_error(
            "Error handling request",
        )))
    }
}

async fn get_ui(ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
    info!("Assuming request for path {} is for UI asset", req.path);
    debug!("Request: {:?}", req);
    let ui: UiSender<_> = UiSender::to_actor(UI_ACTOR);
    let resp: GetAssetResponse = ui.get_asset(ctx, &req.path).await?;
    if resp.found {
        info!("Successfully retrieved asset of {} bytes from path {}", resp.asset.len(), req.path);
        let mut header = HeaderMap::new();
        if let Some(content_type) = resp.content_type {
            debug!("Setting content type to {}", content_type);
            header.insert("Content-Type".to_string(), vec![content_type]);
        }
        Ok(HttpResponse {
            status_code: 200,
            header,
            body: resp.asset,
        })
    } else {
        error!("Could not find asset from path {}", req.path);
        Ok(HttpResponse::not_found())
    }
}

async fn get_logs(ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
    info!("Received request for logs");
    debug!("Request: {:?}", req);
    match serde_json::from_slice::<SearchParams>(req.body.as_slice()) {
        Ok(search_params) => {
            let pangea_api: PangeaApiSender<_> = PangeaApiSender::to_actor(PANGEA_API_ACTOR);
            let search_result: SearchResponse =
                pangea_api.search_audit_log(ctx, &search_params).await?;
            if search_result.status == "Success" {
                info!("Successfully retrieved audit log");
                Ok(HttpResponse::json(&search_result, 200)?)
            } else {
                error!("Error retrieving audit log: {}", search_result.summary);
                Ok(HttpResponse::internal_server_error(format!(
                    "Error retrieving audit log: {}",
                    search_result.summary
                )))
            }
        }
        Err(e) => {
            error!("Failed to deserialize search params: {e:?}");
            Ok(HttpResponse::bad_request(
                "Failed to deserialize search params",
            ))
        }
    }
}

async fn page_results(ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
    info!("Received request for results page");
    debug!("Request: {:?}", req);
    todo!()
}
