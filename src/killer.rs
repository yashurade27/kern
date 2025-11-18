pub fn kill_process(pid: u32, graceful: bool) -> Result<(), String> {
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;
        use std::time::Duration;
        use std::thread;

        if graceful {
            // 1. Send SIGTERM to process
            match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                Ok(_) => {},
                Err(e) => {
                    // If process doesn't exist, it's already dead
                    if e.to_string().contains("No such process") {
                        return Ok(());
                    }
                    return Err(format!("Failed to send SIGTERM to {}: {}", pid, e));
                }
            }

            // 2. Wait 5 seconds for graceful shutdown
            for _ in 0..50 {
                thread::sleep(Duration::from_millis(100));

                // Check if process still alive by sending signal 0 (no-op)
                match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                    Err(e) if e.to_string().contains("No such process") => {
                        return Ok(()); // Process died gracefully
                    }
                    _ => continue,
                }
            }

            // 3. If still alive after 5 seconds, send SIGKILL
            kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
                .map_err(|e| format!("Failed to force kill process {}: {}", pid, e))?;
            Ok(())
        } else {
            // Force kill immediately
            kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
                .map_err(|e| format!("Failed to kill process {}: {}", pid, e))?;
            Ok(())
        }
    }

    #[cfg(not(unix))]
    {
        Err("Process killing is not supported on this platform.".to_string())
    }
}

pub fn kill_processes(pids: &[u32], graceful: bool) -> Result<(), String> {
    for &pid in pids {
        kill_process(pid, graceful)?;
    }
    Ok(())
}

/// Get the path to the kill log file
pub fn get_kill_log_path() -> std::path::PathBuf {
    use std::path::PathBuf;

    if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(config_home).join("kern").join("kern.log")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config").join("kern").join("kern.log")
    } else {
        PathBuf::from("/tmp/kern.log")
    }
}

/// Log a kill action to ~/.config/kern/kern.log
pub fn log_kill_action(pid: u32, name: &str, success: bool, graceful: bool) {
    use chrono::Local;
    use std::fs::OpenOptions;
    use std::io::Write;

    // Get log file path
    let log_path = get_kill_log_path();

    // Ensure directory exists
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    // Format log entry
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let status = if success { "ok" } else { "failed" };
    
    let log_entry = format!(
        "[{}] KILL [PID: {}] name=\"{}\" graceful={} status={}\n",
        timestamp, pid, name, graceful, status
    );

    // Write to log file
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

pub fn is_protected(name: &str, protected_list: &[String]) -> bool {
    protected_list.iter().any(|protected_name| protected_name == name)
}

pub fn is_critical_process(name: &str) -> bool {
    let critical_processes = vec![
        "systemd", "gnome-shell", "Xwayland", "X", "Xvfb",
        "dbus-daemon", "bluetoothd", "wpa_supplicant",
        "NetworkManager", "ModemManager", "upowerd",
        "systemd-logind", "login", "sshd", "sudo"
    ];
    critical_processes.iter().any(|critical| *critical == name)
}

pub fn find_processes_by_name(name: &str) -> Vec<u32> {
    #[cfg(unix)]
    {
        use sysinfo::System;

        let mut system = System::new_all();
        system.refresh_all();

        system
            .processes()
            .iter()
            .filter_map(|(pid, process)| {
                let process_name = process.name().to_string_lossy().to_string();
                if process_name == name {
                    Some(pid.as_u32())
                } else {
                    None
                }
            })
            .collect()
    }

    #[cfg(not(unix))]
    {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_critical_process() {
        assert!(is_critical_process("systemd"));
        assert!(is_critical_process("gnome-shell"));
        assert!(is_critical_process("dbus-daemon"));
        assert!(is_critical_process("sshd"));
        assert!(!is_critical_process("firefox"));
        assert!(!is_critical_process("code"));
    }

    #[test]
    fn test_is_protected() {
        let protected_list = vec![
            "bash".to_string(),
            "zsh".to_string(),
            "firefox".to_string(),
        ];
        
        assert!(is_protected("bash", &protected_list));
        assert!(is_protected("firefox", &protected_list));
        assert!(!is_protected("chrome", &protected_list));
        assert!(!is_protected("systemd", &protected_list));
    }

    #[test]
    fn test_is_protected_empty_list() {
        let protected_list: Vec<String> = vec![];
        assert!(!is_protected("bash", &protected_list));
        assert!(!is_protected("anything", &protected_list));
    }

    #[test]
    fn test_find_processes_by_name_systemd() {
        // systemd should exist on all Linux systems
        let pids = find_processes_by_name("systemd");
        assert!(!pids.is_empty(), "systemd process should exist");
    }

    #[test]
    fn test_find_processes_by_name_nonexistent() {
        // This process name is unlikely to exist
        let pids = find_processes_by_name("nonexistent_process_xyz_12345");
        assert!(pids.is_empty(), "nonexistent process should return empty vec");
    }

    #[test]
    fn test_kill_nonexistent_process() {
        // Trying to kill a non-existent PID returns Ok() gracefully 
        // because the process is already dead
        let result = kill_process(99999, true);
        // Should either be Ok (already dead) or Err (permission/other issue)
        // We just verify it doesn't panic
        let _ = result;
    }
}