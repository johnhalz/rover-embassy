use crate::types::{StanceConfig, StanceType, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct Stance {
    obstacle_rx: mpsc::Receiver<StanceRequest>,
    goal_rx: mpsc::Receiver<StanceRequest>,
    obstacle_tx: mpsc::Sender<StanceConfig>,
    goal_tx: mpsc::Sender<StanceConfig>,
    behavior_tx: mpsc::Sender<StanceConfig>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    current_stance: StanceConfig,
}

#[derive(Debug, Clone)]
pub enum StanceRequest {
    Query,
    Adjust(StanceConfig),
}

impl Stance {
    pub fn new(
        obstacle_rx: mpsc::Receiver<StanceRequest>,
        goal_rx: mpsc::Receiver<StanceRequest>,
        obstacle_tx: mpsc::Sender<StanceConfig>,
        goal_tx: mpsc::Sender<StanceConfig>,
        behavior_tx: mpsc::Sender<StanceConfig>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            obstacle_rx,
            goal_rx,
            obstacle_tx,
            goal_tx,
            behavior_tx,
            log_tx,
            shutdown_rx,
            current_stance: StanceConfig {
                stance_type: StanceType::Normal,
                stability: 1.0,
            },
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "Stance",
            LogLevel::Info,
            "Starting stance controller".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "Stance",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(request) = self.obstacle_rx.recv() => {
                    let tx = self.obstacle_tx.clone();
                    self.handle_request(request, &tx).await;
                }
                Some(request) = self.goal_rx.recv() => {
                    let tx = self.goal_tx.clone();
                    self.handle_request(request, &tx).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "Stance",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn handle_request(&mut self, request: StanceRequest, response_tx: &mpsc::Sender<StanceConfig>) {
        match request {
            StanceRequest::Query => {
                let _ = response_tx.send(self.current_stance.clone()).await;
            }
            StanceRequest::Adjust(new_stance) => {
                let _ = self.log_tx.send(create_log(
                    "Stance",
                    LogLevel::Info,
                    format!("Adjusting stance: {:?}", new_stance.stance_type)
                )).await;

                self.current_stance = new_stance.clone();
                let _ = self.behavior_tx.send(new_stance).await;
            }
        }
    }
}
