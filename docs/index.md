# Rover Embassy Control System

Welcome to the **Rover Embassy Control System** documentation! This is a modular, async rover control system built with Rust and Tokio, implementing a complete control architecture with 20+ modules communicating via channels.

## Overview

Rover Embassy is a sophisticated control system designed for autonomous rovers. It implements a layered architecture that processes sensor data, plans missions, avoids obstacles, and executes behaviors while maintaining safety throughout the operation.

## Key Features

- **Modular Architecture**: 20+ independent modules organized into logical layers
- **Async/Await**: Built on Tokio for efficient concurrent processing
- **Channel-based Communication**: Type-safe message passing between modules
- **MCAP Logging**: Structured logging compatible with Foxglove Studio
- **Safety First**: Multi-layer safety validation before hardware commands
- **Graceful Shutdown**: Clean termination with proper resource cleanup

## Quick Start

```bash
# Build the project
cargo build --release

# Run the system
cargo run --release

# Press 'q' to shutdown gracefully
```

!!! warning "Important"
    Always press **'q'** to quit for proper MCAP file finalization and indexing. Do not use Ctrl+C or kill the process.

## Architecture Layers

The system is organized into four main layers:

1. **Input Layer** - Sensor data, user commands, and hardware status
2. **Core Processing** - Environment understanding, planning, and state management
3. **Behavior & Safety** - Action execution and safety validation
4. **Output Layer** - Hardware control and user feedback

See the [Architecture](architecture.md) page for detailed information and visual diagrams.

## Documentation Structure

- **[Getting Started](getting-started.md)** - Installation and first steps
- **[Architecture](architecture.md)** - System design and module interactions
- **[Modules Reference](modules.md)** - Detailed documentation for each module
- **[MCAP Indexing](MCAP_INDEXING.md)** - Understanding MCAP file structure and indexing

## Project Status

This is a working implementation with all core modules functional. The system includes:

- ✅ All 20 modules implemented
- ✅ Simulated sensor data generation
- ✅ Command flow from user input to hardware
- ✅ Bidirectional communication between modules
- ✅ Safety checks and validation
- ✅ System-wide logging to MCAP files

## Contributing

This project is designed to be extensible. Future enhancements could include:

- Real hardware integration
- Advanced path planning algorithms
- Machine learning for environment understanding
- Network-based remote control
- Configuration file support

---

**Version**: 0.1.0
