use crate::types::{BehaviorCommand, Behavior, SensorData, RobotState, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct SafetyController {
    behavior_rx: mpsc::Receiver<BehaviorCommand>,
    sensor_rx: mpsc::Receiver<SensorData>,
    state_rx: mpsc::Receiver<RobotState>,
    hardware_interface_tx: mpsc::Sender<BehaviorCommand>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    emergency_stop: bool,
    latest_sensor_data: Option<SensorData>,
}

impl SafetyController {
    pub fn new(
        behavior_rx: mpsc::Receiver<BehaviorCommand>,
        sensor_rx: mpsc::Receiver<SensorData>,
        state_rx: mpsc::Receiver<RobotState>,
        hardware_interface_tx: mpsc::Sender<BehaviorCommand>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            behavior_rx,
            sensor_rx,
            state_rx,
            hardware_interface_tx,
            log_tx,
            shutdown_rx,
            emergency_stop: false,
            latest_sensor_data: None,
        }
    }

    pub async fn run(mut self) {
        let _ = self.log_tx.send(create_log(
            "SafetyController",
            LogLevel::Info,
            "Starting safety controller".to_string()
        )).await;

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    let _ = self.log_tx.send(create_log(
                        "SafetyController",
                        LogLevel::Info,
                        "Shutdown signal received".to_string()
                    )).await;
                    break;
                }
                Some(behavior_cmd) = self.behavior_rx.recv() => {
                    self.validate_and_execute(behavior_cmd).await;
                }
                Some(sensor_data) = self.sensor_rx.recv() => {
                    self.latest_sensor_data = Some(sensor_data.clone());
                    self.check_safety(&sensor_data).await;
                }
                Some(state) = self.state_rx.recv() => {
                    if matches!(state, RobotState::EmergencyStop) {
                        self.emergency_stop = true;
                        let _ = self.log_tx.send(create_log(
                            "SafetyController",
                            LogLevel::Error,
                            "EMERGENCY STOP ACTIVATED".to_string()
                        )).await;
                        self.send_stop_command().await;
                    }
                }
            }
        }

        let _ = self.log_tx.send(create_log(
            "SafetyController",
            LogLevel::Info,
            "Stopped".to_string()
        )).await;
    }

    async fn validate_and_execute(&mut self, cmd: BehaviorCommand) {
        // Check emergency stop
        if self.emergency_stop {
            let _ = self.log_tx.send(create_log(
                "SafetyController",
                LogLevel::Warn,
                "Command blocked - emergency stop active".to_string()
            )).await;
            return;
        }

        // Validate against sensor data
        if let Some(ref sensor_data) = self.latest_sensor_data {
            // Check for critical battery level
            if sensor_data.battery_level < 0.1 {
                let _ = self.log_tx.send(create_log(
                    "SafetyController",
                    LogLevel::Error,
                    format!("Command blocked - critical battery level: {:.1}%", sensor_data.battery_level * 100.0)
                )).await;
                return;
            }

            // Check for immediate obstacles in front
            if let Behavior::MoveTowards { .. } = cmd.behavior {
                if let Some(&front_distance) = sensor_data.distance_sensors.get(0) {
                    if front_distance < 0.5 {
                        let _ = self.log_tx.send(create_log(
                            "SafetyController",
                            LogLevel::Warn,
                            format!("Command blocked - obstacle too close: {:.2}m", front_distance)
                        )).await;
                        return;
                    }
                }
            }
        }

        // Command is safe, forward to Hardware Interface
        if let Err(_) = self.hardware_interface_tx.send(cmd).await {
            let _ = self.log_tx.send(create_log(
                "SafetyController",
                LogLevel::Error,
                "Failed to send validated command to hardware interface".to_string()
            )).await;
        } else {
            let _ = self.log_tx.send(create_log(
                "SafetyController",
                LogLevel::Debug,
                "Command validated and forwarded to hardware interface".to_string()
            )).await;
        }
    }

    async fn check_safety(&mut self, sensor_data: &SensorData) {
        // Check for critical battery level
        if sensor_data.battery_level < 0.1 {
            let _ = self.log_tx.send(create_log(
                "SafetyController",
                LogLevel::Error,
                format!("Critical battery level: {:.1}%", sensor_data.battery_level * 100.0)
            )).await;
        }

        // Check for immediate obstacles
        for (i, &distance) in sensor_data.distance_sensors.iter().enumerate() {
            if distance < 0.3 {
                let _ = self.log_tx.send(create_log(
                    "SafetyController",
                    LogLevel::Warn,
                    format!("Close obstacle on sensor {}: {:.2}m", i, distance)
                )).await;
            }
        }
    }

    async fn send_stop_command(&mut self) {
        let stop_cmd = BehaviorCommand {
            timestamp: std::time::SystemTime::now(),
            behavior: Behavior::EmergencyStop,
            priority: 10, // Highest priority for emergency stop
        };
        let _ = self.hardware_interface_tx.send(stop_cmd).await;
    }
}
