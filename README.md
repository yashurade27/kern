# Kern - Smart Resource Manager for Developers

Kern is an intelligent resource management system for Ubuntu that helps developers maintain optimal system performance during coding, building, and testing.

## Features

- **Profile-based resource management** - Switch between Coding, Building, and Normal modes
- **Real-time monitoring** - Track CPU, RAM, and temperature
- **Smart process killing** - Automatically manage resource-hungry processes
- **GNOME Shell integration** - System tray indicator with quick controls
- **Desktop notifications** - Stay informed of system actions
- **Process protection** - Never kill critical development tools

## Installation
```bash
# Build from source
cargo build --release

# Install
sudo ./scripts/install.sh
```

## Usage
```bash
# Show current system status
kern status

# Switch to coding mode
kern mode coding

# Switch to building mode
kern mode building

# Kill a specific process
kern kill chrome

# Protect a process from being killed
kern protect code
```

## Configuration

Configuration files are located in `~/.config/kern/`

See [docs/PROFILES.md](docs/PROFILES.md) for profile configuration details.
