use actor_interfaces::{PangeaApi, PangeaApiSender, SearchParams, SearchResponse};
use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};
use wasmcloud_interface_logging::{debug, error, info};

const PANGEA_API_ACTOR: &str = "iiot/pangea_api";

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
            ("GET", "") => get_ui(ctx, req).await,
            ("POST", "api/logs") => get_logs(ctx, req).await,
            ("POST", "api/results") => page_results(ctx, req).await,
            _ => {
                error!("Received request for unknown path: {}", path);
                Ok(HttpResponse::not_found())
            }
        }
        .or(Ok(HttpResponse::internal_server_error(
            "Error handling request",
        )))
    }
}

async fn get_ui(ctx: &Context, req: &HttpRequest) -> RpcResult<HttpResponse> {
    info!("Received request for root");
    debug!("Request: {:?}", req);
    todo!()
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
