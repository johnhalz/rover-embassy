use crate::types::{HardwareStatus, HealthStatus, MotorCommand, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, sleep};
use std::time::SystemTime;

pub struct HardwareInterface {
    status_tx: mpsc::Sender<HardwareStatus>,
    motor_rx: mpsc::Receiver<MotorCommand>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl HardwareInterface {
    pub fn new(
        status_tx: mpsc::Sender<HardwareStatus>,
        motor_rx: mpsc::Receiver<MotorCommand>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            status_tx,
            motor_rx,
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
                Some(motor_cmd) = self.motor_rx.recv() => {
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
