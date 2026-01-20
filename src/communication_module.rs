use crate::types::{StatusUpdate, UserFeedback, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct CommunicationModule {
    status_rx: mpsc::Receiver<StatusUpdate>,
    feedback_rx: mpsc::Receiver<UserFeedback>,
    user_instructions_tx: mpsc::Sender<String>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl CommunicationModule {
    pub fn new(
        status_rx: mpsc::Receiver<StatusUpdate>,
        feedback_rx: mpsc::Receiver<UserFeedback>,
        user_instructions_tx: mpsc::Sender<String>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            status_rx,
            feedback_rx,
            user_instructions_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "CommunicationModule",
            LogLevel::Info,
            "Starting communication module".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "CommunicationModule",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(status) = self.status_rx.recv() => {
                    self.handle_status(status).await;
                }
                Some(feedback) = self.feedback_rx.recv() => {
                    self.handle_feedback(feedback).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "CommunicationModule",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn handle_status(&mut self, status: StatusUpdate) {
        // In a real system, this would send telemetry to remote systems
        let _ = self.log_tx.send(create_log(
            "CommunicationModule",
            LogLevel::Debug,
            format!("Broadcasting status update: {:?}", status.state)
        )).await;
    }

    async fn handle_feedback(&mut self, feedback: UserFeedback) {
        // Forward feedback back to user instructions
        let _ = self.user_instructions_tx.send(feedback.message.clone()).await;

        let _ = self.log_tx.send(create_log(
            "CommunicationModule",
            LogLevel::Debug,
            format!("Relayed feedback: {}", feedback.message)
        )).await;
    }
}
