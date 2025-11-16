# Kern Profiles Guide

Profiles in Kern allow you to define different operating modes with custom resource limits and process management rules.

## Profile Structure

Each profile is a YAML file located in `~/.config/kern/profiles/`.

### Example Profile

```yaml
name: "Coding"
description: "Optimized for development work"

protected:
  - systemd
  - gnome-shell
  - code
  - cargo

kill_on_activate:
  - chrome
  - firefox

limits:
  max_cpu_percent: 80
  max_ram_percent: 85
  max_temp: 85

auto_activate:
  enabled: false
  triggers: []
```

## Fields

### Basic Fields

- **name**: Profile identifier (required, non-empty)
- **description**: Human-readable profile description

### Protected Processes

The `protected` list contains process names that should never be killed, even when resource limits are exceeded. Essential system processes like `systemd` and `gnome-shell` should always be protected.

### Kill on Activate

The `kill_on_activate` list specifies processes to automatically terminate when this profile is activated. Useful for clearing out resource hogs when switching modes.

### Resource Limits

The `limits` section defines resource thresholds:

- **max_cpu_percent**: Maximum CPU usage (0-100%)
  - Default: 90%
  - When exceeded: Kern will kill the heaviest CPU-consuming process
  
- **max_ram_percent**: Maximum RAM usage (0-100%)
  - Default: 85%
  - When exceeded: Kern will kill the largest memory-consuming process
  
- **max_temp**: Maximum CPU temperature (0-120°C)
  - Default: 85°C
  - When exceeded: Kern activates emergency mode (kills non-critical processes)

### Auto-Activation

The `auto_activate` section enables automatic profile switching based on system conditions:

```yaml
auto_activate:
  enabled: true
  triggers:
    - type: "cpu"
      threshold: 85
    - type: "memory"
      threshold: 90
```

## Built-in Profiles

### normal
Default balanced profile. Suitable for general usage with moderate resource limits.

### coding
Optimized for development work. Protects development tools and IDEs while limiting background processes.

### gaming
Maximum performance mode. High limits with minimal process killing.

### power-saving
Minimal resource usage. Aggressive process killing to extend battery life.

## Creating Custom Profiles

1. Create a new YAML file in `~/.config/kern/profiles/`:
   ```bash
   mkdir -p ~/.config/kern/profiles
   nano ~/.config/kern/profiles/custom.yaml
   ```

2. Define your profile with the structure above

3. Activate the profile:
   ```bash
   kern mode custom
   ```

4. Verify it's active:
   ```bash
   kern status
   ```

## Validation Rules

Profiles are validated when loaded:

- Name cannot be empty
- CPU percentage must be between 0-100%
- RAM percentage must be between 0-100%
- Temperature must be between 0-120°C
- All fields must be valid YAML

Invalid profiles will be rejected with a clear error message.

## Best Practices

1. **Always protect essential processes**: Include `systemd`, `gnome-shell`, `kern`
2. **Be conservative with kill_on_activate**: Only kill processes you're sure about
3. **Set reasonable limits**: Don't set limits too low (0%) or too high (100%)
4. **Test before deploying**: Verify profile behavior before daily use
5. **Use descriptive names**: Make profile purposes clear in the name

## Troubleshooting

### Profile not loading
- Check YAML syntax with: `yamllint ~/.config/kern/profiles/myprofile.yaml`
- Ensure all required fields are present
- Check file permissions: `chmod 644 ~/.config/kern/profiles/myprofile.yaml`

### Profile not switching
- Verify profile name exists: `kern mode normal` (should work)
- Check logs: Profile switching is logged to stdout

### Processes being killed unexpectedly
- Review `kill_on_activate` list - remove aggressive entries
- Increase resource limits if they're too restrictive
- Add processes to `protected` list if they shouldn't be killed
