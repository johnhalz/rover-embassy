use crate::types::{SensorData, EnvironmentState, Obstacle, ObstacleType, TerrainType, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct EnvironmentUnderstanding {
    sensor_rx: mpsc::Receiver<SensorData>,
    env_state_tx: mpsc::Sender<EnvironmentState>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl EnvironmentUnderstanding {
    pub fn new(
        sensor_rx: mpsc::Receiver<SensorData>,
        env_state_tx: mpsc::Sender<EnvironmentState>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            sensor_rx,
            env_state_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "EnvUnderstanding",
            LogLevel::Info,
            "Starting environment understanding".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "EnvUnderstanding",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(sensor_data) = self.sensor_rx.recv() => {
                    let env_state = self.process_sensor_data(&sensor_data);

                    if !env_state.obstacles.is_empty() {
                        let _ = self.log_tx.send(create_log(
                            "EnvUnderstanding",
                            LogLevel::Info,
                            format!("Detected {} obstacles", env_state.obstacles.len())
                        )).await;
                    }

                    let _ = self.env_state_tx.send(env_state).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "EnvUnderstanding",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    fn process_sensor_data(&self, sensor_data: &SensorData) -> EnvironmentState {
        let mut obstacles = Vec::new();

        // Convert distance sensor readings to obstacles
        for (i, &distance) in sensor_data.distance_sensors.iter().enumerate() {
            if distance < 1.5 {
                // Close obstacle detected
                let angle = (i as f32) * std::f32::consts::PI / 2.0;
                let x = distance * angle.cos();
                let y = distance * angle.sin();

                obstacles.push(Obstacle {
                    position: [x, y, 0.0],
                    size: [0.3, 0.3, 0.5],
                    obstacle_type: ObstacleType::Static,
                });
            }
        }

        // Determine terrain type from IMU
        let accel_magnitude = (
            sensor_data.imu.acceleration[0].powi(2) +
            sensor_data.imu.acceleration[1].powi(2) +
            sensor_data.imu.acceleration[2].powi(2)
        ).sqrt();

        let terrain_type = if (accel_magnitude - 9.81).abs() > 2.0 {
            TerrainType::Rough
        } else if sensor_data.imu.orientation[1].abs() > 0.2 {
            TerrainType::Steep
        } else {
            TerrainType::Flat
        };

        EnvironmentState {
            obstacles,
            terrain_type,
            confidence: 0.8,
        }
    }
}
