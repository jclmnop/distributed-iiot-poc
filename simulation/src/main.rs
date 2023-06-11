//! Simulation of sensors for an IIoT system. This simulation is intended to be used with the
//! `sensor-reader` actor, which is responsible for polling the sensors and sending the results
//! to the `pangea_api` actor for storage in the audit log.
//!
//! It simply needs to be connected to a NATS leaf node which is connected to the cosmonic
//! super-constellation and can be hosted anywhere. Once the polling provider is linked to the
//! sensor-reader actor with a subscription to `sim.heartbeat` the simulation will run indefinitely.
//!
//! Note: This code is full of `.unwrap()` calls, but please don't judge me, it's just a simulation.

mod sensor;

use crate::sensor::SensorSimulator;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use rand::Rng;

static NATS_URL: Lazy<String> = Lazy::new(|| {
    let s = format!(
        "nats://{}",
        std::env::var("NATS_URL").unwrap_or("localhost:4222".to_string())
    );
    s
});

#[tokio::main]
async fn main() {
    let nats_client = async_nats::connect(NATS_URL.as_str()).await.unwrap();

    // Stagger the intervals out a bit so that the sensors don't all poll at the same time.
    // 10 minutes minimum poll interval so I don't use up all my free credit for pangea.
    let intervals = vec![600_000u64, 610_000u64, 620_000u64, 630_000u64, 640_000u64, 650_000u64, 660_000u64, 670_000u64, 680_000u64];

    let mut rng = rand::thread_rng();

    #[rustfmt::skip]
    let sensors_and_parameters = vec![
        ("TempSensor01", "Conveyor Belt 1", 0.0..50.0, 25.0, 2.0),
        ("FAULTY_HumiditySensor-A1", "Filling Line 3", 20.0..90.0, 55.0, 10.0),
        ("PressureSensor789", "Boiler Room", 1.0..10.0, 5.5, 1.0),
        ("VibrationSensorB", "Machine Room", 0.0..100.0, 50.0, 10.0),
        ("FlowSensor-R23", "Filling Line 3", 0.0..10.0, 5.0, 0.5),
        ("LevelSensor-L4", "Storage Tank 2", 0.0..100.0, 50.0, 5.0),
        ("RotationalSensor3", "Main Assembly Line", 0.0..360.0, 180.0, 30.0),
        ("AcousticSensor-X2", "Boiler Room", 20.0..120.0, 70.0, 10.0),
        ("ChemicalSensor-C17", "Chemical Storage", 0.0..14.0, 7.0, 1.0),
        ("RadiationSensor-R5", "Waste Disposal", 0.0..10.0, 0.5, 0.1),
        ("LightSensor-LS1", "Main Assembly Line", 0.0..1000.0, 500.0, 50.0),
        ("ProximitySensor-P3", "Loading Dock", 0.0..10.0, 5.0, 1.0),
        ("MagneticSensor-M7", "Machine Room", -100.0..100.0, 0.0, 20.0),
        ("SmokeSensor-S2", "Boiler Room", 0.0..100.0, 5.0, 2.0),
        ("MotionSensor-M1", "Security Gate", 0.0..1.0, 0.5, 0.1),
        ("TempSensor02", "Conveyor Belt 2", 0.0..50.0, 25.0, 2.0),
        ("HumiditySensor-A2", "Filling Line 1", 20.0..90.0, 55.0, 10.0),
        ("PressureSensor790", "HVAC System", 1.0..10.0, 5.5, 1.0),
        ("VibrationSensorC", "Machine Room 2", 0.0..100.0, 50.0, 10.0),
        ("FlowSensor-R24", "Filling Line 2", 0.0..10.0, 5.0, 0.5),
        ("LevelSensor-L5", "Storage Tank 3", 0.0..100.0, 50.0, 5.0),
        ("RotationalSensor4", "Auxiliary Assembly Line", 0.0..360.0, 180.0, 30.0),
        ("AcousticSensor-X3", "HVAC System", 20.0..120.0, 70.0, 10.0),
        ("ChemicalSensor-C18", "Chemical Storage 2", 0.0..14.0, 7.0, 1.0),
        ("RadiationSensor-R6", "Waste Disposal 2", 0.0..10.0, 0.5, 0.1),
        ("LightSensor-LS2", "Auxiliary Assembly Line", 0.0..1000.0, 500.0, 50.0),
        ("ProximitySensor-P4", "Unloading Dock", 0.0..10.0, 5.0, 1.0),
        ("MagneticSensor-M8", "Machine Room 3", -100.0..100.0, 0.0, 20.0),
        ("SmokeSensor-S3", "HVAC System", 0.0..100.0, 5.0, 2.0),
        ("MotionSensor-M2", "Security Gate 2", 0.0..1.0, 0.5, 0.1),
        ("TempSensor03", "Conveyor Belt 3", 0.0..50.0, 25.0, 2.0),
        ("HumiditySensor-A3", "Filling Line 2", 20.0..90.0, 55.0, 10.0),
        ("PressureSensor791", "Boiler Room 2", 1.0..10.0, 5.5, 1.0),
        ("VibrationSensorD", "Machine Room 4", 0.0..100.0, 50.0, 10.0),
        ("FlowSensor-R25", "Filling Line 1", 0.0..10.0, 5.0, 0.5),
        ("LevelSensor-L6", "Storage Tank 1", 0.0..100.0, 50.0, 5.0),
        ("RotationalSensor5", "Main Assembly Line", 0.0..360.0, 180.0, 30.0),
        ("AcousticSensor-X4", "Boiler Room 2", 20.0..120.0, 70.0, 10.0),
        ("ChemicalSensor-C19", "Chemical Storage 3", 0.0..14.0, 7.0, 1.0),
        ("RadiationSensor-R7", "Waste Disposal 3", 0.0..10.0, 0.5, 0.1),
    ];

    let mut sensors = Vec::new();

    for (alias, location, sim_range, mean, std_dev) in sensors_and_parameters {
        let poll_interval = *(intervals.choose(&mut rng).unwrap());
        sensors.push(
            SensorSimulator::new(
                alias.to_string(),
                location.to_string(),
                sim_range,
                mean,
                std_dev,
                Some(poll_interval),
                nats_client.clone(),
            )
            .run()
            .await,
        );
    }

    tokio::signal::ctrl_c().await.unwrap();
    println!("Ctrl-C received, shutting down...");
}
