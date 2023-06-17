use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

// ms
pub type PollInterval = u64;

// TODO: strict new types for valid NATS topics etc

// TODO extra sensor fields:
//  - value type
//  - value range
//  - multiple channels? (e.g. for particle sensors which have channels for each particle size)
//      - could implement using an enum for value type, with one of the options being an array
//  - calibration values (offset etc)
#[derive(Deserialize, Serialize, Clone)]
pub struct Sensor {
    pub alias: String,
    pub id: Uuid,
    pub poll_interval: PollInterval,
    pub poll_topic: String,
    pub read_topic: String, // Only necessary because MQTT v3.* doesn't have reply topics for req/resp
    pub disconnect_topic: String,
    pub ip_addr: IpAddr,
    pub mac_addr: MacAddr6, // TODO: + EUI-64 format,
    pub location: String,   // TODO: gps coords instead? location "name" could be part of alias
}

// TODO: unit tests, mostly so i can make sure i use the correct format
//       in the micropython code for the sensor
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_deserialize() {
        let sensor_json = r#"
            {
                "alias": "test-sensor",
                "id": "a3b3c3d3-e3f3-a3b3-c3d3-e3f3a3b3c3d3",
                "poll_interval": 1000,
                "poll_topic": "test-sensor/poll",
                "read_topic": "test-sensor/read",
                "disconnect_topic": "test-sensor/disconnect",
                "ip_addr": "192.168.1.1",
                "mac_addr": [0, 1, 2, 3, 4, 255],
                "location": "test-location"
           }
           "#;
        let sensor: Sensor = serde_json::from_str(sensor_json).unwrap();
        assert_eq!(sensor.alias, "test-sensor");
        assert_eq!(
            sensor.id,
            Uuid::parse_str("a3b3c3d3-e3f3-a3b3-c3d3-e3f3a3b3c3d3").unwrap()
        );
        assert_eq!(sensor.poll_interval, 1000);
        assert_eq!(sensor.poll_topic, "test-sensor/poll");
        assert_eq!(sensor.read_topic, "test-sensor/read");
        assert_eq!(sensor.disconnect_topic, "test-sensor/disconnect");
        assert_eq!(sensor.ip_addr, IpAddr::from([192, 168, 1, 1]));
        assert_eq!(sensor.mac_addr, MacAddr6::new(0, 1, 2, 3, 4, 0xff));
        assert_eq!(sensor.location, "test-location");
    }

    #[test]
    fn test_picow_deserialize() {
        let sensor_json = r#"
        {"disconnect_topic": "picow/f3088463-5623-476f-a1b5-ecb49446a443/disconnect", "poll_interval": 60000, "ip_addr": "1.2.3.4", "mac_addr": [40, 205, 193, 3, 226, 159], "id": "f3088463-5623-476f-a1b5-ecb49446a443", "location": "rp-pico-w", "poll_topic": "picow/f3088463-5623-476f-a1b5-ecb49446a443/poll", "alias": "temp_01", "read_topic": "picow/f3088463-5623-476f-a1b5-ecb49446a443/read"}
        "#;

        let sensor: Sensor = serde_json::from_str(sensor_json).unwrap();
        assert_eq!(sensor.alias, "temp_01");
        assert_eq!(
            sensor.id,
            Uuid::parse_str("f3088463-5623-476f-a1b5-ecb49446a443").unwrap()
        );
        assert_eq!(sensor.poll_interval, 60000);
        assert_eq!(sensor.poll_topic, "picow/f3088463-5623-476f-a1b5-ecb49446a443/poll");
        assert_eq!(sensor.read_topic, "picow/f3088463-5623-476f-a1b5-ecb49446a443/read");
        assert_eq!(sensor.disconnect_topic, "picow/f3088463-5623-476f-a1b5-ecb49446a443/disconnect");
        assert_eq!(sensor.ip_addr, IpAddr::from([1, 2, 3, 4]));
        assert_eq!(sensor.mac_addr, MacAddr6::new(40, 205, 193, 3, 226, 159));
        assert_eq!(sensor.location, "rp-pico-w");
    }

    #[test]
    fn test_sensor_serialize() {
        let sensor = Sensor {
            alias: "test-sensor".to_string(),
            id: Uuid::parse_str("a3b3c3d3-e3f3-a3b3-c3d3-e3f3a3b3c3d3").unwrap(),
            poll_interval: 1000,
            poll_topic: "test-sensor/poll".to_string(),
            read_topic: "test-sensor/read".to_string(),
            disconnect_topic: "test-sensor/disconnect".to_string(),
            ip_addr: IpAddr::from([192, 168, 1, 1]),
            mac_addr: MacAddr6::new(0, 1, 2, 3, 4, 255),
            location: "test-location".to_string(),
        };
        let sensor_json = serde_json::to_string_pretty(&sensor).unwrap();
        let expected_json = r#"{
  "alias": "test-sensor",
  "id": "a3b3c3d3-e3f3-a3b3-c3d3-e3f3a3b3c3d3",
  "poll_interval": 1000,
  "poll_topic": "test-sensor/poll",
  "read_topic": "test-sensor/read",
  "disconnect_topic": "test-sensor/disconnect",
  "ip_addr": "192.168.1.1",
  "mac_addr": [
    0,
    1,
    2,
    3,
    4,
    255
  ],
  "location": "test-location"
}"#;
        println!("{}", sensor_json);
        assert_eq!(sensor_json, expected_json);
    }
}
