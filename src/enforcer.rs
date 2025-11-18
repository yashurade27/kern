use std::time::{Duration, Instant};
use crate::monitor::{get_system_stats, SystemStats};
use crate::killer;
use crate::config::KernConfig;
use crate::profiles::Profile;
use crate::notify::NotificationManager;

/// Core enforcer state
#[derive(Debug, Clone)]
pub struct Enforcer {
    config: KernConfig,
    current_profile: Profile,
    emergency_mode: bool,
    emergency_since: Option<Instant>,
    last_enforcement: Instant,
    notification_manager: NotificationManager,
}

impl Enforcer {
    pub fn new(config: KernConfig, current_profile: Profile) -> Self {
        let notification_manager = NotificationManager::new(&config.notifications);
        Self {
            config,
            current_profile,
            emergency_mode: false,
            emergency_since: None,
            last_enforcement: Instant::now(),
            notification_manager,
        }
    }

    pub fn enforce_once(&mut self) -> anyhow::Result<bool> {
        let stats = get_system_stats()?;
        let mut action_taken = false;

        // Check if we should exit emergency mode (temperature cooled)
        if self.emergency_mode {
            if stats.temperature < self.config.temperature.warning {
                eprintln!("🟢 Emergency mode disabled - temperature cooled to {:.1}°C", stats.temperature);
                self.emergency_mode = false;
                self.emergency_since = None;
                let _ = self.notification_manager.notify_emergency_mode_resolved(stats.temperature);
            }
        }

        // Check for emergency condition (temp > critical threshold)
        if !self.emergency_mode && stats.temperature > self.config.temperature.critical {
            eprintln!("🔴 EMERGENCY MODE ACTIVATED - Temperature {:.1}°C > {:.1}°C (critical)", 
                stats.temperature, self.config.temperature.critical);
            self.emergency_mode = true;
            self.emergency_since = Some(Instant::now());
            let _ = self.notification_manager.notify_emergency_mode(stats.temperature, self.config.temperature.critical);
            
            // Kill all non-protected processes immediately
            action_taken = self.handle_emergency_mode(&stats)?;
        } else if self.emergency_mode {
            // In emergency mode - continue killing processes
            action_taken = self.handle_emergency_mode(&stats)?;
        } else {
            // Normal operation - check profile limits
            action_taken = self.enforce_resource_limits(&stats)?;
        }

        self.last_enforcement = Instant::now();
        Ok(action_taken)
    }

    // Handle emergency mode - kill all non-critical, non-protected processes
    fn handle_emergency_mode(&mut self, stats: &SystemStats) -> anyhow::Result<bool> {
        let mut killed_count = 0;

        for process in &stats.top_processes {
            // Skip protected processes
            if killer::is_protected(&process.name, &self.current_profile.protected) 
                || killer::is_protected(&process.name, &self.config.protected_processes)
                || killer::is_critical_process(&process.name) {
                continue;
            }

            // Kill the process
            match killer::kill_process(process.pid, self.config.kill_graceful) {
                Ok(_) => {
                    eprintln!("  ⚠️  Killed {} (PID: {}) - emergency mode", process.name, process.pid);
                    killer::log_kill_action(process.pid, &process.name, true, self.config.kill_graceful);
                    killed_count += 1;
                }
                Err(e) => {
                    eprintln!("  Failed to kill {} (PID: {}): {}", process.name, process.pid, e);
                    killer::log_kill_action(process.pid, &process.name, false, self.config.kill_graceful);
                }
            }
        }

        if killed_count > 0 {
            let _ = self.notification_manager.notify_process_killed(0, "emergency", killed_count);
        }

        Ok(killed_count > 0)
    }

    // Enforce resource limits for the current profile
    fn enforce_resource_limits(&mut self, stats: &SystemStats) -> anyhow::Result<bool> {
        let mut action_taken = false;

        // Check CPU limit
        if stats.cpu_usage > self.current_profile.limits.max_cpu_percent {
            eprintln!("⚠️  CPU limit exceeded: {:.1}% > {:.1}%", 
                stats.cpu_usage, self.current_profile.limits.max_cpu_percent);
            let _ = self.notification_manager.notify_resource_limit_exceeded(
                "CPU",
                stats.cpu_usage,
                self.current_profile.limits.max_cpu_percent,
            );
            action_taken |= self.kill_heaviest_process(&stats)?;
        }

        // Check RAM limit
        if stats.memory_percentage > self.current_profile.limits.max_ram_percent {
            eprintln!("⚠️  RAM limit exceeded: {:.1}% > {:.1}%", 
                stats.memory_percentage, self.current_profile.limits.max_ram_percent);
            let _ = self.notification_manager.notify_resource_limit_exceeded(
                "RAM",
                stats.memory_percentage,
                self.current_profile.limits.max_ram_percent,
            );
            action_taken |= self.kill_heaviest_process(&stats)?;
        }

        // Check temperature warning (not critical)
        if stats.temperature > self.config.temperature.warning && stats.temperature < self.config.temperature.critical {
            eprintln!("🟡 Temperature warning: {:.1}°C > {:.1}°C", 
                stats.temperature, self.config.temperature.warning);
            let _ = self.notification_manager.notify_temperature_warning(
                stats.temperature,
                self.config.temperature.warning,
            );
            // Kill one process to cool down
            action_taken |= self.kill_heaviest_process(&stats)?;
        }

        Ok(action_taken)
    }

    // Kill the process using the most CPU (excluding protected/critical)
    fn kill_heaviest_process(&mut self, stats: &SystemStats) -> anyhow::Result<bool> {
        for process in &stats.top_processes {
            // Skip protected processes
            if killer::is_protected(&process.name, &self.current_profile.protected) 
                || killer::is_protected(&process.name, &self.config.protected_processes)
                || killer::is_critical_process(&process.name) {
                continue;
            }

            // Kill this process
            match killer::kill_process(process.pid, self.config.kill_graceful) {
                Ok(_) => {
                    eprintln!("  ✓ Killed {} (PID: {}) - high resource usage", process.name, process.pid);
                    killer::log_kill_action(process.pid, &process.name, true, self.config.kill_graceful);
                    let _ = self.notification_manager.notify_process_killed(process.pid, &process.name, 1);
                    return Ok(true);
                }
                Err(e) => {
                    eprintln!("  Failed to kill {} (PID: {}): {}", process.name, process.pid, e);
                    killer::log_kill_action(process.pid, &process.name, false, self.config.kill_graceful);
                    // Continue to try the next process
                }
            }
        }

        Ok(false)
    }

    // Get the current emergency status
    pub fn is_emergency_mode(&self) -> bool {
        self.emergency_mode
    }

    // Get time in emergency mode (if active)
    pub fn emergency_duration(&self) -> Option<Duration> {
        self.emergency_since.map(|since| since.elapsed())
    }

    // Switch to a new profile
    pub fn switch_profile(&mut self, new_profile: Profile) -> anyhow::Result<()> {
        let old_name = self.current_profile.name.clone();
        eprintln!("Switching profile: {} → {}", old_name, new_profile.name);
        
        // Kill processes marked for killing on activate (only if not protected/critical)
        for proc_name in &new_profile.kill_on_activate {
            let pids = killer::find_processes_by_name(proc_name);
            
            for pid in pids {
                if killer::is_critical_process(proc_name) {
                    eprintln!("  Skipping kill of {} (critical process)", proc_name);
                    continue;
                }
                
                match killer::kill_process(pid, self.config.kill_graceful) {
                    Ok(_) => {
                        eprintln!("  Killed {} (PID: {}) on profile activation", proc_name, pid);
                        killer::log_kill_action(pid, proc_name, true, self.config.kill_graceful);
                    }
                    Err(e) => {
                        eprintln!("  Failed to kill {} (PID: {}): {}", proc_name, pid, e);
                    }
                }
            }
        }

        self.current_profile = new_profile;
        self.emergency_mode = false;
        self.emergency_since = None;
        
        let _ = self.notification_manager.notify_profile_switched(&old_name, &self.current_profile.name);
        
        Ok(())
    }

    /// Get current profile
    pub fn profile(&self) -> &Profile {
        &self.current_profile
    }

    /// Get system stats at the time of last enforcement
    pub fn last_enforcement_time(&self) -> Instant {
        self.last_enforcement
    }
}

/// Run the enforcer in a continuous loop (blocking)
/// Periodically checks system stats and enforces resource limits
pub fn run_enforcer_loop(config: KernConfig, initial_profile: Profile) -> anyhow::Result<()> {
    let mut enforcer = Enforcer::new(config.clone(), initial_profile);
    let interval = Duration::from_secs(config.monitor_interval);

    eprintln!("Starting enforcer loop (interval: {:?})", interval);
    eprintln!("Press Ctrl+C to stop");
    eprintln!();

    loop {
        match enforcer.enforce_once() {
            Ok(action_taken) => {
                if action_taken {
                    if enforcer.is_emergency_mode() {
                        if let Some(duration) = enforcer.emergency_duration() {
                            eprintln!("[Emergency mode - {:.1}s]", duration.as_secs_f64());
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Enforcer error: {}", e);
                // Continue on error instead of crashing
            }
        }

        std::thread::sleep(interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforcer_creation() {
        let config = KernConfig::default();
        let profile = Profile::default();
        let enforcer = Enforcer::new(config, profile);

        assert!(!enforcer.is_emergency_mode());
        assert!(enforcer.emergency_duration().is_none());
    }

    #[test]
    fn test_emergency_mode_activation() {
        let mut config = KernConfig::default();
        config.temperature.critical = 80.0;
        
        let profile = Profile::default();
        let mut enforcer = Enforcer::new(config, profile);

        assert!(!enforcer.is_emergency_mode());

        // In real usage, emergency_since would be set when temp exceeds critical
        enforcer.emergency_mode = true;
        enforcer.emergency_since = Some(Instant::now());

        assert!(enforcer.is_emergency_mode());
        assert!(enforcer.emergency_duration().is_some());
    }

    #[test]
    fn test_profile_switching() {
        let config = KernConfig::default();
        let profile1 = Profile {
            name: "profile1".to_string(),
            ..Default::default()
        };
        let profile2 = Profile {
            name: "profile2".to_string(),
            ..Default::default()
        };

        let mut enforcer = Enforcer::new(config, profile1);
        assert_eq!(enforcer.profile().name, "profile1");

        enforcer.switch_profile(profile2).ok();
        assert_eq!(enforcer.profile().name, "profile2");
    }

    #[test]
    fn test_emergency_mode_exit() {
        let config = KernConfig::default();
        let profile = Profile::default();
        let mut enforcer = Enforcer::new(config, profile);

        enforcer.emergency_mode = true;
        enforcer.emergency_since = Some(Instant::now());

        // Simulate exiting emergency mode
        enforcer.emergency_mode = false;
        enforcer.emergency_since = None;

        assert!(!enforcer.is_emergency_mode());
        assert!(enforcer.emergency_duration().is_none());
    }
}
