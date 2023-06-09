use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_logging::{error, info};
use wasmcloud_interface_messaging::{MessageSubscriber, SubMessage, MessageSubscriberReceiver};
use actor_interfaces::{LogEvent, PangeaApiSender, PangeaApi};

const PANGEA_API_ACTOR: &str = "pangea_api";

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, MessageSubscriber)]
struct EventLoggerActor {}

#[async_trait]
impl MessageSubscriber for EventLoggerActor {
    async fn handle_message(&self, ctx: &Context, msg: &SubMessage) -> RpcResult<()> {
        info!("Received event on subject {}", msg.subject);
        match serde_json::from_slice::<Vec<LogEvent>>(&msg.body) {
            Ok(events) => {
                let pangea_api = PangeaApiSender::to_actor(PANGEA_API_ACTOR);
                let write_result = pangea_api.write_audit_log(&ctx, &events).await?;
                if write_result.success {
                    info!("Successfully wrote event(s) to audit log");
                } else {
                    error!("Error writing event to audit log: {}", write_result.reason.unwrap_or("".to_string()))
                }
            }
            Err(e) => {
                error!("Failed to deserialize message body to event: {e:?}");
            }
        }
        Ok(())
    }
}


