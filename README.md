# Rover Embassy Control System

A modular, async rover control system built with Tokio, implementing a complete control architecture with 16+ modules communicating via channels.

## Architecture

The system is organized into four main layers:

### 1. Input Layer (4 modules)
- **Sensor Array**: Simulates sensor readings (distance, IMU, GPS, battery)
- **Direct User Input**: Captures manual control commands
- **User Instructions**: Handles high-level mission commands
- **Hardware Interface**: Manages hardware status and motor commands

### 2. Core Processing (9 modules)
- **Input Manager**: Central hub aggregating all input sources
- **Logger**: System-wide logging with multiple log levels
- **Model/Calibration Storage**: Stores robot configuration and calibration data
- **Environment Understanding**: Builds world model from sensor data
- **State Manager**: Tracks robot's internal state
- **Stance**: Manages robot posture and balance
- **Task/Mission Manager**: Handles mission queue and task execution
- **Goal Planning**: Plans high-level paths to achieve goals
- **Obstacle Avoidance**: Real-time collision avoidance

### 3. Behavior & Safety (2 modules)
- **Behaviour**: Translates high-level plans into executable actions
- **Safety Controller**: Final safety validation before hardware commands

### 4. Output Layer (3 modules)
- **Output Manager**: Routes commands to appropriate outputs
- **User Feedback**: Displays status information
- **Communication Module**: Handles bidirectional communication

## Data Flow

```
Sensors/User Input → Input Manager → Processing Modules → Behavior → Safety → Hardware
                                                        ↓
                                                     Logger
```

## Building and Running

```bash
# Build
cargo build --release

# Run
cargo run --release

# Press 'q' to shutdown gracefully
```

## Project Structure

```
src/
├── main.rs                      # Entry point
├── lib.rs                       # System initialization and wiring
├── types.rs                     # Shared data structures
├── logger.rs                    # Logging module
│
├── sensor_array.rs              # Input: Sensor data
├── direct_user_input.rs         # Input: Manual control
├── user_instructions.rs         # Input: Mission commands
├── hardware_interface.rs        # Input/Output: Hardware status
│
├── input_manager.rs             # Core: Input aggregation
├── model_calibration_storage.rs # Core: Configuration storage
├── environment_understanding.rs # Core: World modeling
├── state_manager.rs             # Core: State tracking
├── stance.rs                    # Core: Posture control
├── task_mission_manager.rs      # Core: Mission management
├── goal_planning.rs             # Core: Path planning
├── obstacle_avoidance.rs        # Core: Collision avoidance
│
├── behaviour.rs                 # Behavior: Action execution
├── safety_controller.rs         # Safety: Final validation
│
├── output_manager.rs            # Output: Command routing
├── user_feedback.rs             # Output: Status display
└── communication_module.rs      # Output: Communication

docs/
└── diagram.mermaid             # Original system diagram
```

## Features

- **Async/Await**: Built on Tokio for efficient concurrent processing
- **Channel-based Communication**: mpsc and broadcast channels for inter-module messaging
- **Graceful Shutdown**: All modules respond to shutdown signals
- **Modular Design**: Each component in its own file
- **Type Safety**: Strongly typed message passing
- **Logging**: Centralized logging from all modules

## Dependencies

- `tokio` - Async runtime
- `crossterm` - Terminal input handling
- `serde` - Serialization for data structures

## Implementation Status

This is a basic working implementation with:
- ✅ All 20 modules implemented
- ✅ Simulated sensor data generation
- ✅ Command flow from user input to hardware
- ✅ Bidirectional communication between modules
- ✅ Safety checks and validation
- ✅ System-wide logging

Future enhancements could include:
- Real hardware integration
- More sophisticated path planning algorithms
- Machine learning for environment understanding
- Network-based remote control
- Configuration file support
