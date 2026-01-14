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
   pgfault: u64,
   pgmajfault: u64,
   oom_kill: u64,

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

    //Parsing page fault statistics from /proc/vmstat
    let vmstat_content = read_file("/proc/vmstat")?;
    let mut pgfault = 0;
    let mut pgmajfault = 0;
    let mut oom_kill = 0;
    for line in vmstat_content.lines(){
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() <2 {
            continue;
        }
        match parts[0]{
            "pgfault" => pgfault = parts[1].parse()?,
            "pgmajfault" => pgmajfault = parts[1].parse()?,
            "oom_kill" => oom_kill = parts[1].parse()?,
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
        pgfault,
        pgmajfault,
        oom_kill
    })

}

//The key Snapshot Function: Snapshot A vs Snapshot B
fn calculate_deltas(prev: &VmStat, curr: &VmStat, interval_secs: f64) -> (u64,u64, u64, u64,f64,f64,f64,f64, f64){

    //Calcullate delat: B - A for context switches
    let cs_delta = curr.context_switches.saturating_sub(prev.context_switches);
    let in_delta = curr.interrupts.saturating_sub(prev.interrupts);

    // convert to per second rate
    let cs_per_sec = (cs_delta as f64 / interval_secs) as u64;
    let in_per_sec = (in_delta as f64 / interval_secs) as u64;

    // Calculate CPU usage percentages
    let total_prev = prev.cpu_stats.user + prev.cpu_stats.nice + prev.cpu_stats.system + prev.cpu_stats.idle + prev.cpu_stats.iowait;
    let total_curr = curr.cpu_stats.user + curr.cpu_stats.nice + curr.cpu_stats.system + curr.cpu_stats.idle + curr.cpu_stats.iowait;

    let total_delta = total_curr.saturating_sub(total_prev) as f64;

    //Calculate page fault deltas
    let total_faults_now = curr.pgfault;
    let total_faults_prev = prev.pgfault;

    // Get the major page faults now and before
    let major_faults_now = curr.pgmajfault;
    let major_faults_prev = prev.pgmajfault;

    // Calculate the minor page faults (total - major)
    let minor_faults_now = total_faults_now.saturating_sub(major_faults_now);
    let minor_faults_prev = total_faults_prev.saturating_sub(major_faults_prev);

    //Calculate the per-second rates
    let minor_delta = minor_faults_now.saturating_sub(minor_faults_prev);
    let major_delta = major_faults_now.saturating_sub(major_faults_prev);
    
    let minor_per_sec = (minor_delta as f64 / interval_secs) as u64;
    let major_per_sec = (major_delta as f64 / interval_secs) as u64;

    // Calculate percentage of time spent in each state (B - A) / total * 100
    let us = if total_delta > 0.0 {
        ((curr.cpu_stats.user.saturating_sub(prev.cpu_stats.user)) as f64 / total_delta) * 100.0
    } else {
        0.0
    };
    
    let sy = if total_delta > 0.0 {
        ((curr.cpu_stats.system.saturating_sub(prev.cpu_stats.system)) as f64 / total_delta) * 100.0
    } else {
        0.0
    };
    
    let id = if total_delta > 0.0 {
        ((curr.cpu_stats.idle.saturating_sub(prev.cpu_stats.idle)) as f64 / total_delta) * 100.0
    } else {
        0.0
    };
    
    let wa = if total_delta > 0.0 {
        ((curr.cpu_stats.iowait.saturating_sub(prev.cpu_stats.iowait)) as f64 / total_delta) * 100.0
    } else {
        0.0
    };

    (cs_per_sec, in_per_sec, minor_per_sec, major_per_sec,us, sy, id, wa, 0.0)
}

fn print_header() {
    println!("{:>2} {:>2} {:>9} {:>9} {:>6} {:>6} {:>6} {:>6} {:>3} {:>3} {:>3} {:>3} {:>6} {:>6} {:>4} {:>15}",
             "r", "b", "swpd", "free", "buff", "cache", "si", "so", "bi", "bo", "in", "cs", "min", "maj", "oom", "us sy id wa st");
}

fn print_stat(stat: &VmStat, cs_per_sec: u64, in_per_sec: u64, minor_per_sec: u64, major_per_sec: u64,
    us: f64, sy: f64, id: f64, wa: f64, oom_detected: bool) {
    let swpd = (stat.swap_total - stat.swap_free) / 1024; // Convert to MB
    let free = stat.mem_info.free / 1024; // MB
    let buff = stat.mem_info.buffers / 1024; // MB
    let cache = stat.mem_info.cached / 1024; // MB

    let line = format!(
        "{:>2} {:>2} {:>9} {:>9} {:>6} {:>6} {:>6} {:>6} {:>3} {:>3} {:>3} {:>3} {:>6} {:>6} {:>4} {:>3.0} {:>2.0} {:>2.0} {:>2.0} {:>2}",
        stat.procs_running,
        stat.procs_blocked,
        swpd,
        free,
        buff,
        cache,
        0, // si (swap in)
        0, // so (swap out)
        0, // bi (blocks in)
        0, // bo (blocks out)
        in_per_sec,
        cs_per_sec,
        minor_per_sec,
        major_per_sec,
        if oom_detected { "YES" } else {" "},
        us,
        sy,
        id,
        wa,
        0  // st (steal time)
    );

    // CRITICAL ALERT: OOM Kill detected (highest priority)
    if oom_detected {
        println!("{}", line.red().bold());
        eprintln!("{}", "🚨🚨🚨 OOM KILL DETECTED! 🚨🚨🚨".red().bold());
        eprintln!("{}", "A process was terminated by the kernel to save memory!".red().bold());
        eprintln!("{}", "Check `dmesg | tail` or `journalctl -xe` for details.".red());
    }
    // ALERT: Print in RED if context switches exceed threshold
    if cs_per_sec > CTXT_THRESHOLD {
        println!("{}", line.red().bold());
        eprintln!("⚠️  {} - CPU thrashing detected!", 
                  format!("Context switches: {}/sec", cs_per_sec).red().bold());
    }else if major_per_sec > 100 {
        println!("{}", line.yellow().bold());
        eprintln!("⚠️  {} - High major page faults detected!", 
                  format!("Major page faults: {}/sec", major_per_sec).yellow().bold());
    } else {
        println!("{}", line);
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 vmstat-rs - Context Switch Monitor");
    println!("Threshold: {}+ context switches/sec will be highlighted in RED", CTXT_THRESHOLD);
    println!("Major Page Fault Threshold: 100+ maj/sec = YELLOW alert");
    println!("OOM Kill Detection: CRITICAL RED alert if kernel kills a process");
    println!("Press Ctrl+C to exit\n");
    
    
    print_header();

    // SNAPSHOT A: Take initial reading
    let mut prev_stat = parse_vmstat()?;
    
    // Print first line with zeros for rates (no previous snapshot to compare)
    print_stat(&prev_stat, 0, 0, 0, 0, 0.0, 0.0, 100.0, 0.0, false);

    // Main monitoring loop
    loop {
        // SLEEP: Wait 1 second
        thread::sleep(Duration::from_secs(1));
        
        // SNAPSHOT B: Take new reading
        let curr_stat = parse_vmstat()?;
        
        // CALCULATE: (B - A) / time_interval
        let (cs_per_sec, in_per_sec, minor_per_sec, major_per_sec, us, sy, id, wa, _st) = 
            calculate_deltas(&prev_stat, &curr_stat, 1.0);

        let oom_detected = curr_stat.oom_kill > prev_stat.oom_kill;
        
        // PRINT: Display the per-second rates
        print_stat(&curr_stat, cs_per_sec, in_per_sec, minor_per_sec, major_per_sec, us, sy, id, wa, oom_detected);

        // UPDATE: B becomes the new A for next iteration
        prev_stat = curr_stat;
    }

    
}
