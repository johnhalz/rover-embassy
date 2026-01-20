use crate::types::{Mission, Task, TaskType, TaskStatus, UserCommand, MissionCommand, RobotState, Goal, GoalType, RobotPose, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct TaskMissionManager {
    command_rx: mpsc::Receiver<UserCommand>,
    state_rx: mpsc::Receiver<RobotState>,
    goal_tx: mpsc::Sender<Goal>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    current_mission: Option<Mission>,
    mission_counter: u64,
}

impl TaskMissionManager {
    pub fn new(
        command_rx: mpsc::Receiver<UserCommand>,
        state_rx: mpsc::Receiver<RobotState>,
        goal_tx: mpsc::Sender<Goal>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            command_rx,
            state_rx,
            goal_tx,
            log_tx,
            shutdown_rx,
            current_mission: None,
            mission_counter: 0,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "TaskMissionManager",
            LogLevel::Info,
            "Starting task/mission manager".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "TaskMissionManager",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(command) = self.command_rx.recv() => {
                    self.handle_command(command).await;
                }
                Some(_state) = self.state_rx.recv() => {
                    // Update based on state changes
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "TaskMissionManager",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn handle_command(&mut self, command: UserCommand) {
        match command {
            UserCommand::MissionCommand(mission_cmd) => {
                let mission = self.create_mission_from_command(mission_cmd);

                let _ = self.log_tx.send(create_log(
                    "TaskMissionManager",
                    LogLevel::Info,
                    format!("New mission: {} with {} tasks", mission.name, mission.tasks.len())
                )).await;

                self.current_mission = Some(mission.clone());
                self.execute_mission(mission).await;
            }
            _ => {}
        }
    }

    fn create_mission_from_command(&mut self, cmd: MissionCommand) -> Mission {
        self.mission_counter += 1;

        let (name, tasks) = match cmd {
            MissionCommand::GoToWaypoint { lat, lon } => {
                (
                    format!("GoTo({:.4}, {:.4})", lat, lon),
                    vec![Task {
                        id: 1,
                        description: "Navigate to waypoint".to_string(),
                        task_type: TaskType::Navigate(crate::types::Waypoint { lat, lon, tolerance: 2.0 }),
                        status: TaskStatus::Pending,
                    }]
                )
            }
            MissionCommand::Patrol { waypoints, loops } => {
                let tasks: Vec<Task> = waypoints.iter().enumerate().map(|(i, wp)| {
                    Task {
                        id: i as u64 + 1,
                        description: format!("Waypoint {}", i + 1),
                        task_type: TaskType::Navigate(wp.clone()),
                        status: TaskStatus::Pending,
                    }
                }).collect();

                (format!("Patrol {} waypoints x{} loops", waypoints.len(), loops), tasks)
            }
            MissionCommand::FollowPath(waypoints) => {
                let tasks: Vec<Task> = waypoints.iter().enumerate().map(|(i, wp)| {
                    Task {
                        id: i as u64 + 1,
                        description: format!("Path point {}", i + 1),
                        task_type: TaskType::Navigate(wp.clone()),
                        status: TaskStatus::Pending,
                    }
                }).collect();

                (format!("Follow path with {} points", waypoints.len()), tasks)
            }
            MissionCommand::ReturnHome => {
                (
                    "Return Home".to_string(),
                    vec![Task {
                        id: 1,
                        description: "Navigate home".to_string(),
                        task_type: TaskType::Navigate(crate::types::Waypoint {
                            lat: 37.7749,
                            lon: -122.4194,
                            tolerance: 1.0
                        }),
                        status: TaskStatus::Pending,
                    }]
                )
            }
        };

        Mission {
            id: self.mission_counter,
            name,
            tasks,
            priority: 5,
        }
    }

    async fn execute_mission(&mut self, mission: Mission) {
        for task in &mission.tasks {
            let goal = self.task_to_goal(task);
            let _ = self.goal_tx.send(goal).await;
        }
    }

    fn task_to_goal(&self, task: &Task) -> Goal {
        match &task.task_type {
            TaskType::Navigate(waypoint) => {
                Goal {
                    target_pose: RobotPose {
                        position: [waypoint.lat as f32, waypoint.lon as f32, 0.0],
                        orientation: [1.0, 0.0, 0.0, 0.0],
                        velocity: [0.0, 0.0, 0.0],
                        angular_velocity: [0.0, 0.0, 0.0],
                    },
                    goal_type: GoalType::ReachPosition,
                }
            }
            _ => {
                Goal {
                    target_pose: RobotPose {
                        position: [0.0, 0.0, 0.0],
                        orientation: [1.0, 0.0, 0.0, 0.0],
                        velocity: [0.0, 0.0, 0.0],
                        angular_velocity: [0.0, 0.0, 0.0],
                    },
                    goal_type: GoalType::ReachPosition,
                }
            }
        }
    }
}
