# 🦀 vmstat-rs: Modern System Observability

A high-performance implementation of the classic `vmstat` utility, written in Rust.

**Why rebuild a classic?**
Standard `vmstat` aggregates all memory page faults into a single `flt` column. In modern high-performance computing (especially AI/ML training), this is insufficient. This tool distinguishes between **Minor Faults** (cheap memory re-mapping) and **Major Faults** (expensive disk I/O), allowing engineers to identify true latency bottlenecks.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
![Platform](https://img.shields.io/badge/platform-linux-lightgrey)

## 🚀 Key Features

*   **Granular Memory Forensics:** Splits `pgfault` into **Minor (min)** and **Major (maj)** faults.
*   **Zero-Dependency Parsing:** Manually parses `/proc/stat`, `/proc/vmstat`, and `/proc/meminfo` for educational transparency.
*   **Context Switch Alerting:** Highlights rows in **RED** when context switches exceed 5,000/sec (CPU thrashing).
*   **JSON Output:** Support for structured logging via `--json` (coming soon).

## 📊 The "Missing Metric": Major vs. Minor Faults

Most developers see "Page Faults" and panic. This tool helps you panic only when necessary.

| Metric | Column | Description | Performance Impact |
| :--- | :--- | :--- | :--- |
| **Minor Fault** | `min` | The data is in RAM but needs a pointer update. | 🟢 **Negligible.** (Microseconds). Common in shared libraries. |
| **Major Fault** | `maj` | The data is on **DISK**. The CPU must stall and wait for I/O. | 🔴 **Critical.** (Milliseconds). This kills AI training throughput. |

If you see `maj` spiking while training a model, your batch size is likely too large for RAM, and you are thrashing swap/disk.

## 🛠️ Installation & Usage

### Prerequisites
*   Linux (Requires `/proc` filesystem)
*   Rust Toolchain

### Quick Start
```bash
# Clone the repo
git clone https://github.com/yourusername/vmstat-rs
cd vmstat-rs

# Run in release mode (for accuracy)
cargo run --release
```

### Sample Output
```bash
r  b   swpd   free   buff  cache   min   maj    in    cs  us  sy  id  wa
 1  0      0  8192M   128M  2048M   120     0   150   200   2   1  97   0
 0  0      0  8190M   128M  2050M   450     0   210   340   5   2  93   0
 2  1   500M  1024M   120M  1000M  2000    85   800  6000  10  40  10  40
⚠️  HIGH IO LATENCY: 85 Major Faults/sec detected!
```

### How It Works (Under the Hood)
This tool avoids high-level abstractions to interact directly with the Linux Kernel ABI.
1. Source: Reads /proc/vmstat (Kernel 2.6+).
2. The Math:
    The kernel provides pgfault (Total) and pgmajfault (Major).
    to calculate: Minor = Total - Major.
3. Rate Calculation:
4. Kernel counters are cumulative (since boot).
    capture Snapshot A, sleep for 1 second, capture Snapshot B.
    Rate = (B - A) / TimeDelta.

### Roadmap

1. Basic CPU/Memory Stats

2. Major/Minor Fault Separation

3. PSI (Pressure Stall Information): Parsing /proc/pressure/cpu for modern stall tracking.

4. Prometheus Exporter: Optional HTTP server to scrape metrics into Grafana.

### References
Built while studying "Systems Performance" by Brendan Gregg.
Chapter 4: Observability Tools
Chapter 7: Memory (Virtual Memory & Paging)
### License
MIT