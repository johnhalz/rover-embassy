# Getting Started

This guide will help you get up and running with the Rover Embassy Control System.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (latest stable version) - [Install Rust](https://www.rust-lang.org/tools/install)
- **Cargo** (comes with Rust)
- **Git** (for cloning the repository)

## Installation

### 1. Clone the Repository

```bash
git clone <repository-url>
cd rover-embassy
```

### 2. Build the Project

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Release build (optimized, recommended for running)
cargo build --release
```

### 3. Run the System

```bash
# Run in release mode (recommended)
cargo run --release
```

You should see output like:

```
Rover Embassy Control System
Version: 0.1.0
Initializing all modules...
✓ All 20 modules initialized and running!
→ Press 'q' to shutdown
```

## First Run

When you first run the system:

1. **All modules start** - You'll see initialization messages from each module
2. **Sensor data flows** - The system begins generating simulated sensor readings
3. **Logging begins** - An MCAP file is created in the project root: `rover_logs_<timestamp>.mcap`
4. **System is ready** - The rover is ready to receive commands

## Shutting Down

!!! important "Critical: Always Use 'q' to Quit"
    To create properly indexed MCAP files, you **MUST press 'q' to quit**. Do not use Ctrl+C or kill the process.

When you press 'q':
- The shutdown signal is sent to all modules
- The logger finalizes the MCAP file with proper indexing
- All modules clean up gracefully
- You'll see: `✓ All modules stopped. Goodbye!`

## Viewing Logs

The system logs all events to MCAP files that can be visualized in [Foxglove Studio](https://foxglove.dev/download).

### 1. Download Foxglove Studio

Get the free, open-source [Foxglove Studio](https://foxglove.dev/download) for your platform.

### 2. Open Your Log File

1. Launch Foxglove Studio
2. Click "Open local file"
3. Select your `rover_logs_*.mcap` file from the project root

### 3. Explore the Logs

- **Log Panel**: View structured log messages from all modules
- **Raw Messages**: See the JSON data structure
- **Timeline**: Scrub through logs to see system behavior over time
- **Filtering**: Filter by log level or module name

## Testing

### Run the Test Example

To generate a test MCAP file that's guaranteed to be indexed:

```bash
cargo run --example test_mcap
```

This creates `test_indexed.mcap` which you can open in Foxglove to verify proper indexing.

## Project Structure

```
rover-embassy/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # System initialization
│   ├── types.rs             # Shared data structures
│   ├── input/               # Input layer modules
│   ├── perception/          # Perception modules
│   ├── planning/            # Planning modules
│   ├── control/             # Control modules
│   ├── output/              # Output layer modules
│   └── infra/               # Infrastructure (logging)
├── docs/                    # Documentation
├── schemas/                 # FlatBuffer schemas
└── Cargo.toml              # Rust project configuration
```

## Next Steps

- Read the [Architecture](architecture.md) documentation to understand the system design
- Explore the [Modules Reference](modules.md) for detailed module documentation
- Learn about [MCAP Indexing](MCAP_INDEXING.md) for proper log file handling

## Troubleshooting

### Build Errors

If you encounter build errors:

```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### MCAP File Issues

If your MCAP file shows as "unindexed" in Foxglove:

- Make sure you pressed 'q' to quit (not Ctrl+C)
- Check that you see the "MCAP file finalized successfully" message
- Verify file size is > 1KB (unindexed files are ~92 bytes)
- See [MCAP Indexing](MCAP_INDEXING.md) for detailed troubleshooting

### Module Not Starting

If a module fails to start:

- Check the terminal output for error messages
- Ensure all dependencies are installed
- Verify you're using a recent Rust version

## Getting Help

- Check the [Architecture](architecture.md) page for system design details
- Review the [Modules Reference](modules.md) for module-specific information
- See [MCAP Indexing](MCAP_INDEXING.md) for logging and file format details
