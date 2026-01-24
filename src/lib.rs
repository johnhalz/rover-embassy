// Module declarations
pub mod types;
pub mod infra;
pub mod input;
pub mod perception;
pub mod planning;
pub mod control;
pub mod output;

use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use crossterm::style::Stylize;

pub struct RoverSystem {
    shutdown_tx: broadcast::Sender<()>,
    task_handles: Vec<JoinHandle<()>>,
}

impl RoverSystem {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);

        Self {
            shutdown_tx,
            task_handles: Vec::new(),
        }
    }

    pub async fn initialize_and_run(&mut self) {
        let version = env!("CARGO_PKG_VERSION");
        println!("{}", "Rover Embassy Control System".cyan().bold());
        println!("{} {}", "Version:".cyan(), version.cyan().bold());
        println!("{}\n", "Initializing all modules...".yellow());

        // Create all channels
        let (log_tx, log_rx) = mpsc::channel(256);

        // Input layer channels
        // Sensor Array -> Hardware Interface -> Input Manager
        let (sensor_data_hw_tx, sensor_data_hw_rx) = mpsc::channel(32);
        let (sensor_data_im_tx, sensor_data_rx) = mpsc::channel(32);
        let (sensor_data_safety_tx, sensor_data_safety_rx) = mpsc::channel(32);
        let (user_command_tx, user_command_rx) = mpsc::channel(32);
        let (hw_status_tx, hw_status_rx) = mpsc::channel(32);
        let (motor_command_hw_tx, motor_command_hw_rx) = mpsc::channel(32);

        // Input manager outputs
        let (im_env_tx, env_rx) = mpsc::channel(32);
        let (im_state_sensor_tx, state_sensor_rx) = mpsc::channel(32);
        let (im_state_cmd_tx, state_cmd_rx) = mpsc::channel(32);
        let (im_task_tx, task_cmd_rx) = mpsc::channel(32);

        // State manager outputs
        let (state_tx, _state_rx) = mpsc::channel(32);
        let (state_safety_tx, state_safety_rx) = mpsc::channel(32);
        let (state_task_tx, state_task_rx) = mpsc::channel(32);

        // Environment understanding
        let (env_state_tx, env_state_rx) = mpsc::channel(32);

        // Task/Mission manager
        let (goal_tx, goal_rx) = mpsc::channel(32);

        // Stance bidirectional channels
        let (stance_obstacle_req_tx, stance_obstacle_req_rx) = mpsc::channel(32);
        let (stance_obstacle_resp_tx, stance_obstacle_resp_rx) = mpsc::channel(32);
        let (stance_goal_req_tx, stance_goal_req_rx) = mpsc::channel(32);
        let (stance_goal_resp_tx, stance_goal_resp_rx) = mpsc::channel(32);
        let (stance_behavior_tx, stance_behavior_rx) = mpsc::channel(32);

        // Goal planning and obstacle avoidance bidirectional
        let (goal_obstacle_req_tx, goal_obstacle_req_rx) = mpsc::channel(32);
        let (obstacle_goal_resp_tx, obstacle_goal_resp_rx) = mpsc::channel(32);

        // Behavior outputs
        let (behavior_path_goal_tx, behavior_path_goal_rx) = mpsc::channel(32);
        let (behavior_path_obstacle_tx, behavior_path_obstacle_rx) = mpsc::channel(32);

        // Behaviour -> Safety Controller -> Hardware Interface
        let (behavior_safety_tx, behavior_safety_rx) = mpsc::channel(32);
        let (behavior_hw_tx, behavior_hw_rx) = mpsc::channel(32);

        // Output manager
        let (status_feedback_tx, status_feedback_rx) = mpsc::channel(32);
        let (status_comm_tx, status_comm_rx) = mpsc::channel(32);

        // User feedback
        let (user_feedback_tx, user_feedback_rx) = mpsc::channel(32);

        // Communication module to user instructions
        let (comm_user_tx, comm_user_rx) = mpsc::channel(32);

        // Calibration storage (not actively used in this basic implementation)
        let (_calib_req_tx, calib_req_rx) = mpsc::channel(32);
        let (calib_resp_tx, _calib_resp_rx) = mpsc::channel(32);

        // Spawn logger first
        let logger_module = infra::logger::Logger::new(log_rx, self.shutdown_tx.subscribe());
        self.task_handles.push(tokio::spawn(logger_module.run()));

        // Spawn input layer modules
        let sensor_array = input::sensor_array::SensorArray::new(
            sensor_data_hw_tx,
            sensor_data_safety_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(sensor_array.run()));

        let direct_input = input::direct_user_input::DirectUserInput::new(
            user_command_tx.clone(),
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(direct_input.run()));

        let user_instructions = input::user_instructions::UserInstructions::new(
            user_command_tx,
            comm_user_rx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(user_instructions.run()));

        let hardware_interface = output::hardware_interface::HardwareInterface::new(
            sensor_data_hw_rx,
            behavior_hw_rx,
            motor_command_hw_rx,
            sensor_data_im_tx,
            hw_status_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(hardware_interface.run()));

        // Spawn input manager
        let input_manager = input::input_manager::InputManager::new(
            sensor_data_rx,
            user_command_rx,
            hw_status_rx,
            im_env_tx,
            im_state_sensor_tx,
            im_state_cmd_tx,
            im_task_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(input_manager.run()));

        // Spawn calibration storage
        let calibration = perception::model_calibration_storage::ModelCalibrationStorage::new(
            calib_req_rx,
            calib_resp_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(calibration.run()));

        // Spawn core processing modules
        let env_understanding = perception::environment_understanding::EnvironmentUnderstanding::new(
            env_rx,
            env_state_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(env_understanding.run()));

        let state_manager = planning::state_manager::StateManager::new(
            state_sensor_rx,
            state_cmd_rx,
            state_tx,
            state_safety_tx,
            state_task_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(state_manager.run()));

        let stance = perception::stance::Stance::new(
            stance_obstacle_req_rx,
            stance_goal_req_rx,
            stance_obstacle_resp_tx,
            stance_goal_resp_tx,
            stance_behavior_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(stance.run()));

        let task_manager = planning::task_mission_manager::TaskMissionManager::new(
            task_cmd_rx,
            state_task_rx,
            goal_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(task_manager.run()));

        let goal_planning = planning::goal_planning::GoalPlanning::new(
            goal_rx,
            stance_goal_req_tx,
            stance_goal_resp_rx,
            goal_obstacle_req_tx,
            obstacle_goal_resp_rx,
            behavior_path_goal_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(goal_planning.run()));

        let obstacle_avoidance = perception::obstacle_avoidance::ObstacleAvoidance::new(
            env_state_rx,
            stance_obstacle_req_tx,
            stance_obstacle_resp_rx,
            goal_obstacle_req_rx,
            obstacle_goal_resp_tx,
            behavior_path_obstacle_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(obstacle_avoidance.run()));

        let behaviour = control::behaviour::BehaviourModule::new(
            behavior_path_goal_rx,
            behavior_path_obstacle_rx,
            stance_behavior_rx,
            behavior_safety_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(behaviour.run()));

        // Spawn safety controller (validates commands before forwarding to Hardware Interface)
        let safety_controller = control::safety_controller::SafetyController::new(
            behavior_safety_rx,
            sensor_data_safety_rx,
            state_safety_rx,
            behavior_hw_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(safety_controller.run()));

        // Output Manager is no longer in the direct command path (Behaviour -> Hardware Interface)
        // but keeping for status updates and backward compatibility
        let (dummy_motor_cmd_tx, dummy_motor_cmd_rx) = mpsc::channel(32);
        drop(dummy_motor_cmd_tx); // Close the sender so receiver will never receive
        let output_manager = output::output_manager::OutputManager::new(
            dummy_motor_cmd_rx,
            motor_command_hw_tx,
            status_feedback_tx,
            status_comm_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(output_manager.run()));

        let user_feedback = output::user_feedback::UserFeedbackModule::new(
            status_feedback_rx,
            user_feedback_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(user_feedback.run()));

        let communication = output::communication_module::CommunicationModule::new(
            status_comm_rx,
            user_feedback_rx,
            comm_user_tx,
            log_tx.clone(),
            self.shutdown_tx.subscribe(),
        );
        self.task_handles.push(tokio::spawn(communication.run()));

        println!("{} {}", "✓".green().bold(), format!("All {} modules initialized and running!", self.task_handles.len()).green());
        println!("{} {}\n", "→".blue().bold(), "Press 'q' to shutdown".blue());
    }

    pub fn shutdown_tx(&self) -> broadcast::Sender<()> {
        self.shutdown_tx.clone()
    }

    pub async fn wait_for_completion(self) {
        // Wait for all tasks to complete
        for handle in self.task_handles {
            let _ = handle.await;
        }
    }
}
