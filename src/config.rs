use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernConfig { // overall configuration
    #[serde(default = "default_profile")]
    pub default_profile: String,

    // Monitoring interval in seconds (how often to check system stats)
    #[serde(default = "default_monitor_interval")]
    pub monitor_interval: u64,

    // Temperature thresholds for warnings and critical states
    #[serde(default)]
    pub temperature: TemperatureConfig,

    //  Default resource limits
    #[serde(default)]
    pub limits: ResourceLimits,

    // List of processes that should never be killed
    #[serde(default = "default_protected_processes")]
    pub protected_processes: Vec<String>,

    // Notification settings
    #[serde(default)]
    pub notifications: NotificationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureConfig { // temperature thresholds
    // Warning threshold in °C
    #[serde(default = "default_temp_warning")]
    pub warning: f64,

    // Critical threshold in °C (triggers emergency mode)
    #[serde(default = "default_temp_critical")]
    pub critical: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits { // resource usage limits
    // Maximum CPU usage percentage (0-100)
    #[serde(default = "default_max_cpu")]
    pub max_cpu_percent: f64,

    // Maximum RAM usage percentage (0-100)
    #[serde(default = "default_max_ram")]
    pub max_ram_percent: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig { // notification settings
    // Enable desktop notifications
    #[serde(default = "default_notifications_enabled")]
    pub enabled: bool,

    // Show notification when a process is killed
    #[serde(default = "default_show_on_kill")]
    pub show_on_kill: bool,

    // Show notification when profile is switched
    #[serde(default = "default_show_on_profile_switch")]
    pub show_on_profile_switch: bool,
}

// Default values
fn default_profile() -> String {
    "normal".to_string()
}

fn default_monitor_interval() -> u64 {
    2
}

fn default_temp_warning() -> f64 {
    75.0
}

fn default_temp_critical() -> f64 {
    85.0
}

fn default_max_cpu() -> f64 {
    90.0
}

fn default_max_ram() -> f64 {
    85.0
}

fn default_protected_processes() -> Vec<String> {
    vec!["systemd".to_string(), "gnome-shell".to_string(), "kern".to_string()]
}

fn default_notifications_enabled() -> bool {
    true
}

fn default_show_on_kill() -> bool {
    true
}

fn default_show_on_profile_switch() -> bool {
    true
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self {
            warning: default_temp_warning(),
            critical: default_temp_critical(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: default_max_cpu(),
            max_ram_percent: default_max_ram(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: default_notifications_enabled(),
            show_on_kill: default_show_on_kill(),
            show_on_profile_switch: default_show_on_profile_switch(),
        }
    }
}

impl Default for KernConfig {
    fn default() -> Self {
        Self {
            default_profile: default_profile(),
            monitor_interval: default_monitor_interval(),
            temperature: TemperatureConfig::default(),
            limits: ResourceLimits::default(),
            protected_processes: default_protected_processes(),
            notifications: NotificationConfig::default(),
        }
    }
}

impl KernConfig {
    /// Load configuration from file system with fallbacks
    ///
    /// Tries to load in this order:
    /// 1. ~/.config/kern/kern.yaml (user config)
    /// 2. /etc/kern/kern.yaml (system config)
    /// 3. Compiled-in defaults
    pub fn load() -> Result<Self> {
        // Try user config first
        if let Some(config_path) = Self::user_config_path() {
            if config_path.exists() {
                return Self::load_from_file(&config_path);
            }
        }

        // Try system config
        let system_config_path = PathBuf::from("/etc/kern/kern.yaml");
        if system_config_path.exists() {
            return Self::load_from_file(&system_config_path);
        }

        // Use defaults
        Ok(Self::default())
    }

    fn load_from_file(path: &PathBuf) -> Result<Self> { // load config from specified path
        let contents = fs::read_to_string(path)?;
        let config: KernConfig = serde_yaml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    fn user_config_path() -> Option<PathBuf> { // get user config path following XDG standard
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            Some(PathBuf::from(config_home).join("kern").join("kern.yaml"))
        } else if let Ok(home) = std::env::var("HOME") {
            Some(PathBuf::from(home).join(".config").join("kern").join("kern.yaml"))
        } else {
            None
        }
    }

    fn validate(&self) -> Result<()> { // validate config values
        // Validate monitor interval
        if self.monitor_interval < 1 {
            return Err(anyhow!(
                "Invalid monitor_interval: {} (must be >= 1 second)",
                self.monitor_interval
            ));
        }

        if self.monitor_interval > 3600 {
            return Err(anyhow!(
                "Invalid monitor_interval: {} (must be <= 3600 seconds)",
                self.monitor_interval
            ));
        }

        // Validate percentages
        if !(0.0..=100.0).contains(&self.limits.max_cpu_percent) {
            return Err(anyhow!(
                "Invalid max_cpu_percent: {} (must be 0-100)",
                self.limits.max_cpu_percent
            ));
        }

        if !(0.0..=100.0).contains(&self.limits.max_ram_percent) {
            return Err(anyhow!(
                "Invalid max_ram_percent: {} (must be 0-100)",
                self.limits.max_ram_percent
            ));
        }

        // Validate temperatures (0-120°C is reasonable range)
        if !(0.0..=120.0).contains(&self.temperature.warning) {
            return Err(anyhow!(
                "Invalid temperature.warning: {} (must be 0-120°C)",
                self.temperature.warning
            ));
        }

        if !(0.0..=120.0).contains(&self.temperature.critical) {
            return Err(anyhow!(
                "Invalid temperature.critical: {} (must be 0-120°C)",
                self.temperature.critical
            ));
        }

        // Validate temperature ordering
        if self.temperature.critical <= self.temperature.warning {
            return Err(anyhow!(
                "Invalid temperatures: critical ({}) must be > warning ({})",
                self.temperature.critical,
                self.temperature.warning
            ));
        }

        Ok(())
    }

    // Print configuration summary
    pub fn print_summary(&self) {
        println!(" KERN Configuration Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Default Profile: {}", self.default_profile);
        println!("Monitor Interval: {} seconds", self.monitor_interval);
        println!(
            "Temperature Warning: {:.0}°C, Critical: {:.0}°C",
            self.temperature.warning, self.temperature.critical
        );
        println!(
            "Resource Limits: CPU {}%, RAM {}%",
            self.limits.max_cpu_percent, self.limits.max_ram_percent
        );
        println!(
            "Notifications: {} (kill: {}, profile: {})",
            if self.notifications.enabled { "enabled" } else { "disabled" },
            self.notifications.show_on_kill,
            self.notifications.show_on_profile_switch
        );
        println!("Protected Processes: {}", self.protected_processes.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = KernConfig::default();
        assert_eq!(config.default_profile, "normal");
        assert_eq!(config.monitor_interval, 2);
        assert_eq!(config.limits.max_cpu_percent, 90.0);
        assert_eq!(config.limits.max_ram_percent, 85.0);
    }

    #[test]
    fn test_config_validation_interval() {
        let mut config = KernConfig::default();

        // Invalid: too low
        config.monitor_interval = 0;
        assert!(config.validate().is_err());

        // Invalid: too high
        config.monitor_interval = 7200;
        assert!(config.validate().is_err());

        // Valid
        config.monitor_interval = 5;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_cpu_percent() {
        let mut config = KernConfig::default();

        config.limits.max_cpu_percent = -1.0;
        assert!(config.validate().is_err());

        config.limits.max_cpu_percent = 101.0;
        assert!(config.validate().is_err());

        config.limits.max_cpu_percent = 50.0;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_temperature() {
        let mut config = KernConfig::default();

        // Invalid: critical not higher than warning
        config.temperature.critical = 70.0;
        config.temperature.warning = 75.0;
        assert!(config.validate().is_err());

        // Valid
        config.temperature.warning = 70.0;
        config.temperature.critical = 80.0;
        assert!(config.validate().is_ok());

        // Invalid: temperature out of range
        config.temperature.warning = -5.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
default_profile: "coding"
monitor_interval: 3
temperature:
  warning: 75
  critical: 85
limits:
  max_cpu_percent: 80
  max_ram_percent: 75
protected_processes:
  - systemd
  - gnome-shell
  - code
notifications:
  enabled: true
  show_on_kill: true
  show_on_profile_switch: true
"#;

        let config: KernConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(config.default_profile, "coding");
        assert_eq!(config.monitor_interval, 3);
        assert_eq!(config.limits.max_cpu_percent, 80.0);
        assert!(config.protected_processes.contains(&"code".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_parse_minimal_yaml() {
        let yaml = r#"
default_profile: "normal"
"#;

        let config: KernConfig = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(config.default_profile, "normal");
        // Other fields should use defaults
        assert_eq!(config.monitor_interval, 2);
        assert_eq!(config.limits.max_cpu_percent, 90.0);
    }
}
