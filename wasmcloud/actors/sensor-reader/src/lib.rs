use wasmbus_rpc::actor::prelude::*;
use wasmcloud_interface_polling::{PollResult, PollSubscriber, PollSubscriberReceiver};
use wasmcloud_interface_logging::{error, debug, info};
use actor_interfaces::{PangeaApiSender, LogEvent, PangeaApi};

const PANGEA_API_ACTOR: &str = "pangea_api";

#[derive(Debug, Default, Actor, HealthResponder)]
#[services(Actor, PollSubscriber)]
struct SensorReaderActor {}

#[async_trait]
impl PollSubscriber for SensorReaderActor {
    async fn poll_rx(&self, ctx: &Context, poll_result: &PollResult) -> RpcResult<()> {
        info!("sensor_reader: processing received poll result");
        if let Some(poll_error) = poll_result.error.clone() {
            error!(
                "Error polling sensors, \n\tERROR_TYPE: {}\n\tDESCRIPTION: {}\n",
                poll_error.error_type,
                poll_error.description.unwrap_or("n/a")
            );
        } else {
            if let Some(poll_readings) = data {
                match serde_json::from_slice::<Vec<LogEvent>>(&poll_readings) {
                    Ok(poll_readings) => {
                        info!("Sending readings to event log");
                        let pangea_api = PangeaApiSender::to_actor(PANGEA_API_ACTOR);
                        let write_result = pangea_api.write_audit_log(ctx, &poll_readings).await?;
                        if write_result.success {
                            info!("Successfully wrote readings to audit log");
                        } else {
                            error!("Error writing to audit log: {}", write_result.reason.unwrap_or(""))
                        }
                    }
                    Err(e) => {
                        error!("Failed to deserialize sensor readings: {e}");
                    }
                }
            } else {
                error!("No sensor readings contained in poll result");
            }
        }
        Ok(())
    }
}


