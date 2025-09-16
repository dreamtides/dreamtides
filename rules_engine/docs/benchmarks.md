# Benchmarks

This document provides instructions for running benchmarks in the Dreamtides rules engine repository, with a focus on the IAI (Instruction-level benchmarking) system that provides detailed performance analysis using Valgrind's Callgrind tool.

## Overview

The Dreamtides rules engine includes several types of benchmarks:

- **IAI Benchmarks**: Instruction-level performance analysis using Valgrind Callgrind
- **Criterion Benchmarks**: Statistical performance benchmarks with timing analysis

This document focuses on the IAI benchmarks, which provide deterministic, instruction-level performance measurements that are ideal for detecting performance regressions and understanding code efficiency at a low level.

## Prerequisites

### 1. Rust Toolchain

Ensure you have a working Rust installation:

```bash
rustc --version
cargo --version
```

If Rust is not installed, install it using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Just Command Runner

The project uses `just` for automation. Install it if not available:

```bash
cargo install just
```

### 3. Valgrind Installation

**Valgrind is required** for IAI benchmarks. The installation method depends on your operating system:

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install valgrind
```

#### CentOS/RHEL/Fedora
```bash
# CentOS/RHEL
sudo yum install valgrind

# Fedora
sudo dnf install valgrind
```

#### macOS
```bash
# Using Homebrew
brew install valgrind
```

#### Windows
Valgrind does not natively support Windows. For Windows users, consider:
- Using Windows Subsystem for Linux (WSL)
- Using Docker with a Linux container
- Running benchmarks on a Linux VM

#### Verify Valgrind Installation
```bash
valgrind --version
```

You should see output similar to:
```
valgrind-3.22.0
```

### 4. IAI Callgrind Runner

Install the IAI Callgrind runner tool:

```bash
cargo install iai-callgrind-runner
```

## Running IAI Benchmarks

### Basic Usage

From any directory within the project, run:

```bash
just bench-iai
```

This command executes:
```bash
cargo bench --manifest-path rules_engine/Cargo.toml iai_benchmarks
```

### What the Benchmark Measures

The IAI benchmarks in this project measure the performance of core AI decision-making functions:

- **bench_core11_evaluate**: Evaluates game state using the Monte Carlo Tree Search algorithm
- **bench_core11_search_action_candidate**: Searches for the best action candidate in a game state

### Understanding the Output

IAI benchmarks provide several key metrics:

- **Instructions**: Total number of CPU instructions executed
- **L1 Hits**: Level 1 cache hits
- **LL Hits**: Last Level cache hits  
- **RAM Hits**: Memory accesses that went to RAM
- **Total read+write**: Total memory operations
- **Estimated Cycles**: Estimated CPU cycles (approximate timing)

Lower numbers generally indicate better performance, though the specific metric to focus on depends on your optimization goals.

## Troubleshooting

### Common Issues

1. **"valgrind: command not found"**
   - Install Valgrind using the instructions above for your operating system

2. **"Permission denied" or privilege errors**
   - Ensure you have sufficient permissions to run Valgrind
   - On some systems, you may need to run with elevated privileges

3. **"No benchmarks found"**
   - Ensure you're running from the project root directory
   - Verify the benchmark code exists in `rules_engine/benchmarks/battle/benches/iai_benchmarks.rs`

4. **Compilation errors**
   - Run `just build` first to ensure the project builds successfully
   - Check that all dependencies are properly installed

### Performance Notes

- IAI benchmarks can take significantly longer than regular benchmarks (several minutes)
- The first run may take longer due to compilation and setup
- Results are deterministic and should be consistent across runs on the same machine

## Reference Output

Below is sample output from running `just bench-iai` on the development environment:

### First Run (No Baseline)
```
cargo bench --manifest-path rules_engine/Cargo.toml iai_benchmarks
    Finished `bench` profile [optimized] target(s) in 0.34s
     Running benches/iai_benchmarks.rs (rules_engine/target/release/deps/iai_benchmarks-c84f1495fe72e2f7)
iai_benchmarks::bench_group::bench_core11_evaluate eval:benchmark_battles :: core_11_battle :: generate_co...
  Instructions:                      283968|N/A                  (*********)
  L1 Hits:                           410892|N/A                  (*********)
  LL Hits:                             6080|N/A                  (*********)
  RAM Hits:                            1366|N/A                  (*********)
  Total read+write:                  418338|N/A                  (*********)
  Estimated Cycles:                  489102|N/A                  (*********)
iai_benchmarks::bench_group::bench_core11_search_action_candidate eval:benchmark_battles :: core_11_battle :: generate_co...
  Instructions:                     6466603|N/A                  (*********)
  L1 Hits:                          9257751|N/A                  (*********)
  LL Hits:                            97169|N/A                  (*********)
  RAM Hits:                           14496|N/A                  (*********)
  Total read+write:                 9369416|N/A                  (*********)
  Estimated Cycles:                10250956|N/A                  (*********)

Iai-Callgrind result: Ok. 2 without regressions; 0 regressed; 2 benchmarks finished in 4.21421s
```

### Subsequent Run (With Regression Detection)
```
cargo bench --manifest-path rules_engine/Cargo.toml iai_benchmarks
    Finished `bench` profile [optimized] target(s) in 0.19s
     Running benches/iai_benchmarks.rs (rules_engine/target/release/deps/iai_benchmarks-c84f1495fe72e2f7)
iai_benchmarks::bench_group::bench_core11_evaluate eval:benchmark_battles :: core_11_battle :: generate_co...
  Instructions:                      283968|283968               (No change)
  L1 Hits:                           410892|410892               (No change)
  LL Hits:                             6080|6080                 (No change)
  RAM Hits:                            1366|1366                 (No change)
  Total read+write:                  418338|418338               (No change)
  Estimated Cycles:                  489102|489102               (No change)
iai_benchmarks::bench_group::bench_core11_search_action_candidate eval:benchmark_battles :: core_11_battle :: generate_co...
  Instructions:                     6466603|6466603              (No change)
  L1 Hits:                          9257752|9257751              (+0.00001%) [+1.00000x]
  LL Hits:                            97168|97169                (-0.00103%) [-1.00001x]
  RAM Hits:                           14496|14496                (No change)
  Total read+write:                 9369416|9369416              (No change)
  Estimated Cycles:                10250952|10250956             (-0.00004%) [-1.00000x]

Iai-Callgrind result: Ok. 2 without regressions; 0 regressed; 2 benchmarks finished in 4.23631s
```

### Interpreting the Results

From the reference output above:

- **bench_core11_evaluate**: This function executes ~284K instructions with good cache performance (relatively few RAM hits)
- **bench_core11_search_action_candidate**: This is a more complex function executing ~6.5M instructions, indicating it performs significantly more computation

#### Output Format Explanation

Each metric shows: `current_value|previous_value (change)`

- **"N/A"** values indicate this is the first run with no baseline for comparison
- **"(No change)"** indicates identical performance to the baseline
- **Percentage changes** like "(+0.00001%)" show very small variations, which are normal
- **"[+1.00000x]"** indicates the multiplicative factor of change

The very small variations seen in the second run (changes of 0.001% or less) are typical and indicate the benchmarks are highly deterministic and stable.

## Integration with Development Workflow

### Performance Regression Detection

Run IAI benchmarks regularly to detect performance regressions:

```bash
# Before making changes
just bench-iai

# After making changes
just bench-iai
```

Compare the instruction counts and other metrics to identify any significant changes in performance.

### Continuous Integration

IAI benchmarks can be integrated into CI/CD pipelines to automatically detect performance regressions. However, note that:

- Valgrind must be available in the CI environment
- Results may vary slightly between different machines
- Consider setting up dedicated benchmark runners for consistent results

## Additional Resources

- [IAI Callgrind Documentation](https://iai-callgrind.github.io/iai-callgrind/latest/html/)
- [IAI Callgrind Prerequisites](https://iai-callgrind.github.io/iai-callgrind/latest/html/installation/prerequisites.html)
- [Valgrind Callgrind Manual](https://valgrind.org/docs/manual/cl-manual.html)
- [Project Environment Setup](./environment_setup.md)