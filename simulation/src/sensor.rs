use async_nats::connect;
use futures::StreamExt;
use macaddr::MacAddr6;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};
use std::ops::Range;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

const HEARTBEAT_TOPIC: &str = "sim.heartbeat";

#[derive(Serialize, Clone)]
pub struct SensorSimulator {
    pub alias: String,
    pub id: Uuid,
    pub poll_interval: u64,
    pub poll_topic: String,
    pub read_topic: String,
    pub disconnect_topic: String,
    pub ip_addr: IpAddr,
    pub mac_addr: MacAddr6,
    pub location: String,

    #[serde(skip)]
    pub sim_range: Range<f64>,
    #[serde(skip)]
    pub normal: Normal<f64>,
    #[serde(skip)]
    pub nats_client: async_nats::Client,
}

impl SensorSimulator {
    const DEFAULT_POLL_INTERVAL: u64 = 10_000;

    pub fn new(
        alias: String,
        location: String,
        sim_range: Range<f64>,
        mean: f64,
        std_dev: f64,
        poll_interval: Option<u64>,
        nats_client: async_nats::Client,
    ) -> Self {
        let id = Uuid::new_v4();
        let mut rng = rand::thread_rng();
        let mut mac_addr = [0u8; 6];
        rng.fill(&mut mac_addr[..]);

        Self {
            alias,
            id,
            poll_interval: poll_interval.unwrap_or(Self::DEFAULT_POLL_INTERVAL),
            poll_topic: format!("sim.poll.{}", id),
            read_topic: format!("sim.read.{}", id),
            disconnect_topic: format!("sim.disconnect/{}", id),
            location: format!("SIMULATION-{}", location),
            mac_addr: MacAddr6::from(mac_addr),
            ip_addr: IpAddr::V4(Ipv4Addr::from(rng.gen::<u32>())),
            sim_range,
            normal: Normal::new(mean, std_dev).unwrap(),
            nats_client,
        }
    }

    pub fn read(&self) -> f64 {
        let mut rng = rand::thread_rng();
        let val = self.normal.sample(&mut rng);

        let val = if val < self.sim_range.start {
            self.sim_range.start
        } else if val > self.sim_range.end {
            self.sim_range.end
        } else {
            val
        };

        truncate_to_2_decimal_places(val)
    }

    pub async fn run(self) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn(async move {
            let faulty = self.alias.contains("FAULTY");
            println!(
                "Starting sensor simulator for {}.{}",
                self.location, self.alias
            );
            // Spawn heartbeat task
            let nats_client = self.nats_client.clone();
            let heartbeat_msg = serde_json::to_string(&self).unwrap();
            tokio::task::spawn(async move {
                loop {
                    let msg = heartbeat_msg.clone();
                    nats_client
                        .publish(HEARTBEAT_TOPIC.into(), msg.into())
                        .await
                        .unwrap();
                    sleep(Duration::from_secs(60)).await;
                }
            });

            let mut subscription = self
                .nats_client
                .subscribe(self.poll_topic.to_owned())
                .await
                .unwrap();
            while let Some(_msg) = subscription.next().await {
                if faulty {
                    let mut rng = rand::thread_rng();
                    if rng.gen::<u8>() < 25 {
                        continue;
                    }
                }
                let val = self.read();
                println!("{}.{}: \t{}", self.location, self.alias, val);
                let msg = format!("\"{val}\"");
                self.nats_client
                    .publish(self.read_topic.to_owned(), msg.into())
                    .await
                    .unwrap();
            }
        })
    }
}

fn truncate_to_2_decimal_places(float: f64) -> f64 {
    (float * 100.0).trunc() / 100.0
}
