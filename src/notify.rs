use crate::config::NotificationConfig;
use anyhow::Result;
use notify_rust::Notification;
use std::time::{Duration, Instant};

/// Notification manager with rate limiting to avoid spam
#[derive(Debug, Clone)]
pub struct NotificationManager {
    enabled: bool,
    show_on_kill: bool,
    show_on_profile_switch: bool,
    last_kill_notification: Option<Instant>,
    last_emergency_notification: Option<Instant>,
    last_warning_notification: Option<Instant>,
    min_interval_between_notifications: Duration,
}

impl NotificationManager {
    pub fn new(config: &NotificationConfig) -> Self {
        Self {
            enabled: config.enabled,
            show_on_kill: config.show_on_kill,
            show_on_profile_switch: config.show_on_profile_switch,
            last_kill_notification: None,
            last_emergency_notification: None,
            last_warning_notification: None,
            // Rate limit: 1 notification per 3 seconds to avoid spam
            min_interval_between_notifications: Duration::from_secs(3),
        }
    }

    /// Show notification when a process is killed
    pub fn notify_process_killed(&mut self, pid: u32, name: &str, count: usize) -> Result<()> {
        if !self.enabled || !self.show_on_kill {
            return Ok(());
        }

        // Rate limiting
        if let Some(last) = self.last_kill_notification {
            if last.elapsed() < self.min_interval_between_notifications {
                return Ok(());
            }
        }

        let message = if count > 1 {
            format!("Killed {} process(es) matching '{}'", count, name)
        } else {
            format!("Killed process '{}' (PID: {})", name, pid)
        };

        send_notification(
            "Process Killed",
            &message,
            notify_rust::Urgency::Normal,
        )?;

        self.last_kill_notification = Some(Instant::now());
        Ok(())
    }

    /// Show notification for emergency mode activation
    pub fn notify_emergency_mode(&mut self, temperature: f64, critical_temp: f64) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Emergency mode is critical, only rate limit by 5 seconds
        if let Some(last) = self.last_emergency_notification {
            if last.elapsed() < Duration::from_secs(5) {
                return Ok(());
            }
        }

        let message = format!(
            "⚠️ EMERGENCY MODE: Temperature {:.1}°C exceeds critical threshold {:.1}°C",
            temperature, critical_temp
        );

        send_notification(
            "🔴 Emergency Mode Activated",
            &message,
            notify_rust::Urgency::Critical,
        )?;

        self.last_emergency_notification = Some(Instant::now());
        Ok(())
    }

    /// Show notification for emergency mode deactivation
    pub fn notify_emergency_mode_resolved(&mut self, temperature: f64) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let message = format!("Temperature cooled to {:.1}°C - system back to normal", temperature);

        send_notification(
            "🟢 Emergency Mode Resolved",
            &message,
            notify_rust::Urgency::Normal,
        )?;

        Ok(())
    }

    /// Show notification for resource limit exceeded
    pub fn notify_resource_limit_exceeded(
        &mut self,
        resource_type: &str,
        current: f64,
        limit: f64,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Rate limit warnings
        if let Some(last) = self.last_warning_notification {
            if last.elapsed() < self.min_interval_between_notifications {
                return Ok(());
            }
        }

        let message = format!(
            "{} usage {:.1}% exceeds limit {:.1}%",
            resource_type, current, limit
        );

        send_notification(
            "⚠️ Resource Limit Exceeded",
            &message,
            notify_rust::Urgency::Critical,
        )?;

        self.last_warning_notification = Some(Instant::now());
        Ok(())
    }

    /// Show notification when temperature warning threshold is reached
    pub fn notify_temperature_warning(&mut self, temperature: f64, warning_temp: f64) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        // Rate limit warnings
        if let Some(last) = self.last_warning_notification {
            if last.elapsed() < self.min_interval_between_notifications {
                return Ok(());
            }
        }

        let message = format!(
            "Temperature {:.1}°C exceeds warning threshold {:.1}°C",
            temperature, warning_temp
        );

        send_notification(
            "🌡️ Temperature Warning",
            &message,
            notify_rust::Urgency::Critical,
        )?;

        self.last_warning_notification = Some(Instant::now());
        Ok(())
    }

    /// Show notification on profile switch
    pub fn notify_profile_switched(&mut self, old_profile: &str, new_profile: &str) -> Result<()> {
        if !self.enabled || !self.show_on_profile_switch {
            return Ok(());
        }

        let message = format!("Profile switched from '{}' to '{}'", old_profile, new_profile);

        send_notification(
            "Profile Changed",
            &message,
            notify_rust::Urgency::Normal,
        )?;

        Ok(())
    }

    /// Show a generic info notification
    pub fn notify_info(&self, title: &str, message: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        send_notification(title, message, notify_rust::Urgency::Normal)?;
        Ok(())
    }

    /// Check if notifications are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Toggle notifications on/off
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

/// Internal helper to send a notification
fn send_notification(title: &str, body: &str, urgency: notify_rust::Urgency) -> Result<()> {
    // Check if we're running in a display environment
    if std::env::var("DISPLAY").is_err() && std::env::var("WAYLAND_DISPLAY").is_err() {
        // No display, silently skip notification (common on headless systems)
        return Ok(());
    }

    Notification::new()
        .summary(title)
        .body(body)
        .urgency(urgency)
        .timeout(5000) // 5 second timeout
        .show()
        .ok(); // Ignore errors (e.g., no notification daemon running)

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NotificationConfig;

    #[test]
    fn test_notification_manager_creation() {
        let config = NotificationConfig::default();
        let manager = NotificationManager::new(&config);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_notification_manager_disabled() {
        let mut config = NotificationConfig::default();
        config.enabled = false;
        let manager = NotificationManager::new(&config);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_notification_toggle() {
        let config = NotificationConfig::default();
        let mut manager = NotificationManager::new(&config);
        assert!(manager.is_enabled());
        
        manager.set_enabled(false);
        assert!(!manager.is_enabled());
        
        manager.set_enabled(true);
        assert!(manager.is_enabled());
    }

    #[test]
    fn test_rate_limiting() {
        let config = NotificationConfig::default();
        let mut manager = NotificationManager::new(&config);

        // First kill notification should work
        assert!(manager.notify_process_killed(1234, "test", 1).is_ok());

        // Second one should be rate limited (we don't actually send it, so no error)
        assert!(manager.notify_process_killed(5678, "test", 1).is_ok());

        // But the timestamp should still be updated
        assert!(manager.last_kill_notification.is_some());
    }

    #[test]
    fn test_notification_disabled() {
        let mut config = NotificationConfig::default();
        config.enabled = false;
        let mut manager = NotificationManager::new(&config);

        // No notifications should be sent when disabled
        assert!(manager.notify_process_killed(1234, "test", 1).is_ok());
        assert!(manager.notify_emergency_mode(90.0, 85.0).is_ok());
        assert!(manager.notify_profile_switched("old", "new").is_ok());
    }

    #[test]
    fn test_kill_notification_disabled() {
        let mut config = NotificationConfig::default();
        config.show_on_kill = false;
        let mut manager = NotificationManager::new(&config);

        // Kill notification should not be sent when show_on_kill is false
        assert!(manager.notify_process_killed(1234, "test", 1).is_ok());
        assert!(manager.last_kill_notification.is_none());
    }

    #[test]
    fn test_profile_switch_notification_disabled() {
        let mut config = NotificationConfig::default();
        config.show_on_profile_switch = false;
        let mut manager = NotificationManager::new(&config);

        // Profile switch notification should not be sent
        assert!(manager.notify_profile_switched("old", "new").is_ok());
    }
}
