mod stats;
use colored::*;
use std::thread;
use std::time::Duration;
use stats::VmStat;

const CTXT_THRESHOLD: u64 = 5000;


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
    let mut prev_stat = stats::parse_vmstat()?;
    
    // Print first line with zeros for rates (no previous snapshot to compare)
    print_stat(&prev_stat, 0, 0, 0, 0, 0.0, 0.0, 100.0, 0.0, false);

    // Main monitoring loop
    loop {
        // SLEEP: Wait 1 second
        thread::sleep(Duration::from_secs(1));
        
        // SNAPSHOT B: Take new reading
        let curr_stat = stats::parse_vmstat()?;
        
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
