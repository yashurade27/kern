# DBus Interface Specification

Kern exposes a DBus interface for communication with GNOME Shell and other desktop applications.

## Service Details

- **Service Name**: `org.gnome.Shell.Extensions.Kern`
- **Object Path**: `/org/gnome/Shell/Extensions/Kern`
- **Interface**: `org.gnome.Shell.Extensions.Kern`

## Methods

### GetStatus() → (s)

Returns the current system status as a JSON string.

**Parameters**: None

**Returns**:
- `s` (string): JSON object with system statistics

**Example Return**:
```json
{
  "cpu_usage": 42.5,
  "total_memory_gb": 15.6,
  "used_memory_gb": 8.2,
  "memory_percentage": 52.6,
  "temperature": 65.0,
  "top_processes": [
    {
      "pid": 1234,
      "name": "chrome",
      "memory_gb": 2.5,
      "cpu_percentage": 15.3
    }
  ]
}
```

### GetCurrentMode() → (s)

Returns the name of the currently active profile.

**Parameters**: None

**Returns**:
- `s` (string): Profile name (e.g., "normal", "coding")

**Errors**:
- `org.gnome.Shell.Extensions.Kern.Error.NoProfileActive`: No profile is currently active

### GetAvailableModes() → (as)

Lists all available profile names.

**Parameters**: None

**Returns**:
- `as` (array of strings): List of available profile names

**Example Return**:
```
["normal", "coding", "gaming", "power-saving"]
```

### SetMode(s: profile_name) → (b)

Switches to the specified profile.

**Parameters**:
- `s` (string): Name of the profile to activate

**Returns**:
- `b` (boolean): Success (true) or failure (false)

**Errors**:
- `org.gnome.Shell.Extensions.Kern.Error.ProfileNotFound`: Profile doesn't exist
- `org.gnome.Shell.Extensions.Kern.Error.FailedToSwitch`: Could not switch profiles

**Example Call**:
```javascript
const success = await client.SetModeAsync("coding");
```

### GetProcessKillLog(i: limit) → (aa{sv})

Returns recent process kill events (optional, for future implementation).

**Parameters**:
- `i` (int32): Maximum number of events to return

**Returns**:
- `aa{sv}` (array of dictionaries): Kill events with timestamp, process name, reason

## Signals

### ModeChanged(s: old_mode, s: new_mode)

Emitted when the active profile changes.

**Example**:
```javascript
client.on("ModeChanged", (oldMode, newMode) => {
  console.log(`Profile changed from ${oldMode} to ${newMode}`);
});
```

### ProcessKilled(i: pid, s: name, s: reason)

Emitted when Kern kills a process (only if notifications are enabled).

**Parameters**:
- `i` (int32): Process ID
- `s` (string): Process name
- `s` (string): Reason for killing (e.g., "memory_limit_exceeded")

**Example**:
```javascript
client.on("ProcessKilled", (pid, name, reason) => {
  console.log(`Process ${name} (${pid}) killed: ${reason}`);
});
```

### TemperatureWarning(d: temperature)

Emitted when CPU temperature exceeds warning threshold.

**Parameters**:
- `d` (double): Current CPU temperature in °C

## Error Codes

```
org.gnome.Shell.Extensions.Kern.Error.ProfileNotFound
org.gnome.Shell.Extensions.Kern.Error.FailedToSwitch
org.gnome.Shell.Extensions.Kern.Error.NoProfileActive
org.gnome.Shell.Extensions.Kern.Error.PermissionDenied
org.gnome.Shell.Extensions.Kern.Error.InternalError
```

## Usage Example (JavaScript)

```javascript
const DBus = imports.gi.Gio.DBusProxy;

async function getKernStatus() {
  try {
    const proxy = new DBus({
      name: "org.gnome.Shell.Extensions.Kern",
      object_path: "/org/gnome/Shell/Extensions/Kern",
      interface: "org.gnome.Shell.Extensions.Kern",
    });

    const status = await proxy.GetStatusAsync();
    console.log("Current status:", JSON.parse(status[0]));

    const modes = await proxy.GetAvailableModesAsync();
    console.log("Available profiles:", modes[0]);

    const currentMode = await proxy.GetCurrentModeAsync();
    console.log("Current profile:", currentMode[0]);

    // Switch profile
    const success = await proxy.SetModeAsync("coding");
    if (success[0]) {
      console.log("Successfully switched to coding profile");
    }
  } catch (error) {
    console.error("DBus error:", error);
  }
}
```

## Implementation Notes

- All method calls are asynchronous
- String arrays are properly null-terminated
- Timestamps are Unix epoch seconds
- Temperature values are in Celsius
- Memory values are in gigabytes
- Percentages are 0-100

## Version

Current Version: 1.0
Minimum GNOME Version: 42 (due to DBus API changes)
