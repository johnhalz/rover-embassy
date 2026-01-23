use crate::types::{HardwareStatus, HealthStatus, MotorCommand, SensorData, BehaviorCommand, Behavior, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, sleep};
use std::time::SystemTime;

pub struct HardwareInterface {
    // Inputs
    sensor_rx: mpsc::Receiver<SensorData>,
    behavior_rx: mpsc::Receiver<BehaviorCommand>,
    motor_rx: mpsc::Receiver<MotorCommand>,
    
    // Outputs
    sensor_tx: mpsc::Sender<SensorData>,
    status_tx: mpsc::Sender<HardwareStatus>,
    
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl HardwareInterface {
    pub fn new(
        sensor_rx: mpsc::Receiver<SensorData>,
        behavior_rx: mpsc::Receiver<BehaviorCommand>,
        motor_rx: mpsc::Receiver<MotorCommand>,
        sensor_tx: mpsc::Sender<SensorData>,
        status_tx: mpsc::Sender<HardwareStatus>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            sensor_rx,
            behavior_rx,
            motor_rx,
            sensor_tx,
            status_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "HardwareInterface",
            LogLevel::Info,
            "Starting hardware interface".to_string()
        )).await;

        let mut counter = 0;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "HardwareInterface",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(sensor_data) = self.sensor_rx.recv() => {
                    // Forward sensor data to Input Manager
                    if let Err(_) = self.sensor_tx.send(sensor_data).await {
                        let _ = self.log_tx.send(create_log(
                            "HardwareInterface",
                            LogLevel::Error,
                            "Failed to forward sensor data to input manager".to_string()
                        )).await;
                    }
                }
                Some(behavior_cmd) = self.behavior_rx.recv() => {
                    // Convert behavior command to motor command and execute
                    self.handle_behavior_command(behavior_cmd).await;
                }
                Some(motor_cmd) = self.motor_rx.recv() => {
                    // Handle direct motor commands (for backward compatibility)
                    let _ = self.log_tx.send(create_log(
                        "HardwareInterface",
                        LogLevel::Debug,
                        format!("Motor command: L={:.2}, R={:.2}",
                            motor_cmd.left_speed, motor_cmd.right_speed)
                    )).await;
                }
                _ = sleep(Duration::from_secs(2)) => {
                    let status = self.generate_hardware_status(counter);

                    if let Err(_) = self.status_tx.send(status).await {
                        let _ = self.log_tx.send(create_log(
                            "HardwareInterface",
                            LogLevel::Error,
                            "Failed to send hardware status".to_string()
                        )).await;
                    }

                    counter += 1;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "HardwareInterface",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn handle_behavior_command(&mut self, cmd: BehaviorCommand) {
        match cmd.behavior {
            Behavior::MoveTowards { target, speed } => {
                let motor_cmd = self.calculate_motor_command(target, speed);
                let _ = self.log_tx.send(create_log(
                    "HardwareInterface",
                    LogLevel::Debug,
                    format!("Executing MoveTowards: L={:.2}, R={:.2}",
                        motor_cmd.left_speed, motor_cmd.right_speed)
                )).await;
            }
            Behavior::AvoidObstacle { direction } => {
                let motor_cmd = MotorCommand {
                    left_speed: direction[1] * 0.5,
                    right_speed: -direction[1] * 0.5,
                };
                let _ = self.log_tx.send(create_log(
                    "HardwareInterface",
                    LogLevel::Debug,
                    format!("Executing AvoidObstacle: L={:.2}, R={:.2}",
                        motor_cmd.left_speed, motor_cmd.right_speed)
                )).await;
            }
            Behavior::EmergencyStop => {
                let _ = self.log_tx.send(create_log(
                    "HardwareInterface",
                    LogLevel::Warn,
                    "Emergency stop executed".to_string()
                )).await;
            }
            Behavior::AdjustStance(_) => {
                // Stance adjustments are handled by the stance module
                let _ = self.log_tx.send(create_log(
                    "HardwareInterface",
                    LogLevel::Debug,
                    "Stance adjustment received".to_string()
                )).await;
            }
            Behavior::Idle => {
                // No action needed
            }
        }
    }

    fn calculate_motor_command(&self, target: [f32; 3], speed: f32) -> MotorCommand {
        // Simplified differential drive calculation
        let angle_to_target = target[1].atan2(target[0]);
        let turn_factor = angle_to_target.sin();

        MotorCommand {
            left_speed: speed * (1.0 - turn_factor * 0.5),
            right_speed: speed * (1.0 + turn_factor * 0.5),
        }
    }

    fn generate_hardware_status(&self, counter: u64) -> HardwareStatus {
        let voltage = 12.6 - (counter as f32 * 0.01).min(0.5);

        HardwareStatus {
            timestamp: SystemTime::now(),
            battery_voltage: voltage,
            motor_temps: vec![45.0, 46.5, 44.8, 47.2],
            health: if voltage > 11.5 {
                HealthStatus::Healthy
            } else {
                HealthStatus::Warning("Low battery voltage".to_string())
            },
        }
    }
}
