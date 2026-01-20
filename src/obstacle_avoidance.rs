use crate::types::{EnvironmentState, Path, RobotPose, StanceConfig, LogEntry, LogLevel};
use crate::logger::create_log;
use crate::stance::StanceRequest;
use crate::goal_planning::PathRequest;
use tokio::sync::{broadcast, mpsc};

pub struct ObstacleAvoidance {
    env_state_rx: mpsc::Receiver<EnvironmentState>,
    stance_query_tx: mpsc::Sender<StanceRequest>,
    stance_rx: mpsc::Receiver<StanceConfig>,
    goal_path_rx: mpsc::Receiver<PathRequest>,
    goal_path_tx: mpsc::Sender<Path>,
    behavior_tx: mpsc::Sender<Path>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    current_env: Option<EnvironmentState>,
}

impl ObstacleAvoidance {
    pub fn new(
        env_state_rx: mpsc::Receiver<EnvironmentState>,
        stance_query_tx: mpsc::Sender<StanceRequest>,
        stance_rx: mpsc::Receiver<StanceConfig>,
        goal_path_rx: mpsc::Receiver<PathRequest>,
        goal_path_tx: mpsc::Sender<Path>,
        behavior_tx: mpsc::Sender<Path>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            env_state_rx,
            stance_query_tx,
            stance_rx,
            goal_path_rx,
            goal_path_tx,
            behavior_tx,
            log_tx,
            shutdown_rx,
            current_env: None,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "ObstacleAvoidance",
            LogLevel::Info,
            "Starting obstacle avoidance".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "ObstacleAvoidance",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(env_state) = self.env_state_rx.recv() => {
                    self.current_env = Some(env_state);
                }
                Some(path_request) = self.goal_path_rx.recv() => {
                    self.validate_path(path_request).await;
                }
                Some(stance_config) = self.stance_rx.recv() => {
                    // Received stance configuration response
                    let _ = self.log_tx.send(create_log(
                        "ObstacleAvoidance",
                        LogLevel::Debug,
                        format!("Received stance config: stability={:.2}", stance_config.stability)
                    )).await;
                    // Use stance config to adjust avoidance behavior
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "ObstacleAvoidance",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn validate_path(&mut self, request: PathRequest) {
        match request {
            PathRequest::Plan { start, goal } => {
                let _ = self.log_tx.send(create_log(
                    "ObstacleAvoidance",
                    LogLevel::Info,
                    "Validating and adjusting path for obstacles".to_string()
                )).await;

                // Query stance for navigation constraints
                let _ = self.stance_query_tx.send(StanceRequest::Query).await;

                // Create a safe path (simplified - just interpolate)
                let mut waypoints = Vec::new();
                for i in 0..5 {
                    let t = i as f32 / 4.0;
                    let pos = [
                        start.position[0] + t * (goal.position[0] - start.position[0]),
                        start.position[1] + t * (goal.position[1] - start.position[1]),
                        start.position[2] + t * (goal.position[2] - start.position[2]),
                    ];

                    waypoints.push(RobotPose {
                        position: pos,
                        orientation: start.orientation,
                        velocity: [0.5, 0.0, 0.0],
                        angular_velocity: [0.0, 0.0, 0.0],
                    });
                }

                let path = Path {
                    waypoints: waypoints.clone(),
                    total_distance: 5.0,
                    estimated_time: 10.0,
                };

                // Send validated path back to goal planning
                let _ = self.goal_path_tx.send(path.clone()).await;

                // Also send directly to behavior for immediate avoidance
                let _ = self.behavior_tx.send(path).await;
            }
        }
    }
}
