use crate::types::{SensorData, UserCommand, HardwareStatus, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct InputManager {
    // Inputs
    sensor_rx: mpsc::Receiver<SensorData>,
    user_cmd_rx: mpsc::Receiver<UserCommand>,
    hw_status_rx: mpsc::Receiver<HardwareStatus>,

    // Outputs
    env_understanding_tx: mpsc::Sender<SensorData>,
    state_manager_sensor_tx: mpsc::Sender<SensorData>,
    state_manager_cmd_tx: mpsc::Sender<UserCommand>,
    task_manager_tx: mpsc::Sender<UserCommand>,
    log_tx: mpsc::Sender<LogEntry>,

    shutdown_rx: broadcast::Receiver<()>,
}

impl InputManager {
    pub fn new(
        sensor_rx: mpsc::Receiver<SensorData>,
        user_cmd_rx: mpsc::Receiver<UserCommand>,
        hw_status_rx: mpsc::Receiver<HardwareStatus>,
        env_understanding_tx: mpsc::Sender<SensorData>,
        state_manager_sensor_tx: mpsc::Sender<SensorData>,
        state_manager_cmd_tx: mpsc::Sender<UserCommand>,
        task_manager_tx: mpsc::Sender<UserCommand>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            sensor_rx,
            user_cmd_rx,
            hw_status_rx,
            env_understanding_tx,
            state_manager_sensor_tx,
            state_manager_cmd_tx,
            task_manager_tx,
            log_tx,
            shutdown_rx,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "InputManager",
            LogLevel::Info,
            "Starting input manager".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "InputManager",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(sensor_data) = self.sensor_rx.recv() => {
                    self.handle_sensor_data(sensor_data).await;
                }
                Some(user_cmd) = self.user_cmd_rx.recv() => {
                    self.handle_user_command(user_cmd).await;
                }
                Some(hw_status) = self.hw_status_rx.recv() => {
                    self.handle_hardware_status(hw_status).await;
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "InputManager",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn handle_sensor_data(&mut self, sensor_data: SensorData) {
        // Route sensor data to environment understanding and state manager
        let _ = self.env_understanding_tx.send(sensor_data.clone()).await;
        let _ = self.state_manager_sensor_tx.send(sensor_data).await;
    }

    async fn handle_user_command(&mut self, command: UserCommand) {
        let _ = self.log_tx.send(create_log(
            "InputManager",
            LogLevel::Info,
            format!("Routing user command: {:?}", command)
        )).await;

        // Route commands to state manager and task manager
        let _ = self.state_manager_cmd_tx.send(command.clone()).await;
        let _ = self.task_manager_tx.send(command).await;
    }

    async fn handle_hardware_status(&mut self, status: HardwareStatus) {
        match &status.health {
            crate::types::HealthStatus::Warning(msg) => {
                let _ = self.log_tx.send(create_log(
                    "InputManager",
                    LogLevel::Warn,
                    format!("Hardware warning: {}", msg)
                )).await;
            }
            crate::types::HealthStatus::Critical(msg) => {
                let _ = self.log_tx.send(create_log(
                    "InputManager",
                    LogLevel::Error,
                    format!("Hardware critical: {}", msg)
                )).await;
            }
            _ => {}
        }
    }
}
