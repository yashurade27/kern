use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub protected: Vec<String>, // Processes that should never be killed in this profile
    #[serde(default)]
    pub kill_on_activate: Vec<String>, // Processes to kill automatically when this profile is activated
    #[serde(default)] 
    pub limits: ProfileResourceLimits, // Resource limits for this profile
    #[serde(default)]
    pub auto_activate: AutoActivateConfig, // Auto-activation rules
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResourceLimits {
    #[serde(default = "default_max_cpu")]
    pub max_cpu_percent: f64, 
    #[serde(default = "default_max_ram")]
    pub max_ram_percent: f64,
    #[serde(default = "default_max_temp")]
    pub max_temp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoActivateConfig { 
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub triggers: Vec<AutoActivateTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoActivateTrigger {
    #[serde(rename = "type")]
    pub trigger_type: Option<String>,
    pub command_contains: Option<String>,
}

// Default values
fn default_max_cpu() -> f64 {
    90.0
}

fn default_max_ram() -> f64 {
    85.0
}

fn default_max_temp() -> f64 {
    85.0
}

impl Default for ProfileResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: default_max_cpu(),
            max_ram_percent: default_max_ram(),
            max_temp: default_max_temp(),
        }
    }
}

impl Default for AutoActivateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            triggers: Vec::new(),
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            protected: Vec::new(),
            kill_on_activate: Vec::new(),
            limits: ProfileResourceLimits::default(),
            auto_activate: AutoActivateConfig::default(),
        }
    }
}

impl Profile {
    /// Load a single profile from a YAML file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let profile: Profile = serde_yaml::from_str(&contents)?;
        profile.validate()?;
        Ok(profile)
    }

    /// Validate profile values
    fn validate(&self) -> Result<()> {
        // Validate name is not empty
        if self.name.is_empty() {
            return Err(anyhow!("Profile name cannot be empty"));
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

        // Validate temperature (0-120°C is reasonable range)
        if !(0.0..=120.0).contains(&self.limits.max_temp) {
            return Err(anyhow!(
                "Invalid max_temp: {} (must be 0-120°C)",
                self.limits.max_temp
            ));
        }

        Ok(())
    }
}

/// Manager for loading and switching between profiles
pub struct ProfileManager {
    profiles: HashMap<String, Profile>,
    current_profile: String,
    config_dir: PathBuf,
}

impl ProfileManager {
    /// Create a new profile manager and load all profiles from config directory
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self> {
        let config_dir = if let Some(dir) = config_dir {
            dir
        } else {
            Self::default_config_dir()?
        };

        let profiles_dir = config_dir.join("profiles");

        let mut profiles = HashMap::new();

        // Try to load all YAML files from profiles directory
        if profiles_dir.exists() {
            for entry in fs::read_dir(&profiles_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_file() && path.extension().map_or(false, |ext| ext == "yaml") {
                    if let Some(filename) = path.file_stem() {
                        let profile_name = filename.to_string_lossy().to_string();
                        match Profile::load_from_file(&path) {
                            Ok(profile) => {
                                profiles.insert(profile_name, profile);
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to load profile {}: {}",
                                    profile_name, e
                                );
                            }
                        }
                    }
                }
            }
        }

        if profiles.is_empty() {
            return Err(anyhow!(
                "No profiles found in {}. Please create profile files.",
                profiles_dir.display()
            ));
        }

        // Default to "normal" profile if it exists, otherwise use first available
        let current_profile = if profiles.contains_key("normal") {
            "normal".to_string()
        } else {
            profiles.keys().next().unwrap().clone()
        };

        Ok(Self {
            profiles,
            current_profile,
            config_dir,
        })
    }

    /// Get the default config directory following XDG standard
    fn default_config_dir() -> Result<PathBuf> {
        if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
            Ok(PathBuf::from(config_home).join("kern"))
        } else if let Ok(home) = std::env::var("HOME") {
            Ok(PathBuf::from(home).join(".config").join("kern"))
        } else {
            Err(anyhow!("Cannot determine config directory (no HOME or XDG_CONFIG_HOME set)"))
        }
    }

    /// Get the current active profile
    pub fn current(&self) -> Result<&Profile> {
        self.profiles
            .get(&self.current_profile)
            .ok_or_else(|| anyhow!("Current profile '{}' not found", self.current_profile))
    }

    /// Switch to a different profile
    pub fn switch_to(&mut self, profile_name: &str) -> Result<()> {
        if !self.profiles.contains_key(profile_name) {
            return Err(anyhow!(
                "Profile '{}' not found. Available: {}",
                profile_name,
                self.list_names().join(", ")
            ));
        }

        self.current_profile = profile_name.to_string();
        self.save_state()?;
        Ok(())
    }

    /// Get a specific profile by name
    pub fn get(&self, profile_name: &str) -> Option<&Profile> {
        self.profiles.get(profile_name)
    }

    /// List all available profile names
    pub fn list_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.profiles.keys().cloned().collect();
        names.sort();
        names
    }

    /// List all available profiles with details
    pub fn list_all(&self) -> Vec<(&str, &Profile)> {
        let mut profiles: Vec<_> = self
            .profiles
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        profiles.sort_by_key(|a| a.0);
        profiles
    }

    /// Get the current profile name
    pub fn current_name(&self) -> &str {
        &self.current_profile
    }

    /// Save current profile state to config directory
    fn save_state(&self) -> Result<()> {
        let state_file = self.config_dir.join(".state");
        fs::write(&state_file, &self.current_profile)?;
        Ok(())
    }

    /// Load saved profile state from config directory
    pub fn load_state(&mut self) -> Result<()> {
        let state_file = self.config_dir.join(".state");
        if state_file.exists() {
            let saved_profile = fs::read_to_string(&state_file)?;
            let saved_profile = saved_profile.trim();
            if self.profiles.contains_key(saved_profile) {
                self.current_profile = saved_profile.to_string();
            }
        }
        Ok(())
    }

    /// Print all profiles summary
    pub fn print_summary(&self) {
        println!("📋 Available Profiles");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        for (name, profile) in self.list_all() {
            let is_current = if name == self.current_profile {
                " (current)"
            } else {
                ""
            };
            println!("{}{}", name, is_current);
            println!("  └─ {}", profile.description);
            println!(
                "     CPU: {}%, RAM: {}%, Temp: {}°C",
                profile.limits.max_cpu_percent,
                profile.limits.max_ram_percent,
                profile.limits.max_temp
            );
            println!(
                "     Protected: {} | Kill on activate: {}",
                profile.protected.len(),
                profile.kill_on_activate.len()
            );
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_resource_limits_default() {
        let limits = ProfileResourceLimits::default();
        assert_eq!(limits.max_cpu_percent, 90.0);
        assert_eq!(limits.max_ram_percent, 85.0);
        assert_eq!(limits.max_temp, 85.0);
    }

    #[test]
    fn test_auto_activate_config_default() {
        let config = AutoActivateConfig::default();
        assert!(!config.enabled);
        assert!(config.triggers.is_empty());
    }

    #[test]
    fn test_profile_validation_cpu_percent() {
        let mut profile = Profile {
            name: "test".to_string(),
            description: "Test profile".to_string(),
            protected: vec![],
            kill_on_activate: vec![],
            limits: ProfileResourceLimits::default(),
            auto_activate: AutoActivateConfig::default(),
        };

        // Invalid: negative CPU
        profile.limits.max_cpu_percent = -1.0;
        assert!(profile.validate().is_err());

        // Invalid: CPU > 100
        profile.limits.max_cpu_percent = 101.0;
        assert!(profile.validate().is_err());

        // Valid
        profile.limits.max_cpu_percent = 50.0;
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_profile_validation_ram_percent() {
        let mut profile = Profile {
            name: "test".to_string(),
            description: "Test profile".to_string(),
            protected: vec![],
            kill_on_activate: vec![],
            limits: ProfileResourceLimits::default(),
            auto_activate: AutoActivateConfig::default(),
        };

        // Invalid: negative RAM
        profile.limits.max_ram_percent = -5.0;
        assert!(profile.validate().is_err());

        // Invalid: RAM > 100
        profile.limits.max_ram_percent = 150.0;
        assert!(profile.validate().is_err());

        // Valid
        profile.limits.max_ram_percent = 70.0;
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_profile_validation_temperature() {
        let mut profile = Profile {
            name: "test".to_string(),
            description: "Test profile".to_string(),
            protected: vec![],
            kill_on_activate: vec![],
            limits: ProfileResourceLimits::default(),
            auto_activate: AutoActivateConfig::default(),
        };

        // Invalid: negative temperature
        profile.limits.max_temp = -10.0;
        assert!(profile.validate().is_err());

        // Invalid: temperature > 120°C
        profile.limits.max_temp = 150.0;
        assert!(profile.validate().is_err());

        // Valid
        profile.limits.max_temp = 80.0;
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_profile_validation_empty_name() {
        let profile = Profile {
            name: String::new(),
            description: "Test profile".to_string(),
            protected: vec![],
            kill_on_activate: vec![],
            limits: ProfileResourceLimits::default(),
            auto_activate: AutoActivateConfig::default(),
        };

        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_parse_profile_yaml() {
        let yaml = r#"
name: "Test Mode"
description: "A test profile"
protected:
  - systemd
  - gnome-shell
kill_on_activate:
  - chrome
  - spotify
limits:
  max_cpu_percent: 75
  max_ram_percent: 80
  max_temp: 90
auto_activate:
  enabled: false
  triggers: []
"#;

        let profile: Profile = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(profile.name, "Test Mode");
        assert_eq!(profile.description, "A test profile");
        assert_eq!(profile.protected.len(), 2);
        assert_eq!(profile.kill_on_activate.len(), 2);
        assert_eq!(profile.limits.max_cpu_percent, 75.0);
        assert_eq!(profile.limits.max_ram_percent, 80.0);
        assert_eq!(profile.limits.max_temp, 90.0);
        assert!(!profile.auto_activate.enabled);
        assert!(profile.validate().is_ok());
    }

    #[test]
    fn test_parse_profile_minimal_yaml() {
        let yaml = r#"
name: "Minimal"
description: "Minimal profile"
"#;

        let profile: Profile = serde_yaml::from_str(yaml).expect("Failed to parse YAML");
        assert_eq!(profile.name, "Minimal");
        assert!(profile.protected.is_empty());
        assert!(profile.kill_on_activate.is_empty());
        // Should use defaults
        assert_eq!(profile.limits.max_cpu_percent, 90.0);
        assert_eq!(profile.limits.max_ram_percent, 85.0);
        assert_eq!(profile.limits.max_temp, 85.0);
        assert!(profile.validate().is_ok());
    }
}


