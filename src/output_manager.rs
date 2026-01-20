use crate::types::{MotorCommand, StatusUpdate, RobotState, RobotPose, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use std::time::SystemTime;

pub struct OutputManager {
    motor_rx: mpsc::Receiver<MotorCommand>,
    hardware_tx: mpsc::Sender<MotorCommand>,
    feedback_tx: mpsc::Sender<StatusUpdate>,
    comm_tx: mpsc::Sender<StatusUpdate>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl OutputManager {
    pub fn new(
        motor_rx: mpsc::Receiver<MotorCommand>,
        hardware_tx: mpsc::Sender<MotorCommand>,
        feedback_tx: mpsc::Sender<StatusUpdate>,
        comm_tx: mpsc::Sender<StatusUpdate>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            motor_rx,
            hardware_tx,
            feedback_tx,
            comm_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "OutputManager",
            LogLevel::Info,
            "Starting output manager".to_string()
        )).await;

        let mut command_count = 0;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "OutputManager",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(motor_cmd) = self.motor_rx.recv() => {
                    // Forward to hardware interface
                    if let Err(_) = self.hardware_tx.send(motor_cmd.clone()).await {
                        let _ = self.log_tx.send(create_log(
                            "OutputManager",
                            LogLevel::Error,
                            "Failed to send motor command to hardware".to_string()
                        )).await;
                    }

                    command_count += 1;

                    // Periodically send status updates
                    if command_count % 5 == 0 {
                        self.send_status_update().await;
                    }
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "OutputManager",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn send_status_update(&mut self) {
        let status = StatusUpdate {
            timestamp: SystemTime::now(),
            state: RobotState::ExecutingMission,
            pose: RobotPose {
                position: [0.0, 0.0, 0.0],
                orientation: [1.0, 0.0, 0.0, 0.0],
                velocity: [0.5, 0.0, 0.0],
                angular_velocity: [0.0, 0.0, 0.0],
            },
            current_mission: Some("Patrol Mission".to_string()),
            battery_level: 0.75,
        };

        let _ = self.feedback_tx.send(status.clone()).await;
        let _ = self.comm_tx.send(status).await;
    }
}
