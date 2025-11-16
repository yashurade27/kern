mod monitor;
mod config;

use anyhow::Result;
use clap::{Parser, Subcommand, CommandFactory};
use std::process::Command;


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
        let arr: Vec<serde_json::Value> = processes
            .iter()
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

fn kill_process_by_name(name: &str) -> Result<()> {
    if let Some(pid) = monitor::find_process_by_name(name) {
        println!("Found process '{}' -> PID {}", name, pid);
        let status = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()?;
        if status.success() {
            println!("✓ Sent SIGTERM to {} (PID {})", name, pid);
        } else {
            println!("✗ Failed to send SIGTERM; exit code: {}", status);
        }
    } else {
        println!("No running process found matching '{}'", name);
    }
    Ok(())
}

fn main() -> Result<()> {
    // Load configuration at startup
    let config = config::KernConfig::load()?;
    config.print_summary();
    println!();

    let cli = Cli::parse();

    if cli.monitor {
        return monitor_loop(config.monitor_interval);
    }

    match cli.command {
        Some(Commands::Status { json }) => print_status(json)?,
        Some(Commands::List { json, count }) => print_list(json, count)?,
        Some(Commands::Kill { name }) => kill_process_by_name(&name)?,
        Some(Commands::Mode { profile }) => {
            println!("Mode switching to '{}' (not yet implemented)", profile);
        }
        Some(Commands::Thermal) => monitor::debug_thermal_zones()?,
        None => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
