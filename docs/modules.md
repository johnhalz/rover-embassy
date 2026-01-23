# Modules Reference

This page provides detailed documentation for each module in the Rover Embassy Control System.

## Input Layer

### Sensor Array

**Location**: `src/input/sensor_array.rs`

Generates simulated sensor readings for the rover system.

**Inputs**: None (generates data internally)

**Outputs**:
- `sensor_data_tx`: Sensor data to Input Manager
- `sensor_data_safety_tx`: Sensor data to Safety Controller

**Data Generated**:
- Distance sensor readings (multiple sensors)
- IMU data (acceleration, gyroscope, orientation)
- GPS data (latitude, longitude, altitude)
- Battery level

**Logging**: Logs sensor readings at INFO level

---

### Direct User Input

**Location**: `src/input/direct_user_input.rs`

Captures manual control commands from keyboard input.

**Inputs**: Keyboard events (via crossterm)

**Outputs**:
- `user_command_tx`: User commands to Input Manager

**Commands Supported**:
- `ManualControl::MoveForward(speed)`
- `ManualControl::MoveBackward(speed)`
- `ManualControl::TurnLeft(angular_velocity)`
- `ManualControl::TurnRight(angular_velocity)`
- `ManualControl::Stop`

**Logging**: Logs user commands at INFO level

---

### User Instructions

**Location**: `src/input/user_instructions.rs`

Handles high-level mission commands and processes feedback from the Communication Module.

**Inputs**:
- `comm_user_rx`: Commands from Communication Module

**Outputs**:
- `user_command_tx`: Mission commands to Input Manager

**Mission Types**:
- `GoToWaypoint { lat, lon }`
- `FollowPath(waypoints)`
- `Patrol { waypoints, loops }`
- `ReturnHome`

**Logging**: Logs mission commands at INFO level

---

### Hardware Interface

**Location**: `src/output/hardware_interface.rs`

Manages hardware status and executes motor commands. Currently simulates hardware behavior.

**Inputs**:
- `motor_command_hw_rx`: Motor commands from Output Manager

**Outputs**:
- `hw_status_tx`: Hardware status to Input Manager

**Status Information**:
- Battery voltage
- Motor temperatures
- Health status (Healthy, Warning, Critical)

**Logging**: Logs hardware status and motor commands at INFO level

---

## Core Processing Layer

### Input Manager

**Location**: `src/input/input_manager.rs`

Central hub that aggregates all input sources and routes data to appropriate processing modules.

**Inputs**:
- `sensor_data_rx`: Sensor data from Sensor Array
- `user_command_rx`: Commands from Direct User Input and User Instructions
- `hw_status_rx`: Hardware status from Hardware Interface

**Outputs**:
- `im_env_tx`: Sensor data to Environment Understanding
- `im_state_sensor_tx`: Sensor data to State Manager
- `im_state_cmd_tx`: Commands to State Manager
- `im_task_tx`: Mission commands to Task/Mission Manager

**Responsibilities**:
- Aggregates inputs from multiple sources
- Routes data to appropriate modules
- Ensures data consistency

**Logging**: Logs routing decisions at DEBUG level

---

### Logger

**Location**: `src/infra/logger.rs`

System-wide logging infrastructure that writes to MCAP files.

**Inputs**:
- `log_rx`: Log entries from all modules

**Outputs**: MCAP file (written to disk)

**Features**:
- Writes to MCAP file format
- Uses Foxglove Log schema with FlatBuffer encoding
- Creates timestamped log files: `rover_logs_<timestamp>.mcap`
- Properly indexes files on graceful shutdown

**Log Levels**:
- `DEBUG`: Detailed diagnostic information
- `INFO`: General informational messages
- `WARN`: Warning messages
- `ERROR`: Error messages

**Logging**: Logs file operations at INFO level

!!! important "MCAP File Finalization"
    The logger must receive a shutdown signal to properly finalize the MCAP file. Always press 'q' to quit.

---

### Model/Calibration Storage

**Location**: `src/perception/model_calibration_storage.rs`

Stores robot configuration and calibration data.

**Inputs**:
- `calib_req_rx`: Calibration requests

**Outputs**:
- `calib_resp_tx`: Calibration data responses

**Stored Data**:
- Wheel diameter and wheel base
- Maximum speed and angular velocity
- Sensor offsets

**Logging**: Logs calibration requests at DEBUG level

---

### Environment Understanding

**Location**: `src/perception/environment_understanding.rs`

Builds a world model from sensor data, identifying obstacles and terrain.

**Inputs**:
- `env_rx`: Sensor data from Input Manager

**Outputs**:
- `env_state_tx`: Environment state to Obstacle Avoidance

**Capabilities**:
- Obstacle detection and classification
- Terrain type identification
- Confidence scoring

**Logging**: Logs environment updates at INFO level

---

### State Manager

**Location**: `src/planning/state_manager.rs`

Tracks the robot's internal state and manages state transitions.

**Inputs**:
- `state_sensor_rx`: Sensor data from Input Manager
- `state_cmd_rx`: Commands from Input Manager

**Outputs**:
- `state_tx`: State updates (currently unused)
- `state_safety_tx`: State to Safety Controller
- `state_task_tx`: State to Task/Mission Manager

**States**:
- `Idle`: No active mission
- `ManualControl`: Manual control mode
- `ExecutingMission`: Mission in progress
- `Paused`: System paused
- `EmergencyStop`: Emergency stop activated
- `Error(String)`: Error state

**Logging**: Logs state transitions at INFO level

---

### Stance

**Location**: `src/perception/stance.rs`

Manages robot posture and balance, adjusting stance based on terrain and obstacles.

**Inputs**:
- `stance_obstacle_req_rx`: Stance queries from Obstacle Avoidance
- `stance_goal_req_rx`: Stance queries from Goal Planning

**Outputs**:
- `stance_obstacle_resp_tx`: Stance responses to Obstacle Avoidance
- `stance_goal_resp_tx`: Stance responses to Goal Planning
- `stance_behavior_tx`: Stance configuration to Behavior

**Stance Types**:
- `Normal`: Standard operating stance
- `LowProfile`: Lowered for obstacles
- `HighClearance`: Raised for rough terrain
- `TiltCompensation(angle)`: Compensates for tilt

**Logging**: Logs stance changes at INFO level

---

### Task/Mission Manager

**Location**: `src/planning/task_mission_manager.rs`

Handles mission queue and task execution.

**Inputs**:
- `task_cmd_rx`: Mission commands from Input Manager
- `state_task_rx`: State updates from State Manager

**Outputs**:
- `goal_tx`: Goals to Goal Planning

**Capabilities**:
- Mission queue management
- Task execution tracking
- Priority handling

**Logging**: Logs mission and task updates at INFO level

---

### Goal Planning

**Location**: `src/planning/goal_planning.rs`

Plans high-level paths to achieve goals.

**Inputs**:
- `goal_rx`: Goals from Task/Mission Manager
- `stance_goal_resp_rx`: Stance responses from Stance
- `obstacle_goal_resp_rx`: Obstacle responses from Obstacle Avoidance

**Outputs**:
- `stance_goal_req_tx`: Stance queries to Stance
- `goal_obstacle_req_tx`: Path queries to Obstacle Avoidance
- `behavior_path_goal_tx`: Planned paths to Behavior

**Planning Features**:
- Path generation
- Waypoint planning
- Trajectory optimization

**Logging**: Logs planning decisions at INFO level

---

### Obstacle Avoidance

**Location**: `src/perception/obstacle_avoidance.rs`

Real-time collision avoidance system.

**Inputs**:
- `env_state_rx`: Environment state from Environment Understanding
- `stance_obstacle_resp_rx`: Stance responses from Stance
- `goal_obstacle_req_rx`: Path queries from Goal Planning

**Outputs**:
- `stance_obstacle_req_tx`: Stance queries to Stance
- `obstacle_goal_resp_tx`: Path responses to Goal Planning
- `behavior_path_obstacle_tx`: Avoidance paths to Behavior

**Capabilities**:
- Real-time obstacle detection
- Path adjustment
- Collision prevention

**Logging**: Logs obstacle avoidance actions at INFO level

---

## Behavior & Safety Layer

### Behaviour

**Location**: `src/control/behaviour.rs`

Translates high-level plans into executable actions.

**Inputs**:
- `behavior_path_goal_rx`: Planned paths from Goal Planning
- `behavior_path_obstacle_rx`: Avoidance paths from Obstacle Avoidance
- `stance_behavior_rx`: Stance configuration from Stance

**Outputs**:
- `behavior_tx`: Behavior commands to Safety Controller

**Behaviors**:
- `Idle`: No action
- `MoveTowards { target, speed }`: Move toward target
- `AvoidObstacle { direction }`: Avoid obstacle in direction
- `AdjustStance(config)`: Adjust robot stance
- `EmergencyStop`: Emergency stop

**Logging**: Logs behavior commands at INFO level

---

### Safety Controller

**Location**: `src/control/safety_controller.rs`

Final safety validation layer before hardware commands.

**Inputs**:
- `behavior_rx`: Behavior commands from Behaviour
- `sensor_data_safety_rx`: Sensor data from Sensor Array
- `state_safety_rx`: State from State Manager

**Outputs**:
- `motor_command_tx`: Validated motor commands to Output Manager

**Safety Checks**:
- Validates behavior commands against sensor data
- Checks robot state for safety
- Can override commands in emergency situations
- Validates motor command ranges

**Logging**: Logs safety checks and overrides at WARN/ERROR level

---

## Output Layer

### Output Manager

**Location**: `src/output/output_manager.rs`

Routes commands to appropriate outputs.

**Inputs**:
- `motor_command_rx`: Motor commands from Safety Controller

**Outputs**:
- `motor_command_hw_tx`: Motor commands to Hardware Interface
- `status_feedback_tx`: Status to User Feedback
- `status_comm_tx`: Status to Communication Module

**Responsibilities**:
- Routes motor commands to hardware
- Distributes status updates
- Manages output priorities

**Logging**: Logs routing decisions at DEBUG level

---

### User Feedback

**Location**: `src/output/user_feedback.rs`

Displays status information to the user.

**Inputs**:
- `status_feedback_rx`: Status updates from Output Manager

**Outputs**:
- `user_feedback_tx`: Feedback messages to Communication Module

**Feedback Types**:
- `Status`: General status updates
- `Warning`: Warning messages
- `Error`: Error messages
- `Success`: Success confirmations

**Logging**: Logs feedback generation at INFO level

---

### Communication Module

**Location**: `src/output/communication_module.rs`

Handles bidirectional communication with external systems.

**Inputs**:
- `status_comm_rx`: Status updates from Output Manager
- `user_feedback_rx`: Feedback from User Feedback

**Outputs**:
- `comm_user_tx`: Commands to User Instructions

**Capabilities**:
- Status reporting
- Command reception
- Bidirectional communication

**Logging**: Logs communication events at INFO level

---

## Module Communication Summary

| Module | Input Channels | Output Channels | Bidirectional |
|--------|---------------|-----------------|---------------|
| Sensor Array | 0 | 2 | No |
| Direct User Input | 0 | 1 | No |
| User Instructions | 1 | 1 | No |
| Hardware Interface | 1 | 1 | No |
| Input Manager | 3 | 4 | No |
| Logger | 1 | 0 | No |
| Model/Calibration Storage | 1 | 1 | No |
| Environment Understanding | 1 | 1 | No |
| State Manager | 2 | 3 | No |
| Stance | 2 | 3 | Yes (request/response) |
| Task/Mission Manager | 2 | 1 | No |
| Goal Planning | 3 | 3 | Yes (request/response) |
| Obstacle Avoidance | 3 | 3 | Yes (request/response) |
| Behaviour | 3 | 1 | No |
| Safety Controller | 3 | 1 | No |
| Output Manager | 1 | 3 | No |
| User Feedback | 1 | 1 | No |
| Communication Module | 2 | 1 | No |
