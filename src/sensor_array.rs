use crate::types::{SensorData, ImuData, GpsData, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, sleep};
use std::time::SystemTime;

pub struct SensorArray {
    sensor_tx: mpsc::Sender<SensorData>,
    safety_sensor_tx: mpsc::Sender<SensorData>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl SensorArray {
    pub fn new(
        sensor_tx: mpsc::Sender<SensorData>,
        safety_sensor_tx: mpsc::Sender<SensorData>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            sensor_tx,
            safety_sensor_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "SensorArray",
            LogLevel::Info,
            "Starting sensor array".to_string()
        )).await;

        let mut counter = 0;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "SensorArray",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                _ = sleep(Duration::from_millis(500)) => {
                    let sensor_data = self.generate_sensor_data(counter);

                    // Send to both input manager and safety controller
                    if let Err(_) = self.sensor_tx.send(sensor_data.clone()).await {
                        let _ = self.log_tx.send(create_log(
                            "SensorArray",
                            LogLevel::Error,
                            "Failed to send sensor data to input manager".to_string()
                        )).await;
                    }

                    if let Err(_) = self.safety_sensor_tx.send(sensor_data).await {
                        let _ = self.log_tx.send(create_log(
                            "SensorArray",
                            LogLevel::Error,
                            "Failed to send sensor data to safety controller".to_string()
                        )).await;
                    }

                    counter += 1;

                    if counter % 10 == 0 {
                        let _ = self.log_tx.send(create_log(
                            "SensorArray",
                            LogLevel::Debug,
                            format!("Published sensor reading #{}", counter)
                        )).await;
                    }
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "SensorArray",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    fn generate_sensor_data(&self, counter: u64) -> SensorData {
        // Simulate sensor readings with some variation
        let time = counter as f32 * 0.5;

        SensorData {
            timestamp: SystemTime::now(),
            distance_sensors: vec![
                2.5 + (time * 0.1).sin() * 0.5,  // Front
                3.0 + (time * 0.15).sin() * 0.3, // Left
                3.0 + (time * 0.15).cos() * 0.3, // Right
                5.0,                              // Back
            ],
            imu: ImuData {
                acceleration: [
                    (time * 0.05).sin() * 0.1,
                    (time * 0.05).cos() * 0.1,
                    9.81,
                ],
                gyroscope: [
                    (time * 0.02).sin() * 0.01,
                    (time * 0.02).cos() * 0.01,
                    0.0,
                ],
                orientation: [1.0, 0.0, 0.0, 0.0], // Identity quaternion
            },
            gps: GpsData {
                latitude: 37.7749 + ((time * 0.0001).sin() * 0.0001) as f64,
                longitude: -122.4194 + ((time * 0.0001).cos() * 0.0001) as f64,
                altitude: 10.0 + (time * 0.01).sin(),
                accuracy: 2.5,
            },
            battery_level: 0.85 - (counter as f32 * 0.0001).min(0.3),
        }
    }
}
