use crate::types::{UserCommand, MissionCommand, Waypoint, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, sleep};

pub struct UserInstructions {
    command_tx: mpsc::Sender<UserCommand>,
    feedback_rx: mpsc::Receiver<String>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl UserInstructions {
    pub fn new(
        command_tx: mpsc::Sender<UserCommand>,
        feedback_rx: mpsc::Receiver<String>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            command_tx,
            feedback_rx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "UserInstructions",
            LogLevel::Info,
            "Starting user instructions module".to_string()
        )).await;

        // Simulate receiving a mission command after a delay
        let mut mission_sent = false;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "UserInstructions",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(feedback) = self.feedback_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "UserInstructions",
                        LogLevel::Info,
                        format!("Received feedback: {}", feedback)
                    )).await;
                }
                _ = sleep(Duration::from_secs(5)), if !mission_sent => {
                    // Send a patrol mission
                    let mission = UserCommand::MissionCommand(MissionCommand::Patrol {
                        waypoints: vec![
                            Waypoint { lat: 37.7749, lon: -122.4194, tolerance: 2.0 },
                            Waypoint { lat: 37.7750, lon: -122.4195, tolerance: 2.0 },
                            Waypoint { lat: 37.7751, lon: -122.4196, tolerance: 2.0 },
                        ],
                        loops: 2,
                    });

                    let _ = self.log_tx.send(create_log(
                        "UserInstructions",
                        LogLevel::Info,
                        "Sending patrol mission".to_string()
                    )).await;

                    if let Err(_) = self.command_tx.send(mission).await {
                        let _ = self.log_tx.send(create_log(
                            "UserInstructions",
                            LogLevel::Error,
                            "Failed to send mission command".to_string()
                        )).await;
                    }

                    mission_sent = true;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "UserInstructions",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }
}
