use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_httpserver::{HttpRequest, HttpResponse, HttpServer, HttpServerReceiver};

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, HttpServer)]
struct HttpGatewayActor {}

/// Implementation of the HttpServer capability contract
#[async_trait]
impl HttpServer for HttpGatewayActor {
    async fn handle_request(&self, _ctx: &Context, _req: &HttpRequest) -> RpcResult<HttpResponse> {
        Ok(HttpResponse::ok("Hello, World!"))
    }
}
