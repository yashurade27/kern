mod monitor;
mod config;
mod profiles;
mod killer;
mod enforcer;
mod stats;

use anyhow::Result;
use clap::{Parser, Subcommand, CommandFactory};
use std::io::{self, Write};


#[derive(Debug, Parser)]
#[command(name = "kern", about = "Resource and process monitor CLI tool", version)]
struct Cli { // kern --monitor
    /// Start monitoring loop (updates every 2 seconds)
    #[arg(long, default_value_t = false)]
    monitor: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands { // kern status , kern list , kern kill [process_name] , kern mode [profile_name]
    Status {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    List {
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(short, long, default_value_t = 20)]
        count: usize,
    },
    Kill {
        name: String,
    },
    Mode {
        profile: String,
    },
    /// Start enforcer loop (monitors and enforces resource limits)
    Enforce,
    /// Debug thermal zones (shows all available temperature sensors)
    Thermal,
}

fn print_status(json: bool) -> Result<()> {
    let stats = monitor::get_system_stats()?;

    if json {
        let top: Vec<serde_json::Value> = stats
            .top_processes
            .iter()
            .map(|p| {
                serde_json::json!({
                    "pid": p.pid,
                    "name": p.name,
                    "memory_gb": p.memory_gb,
                    "cpu_percentage": p.cpu_percentage,
                })
            })
            .collect();

        let jsonout = serde_json::json!({
            "cpu_usage": stats.cpu_usage,
            "total_memory_gb": stats.total_memory_gb,
            "used_memory_gb": stats.used_memory_gb,
            "memory_percentage": stats.memory_percentage,
            "temperature": stats.temperature,
            "top_processes": top,
        });
        println!("{}", serde_json::to_string_pretty(&jsonout)?);
        return Ok(());
    }

    println!("📊 KERN - System Status");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("CPU: {:.2}%", stats.cpu_usage);
    println!("RAM: {:.2} GB / {:.2} GB ({:.2}%)", 
        stats.used_memory_gb, stats.total_memory_gb, stats.memory_percentage);
    println!("Temp: {:.2} °C", stats.temperature);
    println!();

    println!("Top processes by memory:");
    for (idx, p) in stats.top_processes.iter().take(5).enumerate() {
        println!("  {}. {} (PID: {}) - {:.2} GB - {:.2}% CPU", 
            idx + 1, p.name, p.pid, p.memory_gb, p.cpu_percentage);
    }

    Ok(())
}

fn print_list(json: bool, count: usize) -> Result<()> {
    let processes = monitor::get_all_processes()?;
    if json {
        // For JSON mode, only output the JSON array without config summary
        let arr: Vec<serde_json::Value> = processes
            .iter()
            .take(count)
            .map(|p| {
                serde_json::json!({
                    "pid": p.pid,
                    "name": p.name,
                    "memory_gb": p.memory_gb,
                    "cpu_percentage": p.cpu_percentage
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr)?);
        return Ok(());
    }

    println!("{:<8} {:<8} {:<8} {}", "PID", "MEM(GB)", "CPU%", "NAME");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    for p in processes.iter().take(count) {
        println!("{:<8} {:<8.2} {:<8.2} {}", p.pid, p.memory_gb, p.cpu_percentage, p.name);
    }
    Ok(())
}

fn monitor_loop(interval_secs: u64) -> Result<()> {
    println!("Starting monitor loop (interval: {} seconds). Press Ctrl+C to exit.", interval_secs);
    println!();
    
    loop {
        print_status(false)?;
        println!();
        std::thread::sleep(std::time::Duration::from_secs(interval_secs));
    }
}

fn kill_process_by_name(name: &str, config: &config::KernConfig) -> Result<()> {
    // Find all processes matching the name
    let pids = killer::find_processes_by_name(name);
    
    if pids.is_empty() {
        println!("❌ No running process found matching '{}'", name);
        return Ok(());
    }
    
    println!("Found {} process(es) matching '{}'", pids.len(), name);
    
    // Check if process is critical
    if killer::is_critical_process(name) {
        println!("❌ Cannot kill '{}' - it is a critical system process", name);
        return Ok(());
    }
    
    // Check if process is protected
    if killer::is_protected(name, &config.protected_processes) {
        println!("❌ Cannot kill '{}' - it is in the protected process list", name);
        return Ok(());
    }
    
    // If more than threshold, ask for confirmation
    if pids.len() > config.kill_confirmation_threshold {
        println!("\n⚠️  This will kill {} processes. Are you sure? (yes/no)", pids.len());
        print!("Please confirm: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("yes") && !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }
    
    // Kill the processes
    match killer::kill_processes(&pids, config.kill_graceful) {
        Ok(_) => {
            let kill_type = if config.kill_graceful { "gracefully" } else { "forcefully" };
            println!("✅ Killed {} process(es) {} (PID: {})", 
                pids.len(), 
                kill_type,
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            
            // Log the action for each PID
            for pid in &pids {
                killer::log_kill_action(*pid, name, true, config.kill_graceful);
            }
        }
        Err(e) => {
            println!("❌ Error killing processes: {}", e);
            // Log failed attempt
            for pid in &pids {
                killer::log_kill_action(*pid, name, false, config.kill_graceful);
            }
        }
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load configuration at startup
    let config = config::KernConfig::load()?;
    
    // Suppress config summary in JSON mode
    let is_json_mode = match &cli.command {
        Some(Commands::Status { json }) => *json,
        Some(Commands::List { json, .. }) => *json,
        _ => false,
    };
    
    if !is_json_mode {
        config.print_summary();
        println!();
    }

    if cli.monitor {
        return monitor_loop(config.monitor_interval);
    }

    match cli.command {
        Some(Commands::Status { json }) => print_status(json)?,
        Some(Commands::List { json, count }) => print_list(json, count)?,
        Some(Commands::Kill { name }) => kill_process_by_name(&name, &config)?,
        Some(Commands::Mode { profile }) => {
            println!("Mode switching to '{}' (not yet implemented)", profile);
        }
        Some(Commands::Enforce) => {
            let default_profile = profiles::Profile {
                name: config.default_profile.clone(),
                ..Default::default()
            };
            enforcer::run_enforcer_loop(config, default_profile)?;
        }
        Some(Commands::Thermal) => monitor::debug_thermal_zones()?,
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
