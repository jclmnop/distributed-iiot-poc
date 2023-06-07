use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

// ms
pub type PollInterval = u64;

//TODO: strict new types for valid NATS topics etc

#[derive(Deserialize, Serialize, Clone)]
pub struct Sensor {
    pub alias: String,
    pub id: Uuid,
    pub poll_interval: PollInterval,
    pub poll_topic: String,
    pub read_topic: String, // Only necessary because MQTT v3.* doesn't have reply topics for req/resp
    pub disconnect_topic: String,
    pub ip_addr: IpAddr,
    pub mac_addr: MacAddr6, // TODO: EUI-64 format,
    pub location: String,   // TODO: gps coords instead? location "name" could be part of alias
                            // TODO:
                            //  - value type
                            //  - value range
                            //  - multiple channels? (e.g. for particle sensors which have channels for each particle size)
                            //  - calibration values (offset etc)
}
