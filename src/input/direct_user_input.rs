use crate::types::{UserCommand, ManualControl, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, sleep};

pub struct DirectUserInput {
    command_tx: mpsc::Sender<UserCommand>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl DirectUserInput {
    pub fn new(
        command_tx: mpsc::Sender<UserCommand>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            command_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "DirectUserInput",
            LogLevel::Info,
            "Starting direct user input handler".to_string()
        )).await;

        // Simulate user pressing forward, then turning
        let commands = vec![
            UserCommand::ManualControl(ManualControl::MoveForward(0.5)),
            UserCommand::ManualControl(ManualControl::TurnLeft(0.3)),
            UserCommand::ManualControl(ManualControl::MoveForward(0.7)),
            UserCommand::ManualControl(ManualControl::Stop),
        ];

        let mut cmd_idx = 0;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "DirectUserInput",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                _ = sleep(Duration::from_secs(3)) => {
                    if cmd_idx < commands.len() {
                        let command = commands[cmd_idx].clone();

                        let _ = self.log_tx.send(create_log(
                            "DirectUserInput",
                            LogLevel::Info,
                            format!("User input: {:?}", command)
                        )).await;

                        if let Err(_) = self.command_tx.send(command).await {
                            let _ = self.log_tx.send(create_log(
                                "DirectUserInput",
                                LogLevel::Error,
                                "Failed to send user command".to_string()
                            )).await;
                        }

                        cmd_idx += 1;
                    }
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "DirectUserInput",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }
}
