use crate::types::{StatusUpdate, UserFeedback, FeedbackType, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct UserFeedbackModule {
    status_rx: mpsc::Receiver<StatusUpdate>,
    comm_tx: mpsc::Sender<UserFeedback>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl UserFeedbackModule {
    pub fn new(
        status_rx: mpsc::Receiver<StatusUpdate>,
        comm_tx: mpsc::Sender<UserFeedback>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            status_rx,
            comm_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "UserFeedback",
            LogLevel::Info,
            "Starting user feedback module".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "UserFeedback",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(status) = self.status_rx.recv() => {
                    self.display_status(&status).await;
                    self.forward_to_comm(&status).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "UserFeedback",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn display_status(&mut self, status: &StatusUpdate) {
        let mission_str = status.current_mission.as_ref()
            .map(|m| m.as_str())
            .unwrap_or("None");

        let _ = self.log_tx.send(create_log(
            "UserFeedback",
            LogLevel::Info,
            format!(
                "Status: {:?} | Mission: {} | Battery: {:.0}%",
                status.state,
                mission_str,
                status.battery_level * 100.0
            )
        )).await;
    }

    async fn forward_to_comm(&mut self, status: &StatusUpdate) {
        let feedback = UserFeedback {
            message: format!(
                "State: {:?}, Battery: {:.0}%",
                status.state,
                status.battery_level * 100.0
            ),
            feedback_type: FeedbackType::Status,
        };

        let _ = self.comm_tx.send(feedback).await;
    }
}
