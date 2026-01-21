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
# Important: Always use 'q' to quit for proper MCAP file finalization and indexing
```

**CRITICAL**: To create properly indexed MCAP files, you **MUST press 'q' to quit**. Do not use Ctrl+C or kill the process.

When you press 'q':
- The shutdown signal is sent to all modules
- The logger calls `writer.finish()` which writes:
  - Summary section with channel and schema info
  - Chunk indices for fast message lookup
  - Statistics record
  - Footer with summary offsets
- The file will be properly indexed (3-5KB for typical runs)

If you kill the process forcefully:
- The file will be ~92 bytes (just header and footer)
- Foxglove will show "This file is unindexed" warning
- The summary section will be missing
- Message data may be incomplete

**Testing**: Run `cargo run --example test_mcap` to generate a test file that is guaranteed to be indexed.

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
- **MCAP Logging**: Structured logging to MCAP files for visualization in Foxglove

## Dependencies

- `tokio` - Async runtime
- `crossterm` - Terminal input handling
- `serde` - Serialization for data structures
- `mcap` - MCAP file format for logging
- `serde_json` - JSON serialization for log messages

## Logging with MCAP and Foxglove

The system logs all events to an MCAP file (`.mcap`), which can be opened and visualized with [Foxglove](https://foxglove.dev/).

### Log Files

When you run the rover system, it creates a timestamped MCAP file in the project root:
```
rover_logs_1737418560.mcap
```

The file contains module-specific topics with the structure `roverOS/<module>` (e.g., `roverOS/InputManager`, `roverOS/SensorArray`) using the Foxglove Log schema with FlatBuffer encoding:
- Timestamp (seconds and nanoseconds since epoch)
- Log level (DEBUG, INFO, WARNING, ERROR)
- Module name (in the `name` field)
- Log message
- Optional file and line number fields

### Viewing Logs in Foxglove

1. **Download Foxglove**: Get [Foxglove Studio](https://foxglove.dev/download) (free and open source)

2. **Open your log file**:
   - Launch Foxglove Studio
   - Click "Open local file"
   - Select your `rover_logs_*.mcap` file

3. **View the logs**:
   - Add a "Log" panel to view structured log messages
   - Add a "Raw Messages" panel to see the JSON data
   - Use the timeline to scrub through logs
   - Filter by log level or module name

### Benefits of MCAP Logging

- **Structured data**: Logs are structured using FlatBuffers, making them searchable and filterable
- **Time synchronization**: All logs are timestamped for correlation
- **Efficient storage**: MCAP files are compressed and efficient
- **Native Foxglove support**: Uses the official Foxglove Log schema for optimal compatibility
- **Rich visualization**: Foxglove provides powerful tools for log analysis with built-in Log panel support
- **Post-processing**: Easy to extract and analyze log data programmatically

### Schema Compilation (For Developers)

The logger uses pre-compiled FlatBuffer schemas located in `schemas/`. If you need to recompile them:

```bash
# Install flatbuffers compiler if not already installed
# brew install flatbuffers (on macOS)

# Compile to binary schema format
flatc --binary --schema -o schemas/ schemas/Log.fbs

# Compile to Rust code (already done, stored in src/foxglove/)
flatc --rust -o src/foxglove schemas/Log.fbs schemas/Time.fbs
```

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
