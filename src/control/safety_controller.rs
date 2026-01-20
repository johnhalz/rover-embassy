use crate::types::{BehaviorCommand, Behavior, SensorData, RobotState, MotorCommand, LogEntry, LogLevel};
use crate::infra::logger::create_log;
use tokio::sync::{broadcast, mpsc};

pub struct SafetyController {
    behavior_rx: mpsc::Receiver<BehaviorCommand>,
    sensor_rx: mpsc::Receiver<SensorData>,
    state_rx: mpsc::Receiver<RobotState>,
    motor_tx: mpsc::Sender<MotorCommand>,
    log_tx: mpsc::Sender<LogEntry>,
    shutdown_rx: broadcast::Receiver<()>,
    emergency_stop: bool,
}

impl SafetyController {
    pub fn new(
        behavior_rx: mpsc::Receiver<BehaviorCommand>,
        sensor_rx: mpsc::Receiver<SensorData>,
        state_rx: mpsc::Receiver<RobotState>,
        motor_tx: mpsc::Sender<MotorCommand>,
        log_tx: mpsc::Sender<LogEntry>,
        shutdown_rx: broadcast::Receiver<()>,
    ) -> Self {
        Self {
            behavior_rx,
            sensor_rx,
            state_rx,
            motor_tx,
            log_tx,
            shutdown_rx,
            emergency_stop: false,
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
        if self.emergency_stop {
            let _ = self.log_tx.send(create_log(
                "SafetyController",
                LogLevel::Warn,
                "Command blocked - emergency stop active".to_string()
            )).await;
            return;
        }

        match cmd.behavior {
            Behavior::MoveTowards { target, speed } => {
                let motor_cmd = self.calculate_motor_command(target, speed);
                let _ = self.motor_tx.send(motor_cmd).await;
            }
            Behavior::AvoidObstacle { direction } => {
                let _ = self.log_tx.send(create_log(
                    "SafetyController",
                    LogLevel::Info,
                    "Executing obstacle avoidance maneuver".to_string()
                )).await;

                let motor_cmd = MotorCommand {
                    left_speed: direction[1] * 0.5,
                    right_speed: -direction[1] * 0.5,
                };
                let _ = self.motor_tx.send(motor_cmd).await;
            }
            Behavior::EmergencyStop => {
                self.send_stop_command().await;
            }
            _ => {}
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
        let stop_cmd = MotorCommand {
            left_speed: 0.0,
            right_speed: 0.0,
        };
        let _ = self.motor_tx.send(stop_cmd).await;
    }

    fn calculate_motor_command(&self, target: [f32; 3], speed: f32) -> MotorCommand {
        // Simplified differential drive calculation
        let angle_to_target = target[1].atan2(target[0]);
        let turn_factor = angle_to_target.sin();

        MotorCommand {
            left_speed: speed * (1.0 - turn_factor * 0.5),
            right_speed: speed * (1.0 + turn_factor * 0.5),
        }
    }
}
