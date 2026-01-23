use crate::types::{Path, StanceConfig, BehaviorCommand, Behavior, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};
use std::time::SystemTime;

pub struct BehaviourModule {
    goal_path_rx: mpsc::Receiver<Path>,
    obstacle_path_rx: mpsc::Receiver<Path>,
    stance_rx: mpsc::Receiver<StanceConfig>,
    safety_controller_tx: mpsc::Sender<BehaviorCommand>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl BehaviourModule {
    pub fn new(
        goal_path_rx: mpsc::Receiver<Path>,
        obstacle_path_rx: mpsc::Receiver<Path>,
        stance_rx: mpsc::Receiver<StanceConfig>,
        safety_controller_tx: mpsc::Sender<BehaviorCommand>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            goal_path_rx,
            obstacle_path_rx,
            stance_rx,
            safety_controller_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "Behaviour",
            LogLevel::Info,
            "Starting behaviour module".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "Behaviour",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(path) = self.goal_path_rx.recv() => {
                    self.execute_path(path, "goal planning").await;
                }
                Some(path) = self.obstacle_path_rx.recv() => {
                    self.execute_path(path, "obstacle avoidance").await;
                }
                Some(stance) = self.stance_rx.recv() => {
                    self.adjust_for_stance(stance).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "Behaviour",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn execute_path(&mut self, path: Path, source: &str) {
        let _ = self.log_tx.send(create_log(
            "Behaviour",
            LogLevel::Info,
            format!("Executing path from {} with {} waypoints", source, path.waypoints.len())
        )).await;

        if let Some(first_waypoint) = path.waypoints.first() {
            let behavior = BehaviorCommand {
                timestamp: SystemTime::now(),
                behavior: Behavior::MoveTowards {
                    target: first_waypoint.position,
                    speed: 0.5,
                },
                priority: 5,
            };

            if let Err(_) = self.safety_controller_tx.send(behavior).await {
                let _ = self.log_tx.send(create_log(
                    "Behaviour",
                    LogLevel::Error,
                    "Failed to send behavior command to safety controller".to_string()
                )).await;
            }
        }
    }

    async fn adjust_for_stance(&mut self, stance: StanceConfig) {
        let _ = self.log_tx.send(create_log(
            "Behaviour",
            LogLevel::Debug,
            format!("Adjusting behavior for stance: {:?}", stance.stance_type)
        )).await;

        let behavior = BehaviorCommand {
            timestamp: SystemTime::now(),
            behavior: Behavior::AdjustStance(stance),
            priority: 7,
        };

        let _ = self.safety_controller_tx.send(behavior).await;
    }
}
