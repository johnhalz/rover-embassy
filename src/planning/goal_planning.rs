use crate::types::{Goal, Path, RobotPose, StanceConfig, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use crate::perception::stance::StanceRequest;
use tokio::sync::{broadcast, mpsc};

pub struct GoalPlanning {
    goal_rx: mpsc::Receiver<Goal>,
    stance_query_tx: mpsc::Sender<StanceRequest>,
    stance_rx: mpsc::Receiver<StanceConfig>,
    obstacle_tx: mpsc::Sender<PathRequest>,
    obstacle_rx: mpsc::Receiver<Path>,
    behavior_tx: mpsc::Sender<Path>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
}

#[derive(Debug, Clone)]
pub enum PathRequest {
    Plan { start: RobotPose, goal: RobotPose },
}

impl GoalPlanning {
    pub fn new(
        goal_rx: mpsc::Receiver<Goal>,
        stance_query_tx: mpsc::Sender<StanceRequest>,
        stance_rx: mpsc::Receiver<StanceConfig>,
        obstacle_tx: mpsc::Sender<PathRequest>,
        obstacle_rx: mpsc::Receiver<Path>,
        behavior_tx: mpsc::Sender<Path>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            goal_rx,
            stance_query_tx,
            stance_rx,
            obstacle_tx,
            obstacle_rx,
            behavior_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "GoalPlanning",
            LogLevel::Info,
            "Starting goal planning".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "GoalPlanning",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(goal) = self.goal_rx.recv() => {
                    self.plan_to_goal(goal).await;
                }
                Some(path) = self.obstacle_rx.recv() => {
                    // Received validated path from obstacle avoidance
                    let _ = self.log_tx.send(create_log(
                        "GoalPlanning",
                        LogLevel::Info,
                        format!("Received safe path with {} waypoints", path.waypoints.len())
                    )).await;
                    let _ = self.behavior_tx.send(path).await;
                }
                Some(stance_config) = self.stance_rx.recv() => {
                    // Received stance configuration response
                    let _ = self.log_tx.send(create_log(
                        "GoalPlanning",
                        LogLevel::Debug,
                        format!("Received stance config: stability={:.2}", stance_config.stability)
                    )).await;
                    // Use stance config to adjust planning parameters
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "GoalPlanning",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn plan_to_goal(&mut self, goal: Goal) {
        let _ = self.log_tx.send(create_log(
            "GoalPlanning",
            LogLevel::Info,
            format!("Planning path to goal: {:?}", goal.goal_type)
        )).await;

        // Query current stance
        let _ = self.stance_query_tx.send(StanceRequest::Query).await;

        // Create a simple path to goal
        let start = RobotPose {
            position: [0.0, 0.0, 0.0],
            orientation: [1.0, 0.0, 0.0, 0.0],
            velocity: [0.0, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
        };

        // Request path validation from obstacle avoidance
        let _ = self.obstacle_tx.send(PathRequest::Plan {
            start,
            goal: goal.target_pose,
        }).await;
    }
}
