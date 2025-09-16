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

**Valgrind is required** for IAI benchmarks. It is only available on Linux.

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

### Performance Notes

- IAI benchmarks can take significantly longer than regular benchmarks (several minutes)
- The first run may take longer due to compilation and setup
- Results are deterministic and should be consistent across runs on the same machine

## Reference Output

Below is sample output from running `just bench-iai` on the development environment:

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

Generally, the "bench_core11_search_action_candidate" benchmark is the source of truth for performance, and
we want to look at the "Estimated Cycles" to get an idea of how long it took to run. This benchmark should
report at most around 10 million (10,000,000) cycles.

Anything over a 10% increase to the estimated cycles here is a major issue which should be flagged.
