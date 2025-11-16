use anyhow::Result;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub memory_gb: f64,
    pub cpu_percentage: f64,
}

#[derive(Debug)]
pub struct SystemStats {
    pub cpu_usage: f64,
    pub total_memory_gb: f64,
    pub used_memory_gb: f64,
    pub memory_percentage: f64,
    pub temperature: f64,
    pub top_processes: Vec<ProcessInfo>,
}

fn get_process_memory_from_proc(pid: u32) -> Option<u64> {
    let status_path = format!("/proc/{}/status", pid);
    let contents = std::fs::read_to_string(status_path).ok()?;
    
    for line in contents.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(kb) = parts[1].parse::<u64>() {
                    return Some(kb * 1024);
                }
            }
        }
    }
    None
}

fn is_thread(pid: u32) -> bool {
    if let Ok(contents) = std::fs::read_to_string(format!("/proc/{}/status", pid)) {
        let mut tgid = None;
        let mut pid_val = None;
        
        for line in contents.lines() {
            if line.starts_with("Tgid:") {
                tgid = line.split_whitespace().nth(1).and_then(|s| s.parse::<u32>().ok());
            } else if line.starts_with("Pid:") {
                pid_val = line.split_whitespace().nth(1).and_then(|s| s.parse::<u32>().ok());
            }
        }
        
        if let (Some(tgid), Some(pid_val)) = (tgid, pid_val) {
            return tgid != pid_val;
        }
    }
    false
}

pub fn get_system_stats() -> Result<SystemStats> {
    let mut sys = System::new_all();
    sys.refresh_all();

    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    let cpu_usage = sys.global_cpu_usage() as f64;

    let total_memory = sys.total_memory() as f64 / 1_073_741_824.0;
    let used_memory = sys.used_memory() as f64 / 1_073_741_824.0;
    let memory_percentage = (used_memory / total_memory) * 100.0;

    let temperature = get_cpu_temperature().unwrap_or(0.0);

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let pid_val = pid.as_u32();
            
            if is_thread(pid_val) {
                return None;
            }
            
            let memory_bytes = get_process_memory_from_proc(pid_val)
                .unwrap_or_else(|| process.memory());
            
            Some(ProcessInfo {
                pid: pid_val,
                name: process.name().to_string_lossy().to_string(),
                memory_gb: memory_bytes as f64 / 1_073_741_824.0,
                cpu_percentage: process.cpu_usage() as f64,
            })
        })
        .collect();

    processes.sort_by(|a, b| b.memory_gb.partial_cmp(&a.memory_gb).unwrap());

    Ok(SystemStats {
        cpu_usage,
        total_memory_gb: total_memory,
        used_memory_gb: used_memory,
        memory_percentage,
        temperature,
        top_processes: processes,
    })
}

pub fn get_all_processes() -> Result<Vec<ProcessInfo>> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .filter_map(|(pid, process)| {
            let pid_val = pid.as_u32();
            
            if is_thread(pid_val) {
                return None;
            }
            
            let memory_bytes = get_process_memory_from_proc(pid_val)
                .unwrap_or_else(|| process.memory());
            
            Some(ProcessInfo {
                pid: pid_val,
                name: process.name().to_string_lossy().to_string(),
                memory_gb: memory_bytes as f64 / 1_073_741_824.0,
                cpu_percentage: process.cpu_usage() as f64,
            })
        })
        .collect();

    processes.sort_by(|a, b| b.memory_gb.partial_cmp(&a.memory_gb).unwrap());

    Ok(processes)
}

pub fn find_process_by_name(name: &str) -> Option<u32> {
    let sys = System::new_all();
    
    for (pid, process) in sys.processes() {
        let process_name = process.name().to_string_lossy().to_lowercase();
        if process_name.contains(&name.to_lowercase()) {
            return Some(pid.as_u32());
        }
    }

    None
}

fn get_cpu_temperature() -> Result<f64> {
    let thermal_zones = [
        "/sys/class/thermal/thermal_zone4/temp",
        "/sys/class/thermal/thermal_zone6/temp",
        "/sys/class/thermal/thermal_zone1/temp",
        "/sys/class/thermal/thermal_zone2/temp",
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/class/thermal/thermal_zone5/temp",
        "/sys/class/thermal/thermal_zone3/temp",
    ];

    for path in &thermal_zones {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(temp) = contents.trim().parse::<f64>() {
                return Ok(temp / 1000.0);
            }
        }
    }
    Ok(0.0)
}

pub fn debug_thermal_zones() -> Result<()> {
    println!("Available thermal zones:");
    for i in 0..10 {
        let type_path = format!("/sys/class/thermal/thermal_zone{}/type", i);
        let temp_path = format!("/sys/class/thermal/thermal_zone{}/temp", i);
        
        if let Ok(zone_type) = std::fs::read_to_string(&type_path) {
            if let Ok(temp_str) = std::fs::read_to_string(&temp_path) {
                if let Ok(temp) = temp_str.trim().parse::<f64>() {
                    println!("  thermal_zone{}: {} - {:.2}°C", i, zone_type.trim(), temp / 1000.0);
                }
            }
        }
    }
    Ok(())
}