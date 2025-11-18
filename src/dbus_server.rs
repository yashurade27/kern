use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use zbus::dbus_interface;
use zbus::Connection;

use crate::config::KernConfig;
use crate::monitor;
use crate::profiles::ProfileManager;

/// DBus interface implementation for Kern
/// Service: org.gnome.Shell.Extensions.Kern
/// Object Path: /org/gnome/Shell/Extensions/Kern
pub struct KernDBusInterface {
    profile_manager: Arc<RwLock<ProfileManager>>,
    #[allow(dead_code)]
    config: Arc<KernConfig>,
}

impl KernDBusInterface {
    pub fn new(profile_manager: ProfileManager, config: KernConfig) -> Self {
        Self {
            profile_manager: Arc::new(RwLock::new(profile_manager)),
            config: Arc::new(config),
        }
    }
}

#[dbus_interface(name = "org.gnome.Shell.Extensions.Kern")]
impl KernDBusInterface {
    /// GetStatus() → (s)
    /// Returns the current system status as a JSON string
    async fn get_status(&self) -> zbus::fdo::Result<String> {
        let stats = monitor::get_system_stats()
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to get system stats: {}", e)))?;

        let top: Vec<serde_json::Value> = stats
            .top_processes
            .iter()
            .take(10)
            .map(|p| {
                json!({
                    "pid": p.pid,
                    "name": p.name,
                    "memory_gb": p.memory_gb,
                    "cpu_percentage": p.cpu_percentage,
                })
            })
            .collect();

        let status_json = json!({
            "cpu_usage": stats.cpu_usage,
            "total_memory_gb": stats.total_memory_gb,
            "used_memory_gb": stats.used_memory_gb,
            "memory_percentage": stats.memory_percentage,
            "temperature": stats.temperature,
            "top_processes": top,
        });

        Ok(serde_json::to_string(&status_json).unwrap_or_else(|_| "{}".to_string()))
    }

    /// GetCurrentMode() → (s)
    /// Returns the name of the currently active profile
    async fn get_current_mode(&self) -> zbus::fdo::Result<String> {
        let manager = self.profile_manager.read().await;
        Ok(manager.current_name().to_string())
    }

    /// GetAvailableModes() → (as)
    /// Lists all available profile names
    async fn get_available_modes(&self) -> zbus::fdo::Result<Vec<String>> {
        let manager = self.profile_manager.read().await;
        Ok(manager.list_names())
    }

    /// SetMode(s: profile_name) → (b)
    /// Switches to the specified profile
    async fn set_mode(&self, profile_name: &str) -> zbus::fdo::Result<bool> {
        let mut manager = self.profile_manager.write().await;

        if !manager.list_names().contains(&profile_name.to_string()) {
            return Err(zbus::fdo::Error::Failed(format!(
                "Profile '{}' not found",
                profile_name
            )));
        }

        manager.switch_to(profile_name).map_err(|e| {
            zbus::fdo::Error::Failed(format!("Failed to switch profile: {}", e))
        })?;

        Ok(true)
    }

    /// GetProcessKillLog(i: limit) → (as)
    /// Returns recent process kill events
    async fn get_process_kill_log(&self, limit: i32) -> zbus::fdo::Result<Vec<String>> {
        let limit = limit.max(0) as usize;

        // Read kill log from file
        let log_file = crate::killer::get_kill_log_path();

        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let contents = std::fs::read_to_string(&log_file)
            .map_err(|e| zbus::fdo::Error::Failed(format!("Failed to read log file: {}", e)))?;

        let lines: Vec<String> = if limit == 0 {
            contents.lines().map(|s| s.to_string()).collect()
        } else {
            contents
                .lines()
                .rev()
                .take(limit)
                .map(|s| s.to_string())
                .collect()
        };

        Ok(lines)
    }
}

/// Start the DBus server
pub async fn start_dbus_server(
    profile_manager: ProfileManager,
    config: KernConfig,
) -> Result<()> {
    let kern_iface = KernDBusInterface::new(profile_manager, config);

    let connection = Connection::session().await?;

    connection
        .object_server()
        .at("/org/gnome/Shell/Extensions/Kern", kern_iface)
        .await?;

    connection
        .request_name("org.gnome.Shell.Extensions.Kern")
        .await?;

    eprintln!("✅ DBus server started: org.gnome.Shell.Extensions.Kern");

    // Keep the connection alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KernConfig;
    use crate::profiles::ProfileManager;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_dbus_interface_creation() {
        // Create a temporary directory for test config
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        // Create a minimal profiles directory with a test profile
        let profiles_dir = config_path.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let test_profile = r#"
name: "test"
description: "Test profile"
limits:
  max_cpu_percent: 90
  max_ram_percent: 85
  max_temp: 85
"#;

        std::fs::write(profiles_dir.join("test.yaml"), test_profile).unwrap();

        let profile_manager =
            ProfileManager::new(Some(config_path.to_path_buf())).expect("Failed to create PM");
        let config = KernConfig::load().expect("Failed to load config");

        let iface = KernDBusInterface::new(profile_manager, config);

        // Verify the interface was created successfully
        assert!(!iface.profile_manager.read().await.list_names().is_empty());
    }

    #[tokio::test]
    async fn test_get_current_mode() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        let profiles_dir = config_path.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let test_profile = r#"
name: "test"
description: "Test profile"
"#;

        std::fs::write(profiles_dir.join("test.yaml"), test_profile).unwrap();

        let profile_manager =
            ProfileManager::new(Some(config_path.to_path_buf())).expect("Failed to create PM");
        let config = KernConfig::load().expect("Failed to load config");

        let iface = KernDBusInterface::new(profile_manager, config);

        let current_mode = iface.get_current_mode().await.unwrap();
        assert_eq!(current_mode, "test");
    }

    #[tokio::test]
    async fn test_get_available_modes() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        let profiles_dir = config_path.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let profiles = vec!["test1", "test2", "test3"];
        for profile_name in &profiles {
            let profile_content = format!(
                r#"
name: "{}"
description: "Test profile {}"
"#,
                profile_name, profile_name
            );
            std::fs::write(
                profiles_dir.join(format!("{}.yaml", profile_name)),
                profile_content,
            )
            .unwrap();
        }

        let profile_manager =
            ProfileManager::new(Some(config_path.to_path_buf())).expect("Failed to create PM");
        let config = KernConfig::load().expect("Failed to load config");

        let iface = KernDBusInterface::new(profile_manager, config);

        let available_modes = iface.get_available_modes().await.unwrap();
        assert_eq!(available_modes.len(), 3);
        assert!(available_modes.contains(&"test1".to_string()));
        assert!(available_modes.contains(&"test2".to_string()));
        assert!(available_modes.contains(&"test3".to_string()));
    }

    #[tokio::test]
    async fn test_set_mode_valid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        let profiles_dir = config_path.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let profiles = vec!["test1", "test2"];
        for profile_name in &profiles {
            let profile_content = format!(
                r#"
name: "{}"
description: "Test profile {}"
"#,
                profile_name, profile_name
            );
            std::fs::write(
                profiles_dir.join(format!("{}.yaml", profile_name)),
                profile_content,
            )
            .unwrap();
        }

        let profile_manager =
            ProfileManager::new(Some(config_path.to_path_buf())).expect("Failed to create PM");
        let config = KernConfig::load().expect("Failed to load config");

        let iface = KernDBusInterface::new(profile_manager, config);

        // Set to test2
        let result = iface.set_mode("test2").await.unwrap();
        assert!(result);

        // Verify the change
        let current_mode = iface.get_current_mode().await.unwrap();
        assert_eq!(current_mode, "test2");
    }

    #[tokio::test]
    async fn test_set_mode_invalid() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        let profiles_dir = config_path.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let test_profile = r#"
name: "test"
description: "Test profile"
"#;

        std::fs::write(profiles_dir.join("test.yaml"), test_profile).unwrap();

        let profile_manager =
            ProfileManager::new(Some(config_path.to_path_buf())).expect("Failed to create PM");
        let config = KernConfig::load().expect("Failed to load config");

        let iface = KernDBusInterface::new(profile_manager, config);

        // Try to set to non-existent profile
        let result = iface.set_mode("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_status_format() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path();

        let profiles_dir = config_path.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let test_profile = r#"
name: "test"
description: "Test profile"
"#;

        std::fs::write(profiles_dir.join("test.yaml"), test_profile).unwrap();

        let profile_manager =
            ProfileManager::new(Some(config_path.to_path_buf())).expect("Failed to create PM");
        let config = KernConfig::load().expect("Failed to load config");

        let iface = KernDBusInterface::new(profile_manager, config);

        let status_json = iface.get_status().await.unwrap();

        // Verify the JSON contains required fields
        let parsed: serde_json::Value = serde_json::from_str(&status_json).unwrap();
        assert!(parsed.get("cpu_usage").is_some());
        assert!(parsed.get("total_memory_gb").is_some());
        assert!(parsed.get("used_memory_gb").is_some());
        assert!(parsed.get("memory_percentage").is_some());
        assert!(parsed.get("temperature").is_some());
        assert!(parsed.get("top_processes").is_some());
    }
}
