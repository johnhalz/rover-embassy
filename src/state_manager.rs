use crate::types::{RobotState, RobotPose, SensorData, UserCommand, SystemCommand, LogEntry, LogLevel};
use crate::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct StateManager {
    sensor_rx: mpsc::Receiver<SensorData>,
    command_rx: mpsc::Receiver<UserCommand>,
    state_tx: mpsc::Sender<RobotState>,
    safety_state_tx: mpsc::Sender<RobotState>,
    task_manager_state_tx: mpsc::Sender<RobotState>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    current_state: RobotState,
    pose: RobotPose,
}

impl StateManager {
    pub fn new(
        sensor_rx: mpsc::Receiver<SensorData>,
        command_rx: mpsc::Receiver<UserCommand>,
        state_tx: mpsc::Sender<RobotState>,
        safety_state_tx: mpsc::Sender<RobotState>,
        task_manager_state_tx: mpsc::Sender<RobotState>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            sensor_rx,
            command_rx,
            state_tx,
            safety_state_tx,
            task_manager_state_tx,
            log_tx,
            shutdown_rx,
            current_state: RobotState::Idle,
            pose: RobotPose {
                position: [0.0, 0.0, 0.0],
                orientation: [1.0, 0.0, 0.0, 0.0],
                velocity: [0.0, 0.0, 0.0],
                angular_velocity: [0.0, 0.0, 0.0],
            },
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "StateManager",
            LogLevel::Info,
            "Starting state manager".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "StateManager",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(sensor_data) = self.sensor_rx.recv() => {
                    self.update_pose(&sensor_data);
                }
                Some(command) = self.command_rx.recv() => {
                    self.handle_command(command).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "StateManager",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    fn update_pose(&mut self, sensor_data: &SensorData) {
        // Update pose from sensor data
        self.pose.orientation = sensor_data.imu.orientation;
        // In a real system, we'd integrate velocity to get position
    }

    async fn handle_command(&mut self, command: UserCommand) {
        let new_state = match command {
            UserCommand::ManualControl(_) => {
                RobotState::ManualControl
            }
            UserCommand::MissionCommand(_) => {
                RobotState::ExecutingMission
            }
            UserCommand::SystemCommand(sys_cmd) => {
                match sys_cmd {
                    SystemCommand::Pause => RobotState::Paused,
                    SystemCommand::Resume => RobotState::ExecutingMission,
                    SystemCommand::EmergencyStop => RobotState::EmergencyStop,
                    SystemCommand::Calibrate => RobotState::Idle,
                }
            }
        };

        if !matches!(&self.current_state, state if std::mem::discriminant(state) == std::mem::discriminant(&new_state)) {
            let _ = self.log_tx.send(create_log(
                "StateManager",
                LogLevel::Info,
                format!("State transition: {:?} -> {:?}", self.current_state, new_state)
            )).await;

            self.current_state = new_state.clone();

            // Broadcast state to interested modules
            let _ = self.state_tx.send(new_state.clone()).await;
            let _ = self.safety_state_tx.send(new_state.clone()).await;
            let _ = self.task_manager_state_tx.send(new_state).await;
        }
    }
}
