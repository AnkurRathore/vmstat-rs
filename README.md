# vmstat-rs
Inspired by Brendan Gregg’s 'Systems Performance', this tool aims to provide a modern, safe, and high-performance alternative to traditional procps tools using Rust.

A Rust implementation of the classic Linux `vmstat` utility with real-time context switch monitoring and alerting. This educational tool demonstrates low-level system programming by manually parsing `/proc` filesystem files without relying on high-level crates like `procfs`.

##  What Does This Tool Do?

**vmstat-rs** monitors your Linux system's vital statistics in real-time and alerts you when your CPU is thrashing. Every second, it displays:

- **Process states** (running, blocked)
- **Memory usage** (free, buffers, cache, swap)
- **CPU utilization** (user, system, idle, I/O wait)
- **Context switches per second** ⚡ **← THE STAR OF THE SHOW**
- **Interrupts per second**

When context switches spike above **5,000 per second**, the entire line turns **RED** with a warning message, indicating your CPU is spending more time switching between tasks than doing actual work.

## Why Monitor Context Switches?

### What is a Context Switch?

A context switch occurs when the CPU switches from executing one process/thread to another. The kernel must:
1. Save the current process state (registers, program counter, stack pointer)
2. Load the next process state
3. Resume execution

This takes time—precious CPU cycles that could be doing actual work.

### When Do Context Switches Become a Problem?

**Normal rates:** 1,000-3,000 context switches/second on a typical system  
**Warning zone:** 5,000-10,000 context switches/second  
**Critical:** 10,000+ context switches/second

### Signs of CPU Thrashing

When you see **RED alerts** from this tool, it means:
- Too many processes competing for CPU time
- CPU spending more time switching than executing
- System responsiveness degrading
- Applications starving for CPU resources

**Common causes:**
- Runaway processes spawning too many threads
- Memory pressure causing excessive swapping
- Poorly optimized multi-threaded applications
- System overload with too many concurrent processes

## How It Works: The "Hard Way"

This tool intentionally avoids using convenience crates to demonstrate raw `/proc` filesystem parsing:

### The Snapshot-Delta Pattern

The kernel provides **cumulative counters since boot** in `/proc/stat`. To get per-second rates, we use the snapshot-delta pattern:

```rust
// SNAPSHOT A: Read initial values
let prev_stat = parse_vmstat()?;
//   context_switches: 45,823,910

thread::sleep(Duration::from_secs(1));

// SNAPSHOT B: Read new values  
let curr_stat = parse_vmstat()?;
//   context_switches: 45,829,142

// CALCULATE: (B - A) = rate per second
let cs_per_sec = 45,829,142 - 45,823,910 = 5,232/sec  //  RED ALERT!
```

### Manual /proc Parsing

**Instead of this (easy way):**
```rust
use procfs::CpuStat;
let stats = procfs::read_cpu_stat()?;
```

**We do this (educational way):**
```rust
fn parse_stat() -> Result<...> {
    let content = fs::read_to_string("/proc/stat")?;
    
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        match parts[0] {
            "ctxt" => {
                // Found it! Parse the counter
                context_switches = parts[1].parse().unwrap_or(0);
            }
            "cpu" => {
                // Parse: user, nice, system, idle, iowait...
                user = parts[1].parse().unwrap_or(0);
                system = parts[3].parse().unwrap_or(0);
                // etc.
            }
            _ => {}
        }
    }
}
```

### What We Parse

**`/proc/stat`** - CPU and system statistics
```
cpu  74608 2520 24433 1117073 6176 4054 0 0 0 0
cpu0 37088 1260 12341 558482 3088 2027 0 0 0 0
cpu1 37520 1260 12092 558591 3088 2027 0 0 0 0
intr 4122258 20 9 0 0 0 0 0 0 1 ...
ctxt 45829142          ← WE WANT THIS NUMBER
btime 1704931200
processes 12483
procs_running 2        ← AND THIS
procs_blocked 0        ← AND THIS
```

**`/proc/meminfo`** - Memory statistics
```
MemTotal:       16384000 kB
MemFree:         8192000 kB
Buffers:          512000 kB
Cached:          2048000 kB
SwapTotal:       8192000 kB
SwapFree:        8192000 kB
```

## Output Format

```
vmstat-rs - Context Switch Monitor
Threshold: 5000+ context switches/sec will be highlighted in RED
Press Ctrl+C to exit

 r  b       swpd       free   buff  cache   si   so   bi   bo  in  cs us sy id wa st
 2  0       1024    4521344  51200 812032    0    0    0    0 120 234  5  2 92  1  0
 1  0       1024    4518912  51200 812096    0    0    0    0 145 312  7  3 89  1  0
 3  0       1024    4516480  51232 812128    0    0    0    0 189 6234 12  8 78  2  0
 Context switches: 6234/sec - CPU thrashing detected!    ← RED ALERT!
```

### Column Reference

| Column | Description | Unit |
|--------|-------------|------|
| **r** | Runnable processes (waiting for CPU) | count |
| **b** | Blocked processes (waiting for I/O) | count |
| **swpd** | Virtual memory used | KB |
| **free** | Free physical memory | KB |
| **buff** | Memory used as buffers | KB |
| **cache** | Memory used as cache | KB |
| **si** | Memory swapped in from disk | KB/s |
| **so** | Memory swapped out to disk | KB/s |
| **bi** | Blocks received from devices | blocks/s |
| **bo** | Blocks sent to devices | blocks/s |
| **in** | Interrupts per second | /s |
| **cs** | Context switches per second ⚡ | /s |
| **us** | CPU time in user mode | % |
| **sy** | CPU time in system/kernel mode | % |
| **id** | CPU idle time | % |
| **wa** | CPU waiting for I/O | % |
| **st** | Stolen time (virtualization) | % |

##  Installation & Usage

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Linux system** - Requires `/proc` filesystem
- **Root/sudo access** - Needed to read some `/proc` files

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/vmstat-rs.git
cd vmstat-rs

# Build in release mode (optimized)
cargo build --release

# Run the binary
sudo ./target/release/vmstat-rs
```

### Quick Run (Development)

```bash
# Run directly with Cargo
sudo cargo run --release
```

### Why sudo?

Some systems restrict read access to certain `/proc` files. If you get permission errors, run with `sudo`.

## Learning Objectives

This project teaches:

✅ **Low-level Linux system programming**  
✅ **Manual file parsing without helper libraries**  
✅ **Understanding /proc filesystem structure**  
✅ **Delta calculation for rate metrics**  
✅ **Real-time monitoring patterns**  
✅ **Error handling with Result types**  
✅ **Performance monitoring concepts**

## Understanding the Code

### Key Functions

**`parse_stat()`** - Reads `/proc/stat` line by line
```rust
for line in content.lines() {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts[0] {
        "ctxt" => context_switches = parts[1].parse()?,
        "cpu" => /* parse CPU ticks */,
        // ...
    }
}
```

**`calculate_deltas()`** - Converts cumulative counters to rates
```rust
let cs_delta = curr.context_switches - prev.context_switches;
let cs_per_sec = cs_delta / interval_secs;
```

**`print_stat()`** - Formats output and highlights alerts
```rust
if cs_per_sec > CTXT_THRESHOLD {
    println!("{}", line.red().bold());  // 🚨 RED ALERT
}
```

### The Main Loop

```rust
let mut prev_stat = parse_vmstat()?;      // Initial snapshot

loop {
    thread::sleep(Duration::from_secs(1)); // Wait 1 second
    let curr_stat = parse_vmstat()?;       // New snapshot
    
    let deltas = calculate_deltas(&prev_stat, &curr_stat, 1.0);
    print_stat(&curr_stat, deltas);
    
    prev_stat = curr_stat;                 // Update for next iteration
}
```

## Testing & Experimentation

### Trigger a Context Switch Spike

**Stress test with many short-lived processes:**
```bash
# Terminal 1: Run vmstat-rs
sudo cargo run --release

# Terminal 2: Create artificial load
for i in {1..1000}; do sleep 0.001 & done
```

You should see the context switch counter climb and turn RED!

### Compare with Real vmstat

```bash
# Terminal 1
sudo ./target/release/vmstat-rs

# Terminal 2
vmstat 1
```

The numbers should match closely (minor differences due to timing).

##Troubleshooting

**Error: Permission denied reading /proc/stat**
- Solution: Run with `sudo`

**Numbers seem off compared to vmstat**
- Expected: Minor timing differences are normal
- Check: Ensure you're comparing the same columns

**Build errors with colored crate**
- Solution: `cargo clean && cargo build --release`

## Roadmap & Ideas

- [ ] **Command-line arguments** - Custom threshold: `vmstat-rs --threshold 10000`
- [ ] **Delay/count mode** - Like real vmstat: `vmstat-rs 2 10` (2s interval, 10 iterations)
- [ ] **Historical tracking** - Store and graph context switch trends
- [ ] **Multiple alert levels** - Yellow warning, red critical
- [ ] **Disk I/O monitoring** - Parse `/proc/diskstats` for bi/bo metrics
- [ ] **Export formats** - CSV/JSON output for external analysis
- [ ] **Process attribution** - Which process caused the spike?
- [ ] **Memory pressure alerts** - Swap activity warnings



## 📖 Further Reading

**Understanding /proc filesystem:**
- [Linux /proc documentation](https://www.kernel.org/doc/Documentation/filesystems/proc.txt)
- [man proc](https://man7.org/linux/man-pages/man5/proc.5.html)

**Context switching deep dive:**
- [What is a Context Switch?](https://en.wikipedia.org/wiki/Context_switch)
- [Linux Performance Analysis](https://www.brendangregg.com/linuxperf.html)

**Rust systems programming:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust by Example - File I/O](https://doc.rust-lang.org/rust-by-example/std_misc/file.html)


## Acknowledgments

- Inspired by the classic `vmstat` utility (part of procps-ng)
- Built as an educational tool for understanding Linux system internals
- Context switch monitoring helps identify real-world performance issues


## Support

If you found this project helpful for learning systems programming, please consider:
- Giving it a ⭐ on GitHub
- Sharing it with others learning Rust
- Contributing improvements or documentation

---

