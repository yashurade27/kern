# Kern Documentation

This directory contains detailed documentation for the Kern project.

## Files

- **[PROFILES.md](./PROFILES.md)** - Complete guide to creating and managing resource profiles
- **[DBUS.md](./DBUS.md)** - DBus interface specification for GNOME integration
- **[README.md](./README.md)** - Overview and getting started

## Quick Start

Kern is a smart resource and process manager for developers. It helps you:

- Monitor system resources (CPU, RAM, temperature)
- Create custom resource profiles for different workflows
- Automatically manage processes based on system load
- Integrate with GNOME Shell through DBus

### Basic Commands

```bash
# Check system status
kern status

# List all processes by memory usage
kern list

# Switch to a different profile
kern mode coding

# Get real-time stats as JSON
kern status --json

# Debug thermal zones
kern thermal
```

### Configuration

Configuration files are located in `~/.config/kern/`:

- `kern.yaml` - Main configuration
- `profiles/` - Directory containing profile definitions

### Profiles

Profiles let you define different operating modes:

- **normal** - Default balanced mode
- **coding** - Optimized for development work
- **gaming** - Maximum performance mode
- **power-saving** - Minimal resource usage

See [PROFILES.md](./PROFILES.md) for detailed information.

### Integration

Kern integrates with:

- **GNOME Shell** - Via extension in the system menu
- **systemd** - As a user service
- **Desktop Notifications** - For alerts and warnings

See [DBUS.md](./DBUS.md) for technical details.
