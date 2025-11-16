use anyhow::Result;
use sysinfo::{System};

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

//get current system stats
pub fn get_system_stats() -> Result<SystemStats> { // kern status
    let mut sys = System::new_all();
    sys.refresh_all();

    std::thread::sleep(std::time::Duration::from_millis(200));
    sys.refresh_cpu_all();

    let cpu_usage = sys.global_cpu_usage() as f64;

    let total_memory = sys.total_memory() as f64 / 1_073_741_824.0; // Convert bytes to GB
    let used_memory = sys.used_memory() as f64 / 1_073_741_824.0; // Convert bytes to GB
    let memory_percentage = (used_memory / total_memory) * 100.0;

    let temperature = get_cpu_temperature().unwrap_or(0.0);

    // Process list
    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            memory_gb: process.memory() as f64 / 1_073_741_824.0,
            cpu_percentage: process.cpu_usage() as f64,
        })
        .collect();

    // Sort by memory usage (descending)
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

pub fn get_all_processes() -> Result<Vec<ProcessInfo>> { //kern list
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            memory_gb: process.memory() as f64 / 1_073_741_824.0,
            cpu_percentage: process.cpu_usage() as f64,
        })
        .collect();

    // Sort by memory usage
    processes.sort_by(|a, b| b.memory_gb.partial_cmp(&a.memory_gb).unwrap());

    Ok(processes)
}

// find process to kill it
pub fn find_process_by_name(name: &str) -> Option<u32> { //kern kill [process_name] , eg: kern kill chrome
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
    // Check thermal zones in order of preference (CPU-related first)
    let thermal_zones = [
        "/sys/class/thermal/thermal_zone4/temp", // TCPU (CPU)
        "/sys/class/thermal/thermal_zone6/temp", // x86_pkg_temp (package)
        "/sys/class/thermal/thermal_zone1/temp", // TSKN (skin)
        "/sys/class/thermal/thermal_zone2/temp", // TMEM (memory)
        "/sys/class/thermal/thermal_zone0/temp", // INT3400
        "/sys/class/thermal/thermal_zone5/temp", // iwlwifi
        "/sys/class/thermal/thermal_zone3/temp", // NGFF
    ];

    for path in &thermal_zones {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(temp) = contents.trim().parse::<f64>() {
                // Convert from millidegree Celsius to degree Celsius
                return Ok(temp / 1000.0);
            }
        }
    }
    Ok(0.0)
}

// Debug function to list all thermal zones and their readings
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