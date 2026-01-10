use colored::*;
use std::fs;
use std::thread;
use std::time::Duration;

const CTXT_THRESHOLD: u64 = 5000;
#[derive(Debug, Clone)]
struct Meminfo{
    total: u64,
    free: u64,
    buffers: u64,
    cached: u64,
}
#[derive(Debug, Clone)]
struct CpuStats{
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64
   
}

#[derive(Debug, Clone)]
struct VmStat{
   procs_running: u64,
   procs_blocked: u64,
   mem_info: Meminfo,
   cpu_stats: CpuStats,
   swap_total: u64,
   swap_free: u64,
   context_switches: u64,
   interrupts: u64,
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

fn parse_meminfo() -> Result<Meminfo, Box<dyn std::error::Error>> {
    let content = read_file("/proc/meminfo")?;
    let mut meminfo = Meminfo {
        total: 0,
        free: 0,
        buffers: 0,
        cached: 0,
    };
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "MemTotal:" => meminfo.total = parts[1].parse()?,
            "MemFree:" => meminfo.free = parts[1].parse()?,
            "Buffers:" => meminfo.buffers = parts[1].parse()?,
            "Cached:" => meminfo.cached = parts[1].parse()?,
            _ => (),
        }
    }
    Ok(meminfo)
}

fn parse_stat() -> Result<(CpuStats,u64,u64, u64, u64), Box<dyn std::error::Error>> {
    let content = read_file("/proc/stat")?;
    let mut cpu_stats = CpuStats {
        user: 0,
        nice: 0,
        system: 0,
        idle: 0,
        iowait: 0,
    };
    let mut context_switches = 0;
    let mut interrupts = 0;
    let mut procs_running = 0;
    let mut procs_blocked = 0;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "cpu" => {
                if parts.len() >= 6 {
                    cpu_stats.user = parts[1].parse()?;
                    cpu_stats.nice = parts[2].parse()?;
                    cpu_stats.system = parts[3].parse()?;
                    cpu_stats.idle = parts[4].parse()?;
                    cpu_stats.iowait = parts[5].parse()?;
                
                }

        }
        "ctxt" => {
            if parts.len() >= 2 {
                context_switches = parts[1].parse()?;
            }
        }
        "intr" => {
            if parts.len() >= 2 {
                interrupts = parts[1].parse()?;

            }
        }
        "procs_running" => {
            if parts.len() >= 2 {
                procs_running = parts[1].parse()?;
            }
        }
        "procs_blocked" => {
            if parts.len() >= 2 {
                procs_blocked = parts[1].parse()?;
            }
        }
        _ => (),
    }
}

    Ok((cpu_stats,context_switches, interrupts, procs_running, procs_blocked))
}

fn parse_vmstat() -> Result<VmStat, Box<dyn std::error::Error>> {
    let mem_info = parse_meminfo()?;
    let (cpu_stats, context_switches, interrupts, procs_running, procs_blocked) = parse_stat()?;
    let content = read_file("/proc/meminfo")?;
    let mut swap_total = 0;
    let mut swap_free = 0;
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() <2 {
            continue;

        }
        match parts[0] {
            "SwapTotal:" => swap_total = parts[1].parse()?,
            "SwapFree:" => swap_free = parts[1].parse()?,
            _ => (),
        }   
        
    }
    Ok(VmStat {
        procs_running,
        procs_blocked,
        mem_info,
        cpu_stats,
        swap_total,
        swap_free,
        context_switches,
        interrupts,
    })

}



fn main() {
    println!("Hello, world!");
}
//
//

