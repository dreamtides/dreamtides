# Benchmarks

The rules engine includes two benchmark systems for measuring AI search
performance: Criterion benchmarks for statistical timing analysis and IAI
benchmarks for deterministic instruction-level profiling via Valgrind Callgrind.

## Benchmark Types

**Criterion benchmarks** measure wall-clock timing with statistical analysis.
They run on any platform and are useful for quick local performance checks.

**IAI benchmarks** count CPU instructions, cache hits, and estimated cycles
using Valgrind Callgrind. Results are deterministic across runs on the same
machine, making them ideal for detecting regressions. IAI benchmarks require
Linux (Valgrind is Linux-only).

## What Gets Benchmarked

All benchmarks exercise the AI Monte Carlo Tree Search against a fixed game
state generated from the "core 11" battle scenario. The benchmark source is at
`rules_engine/benchmarks/battle/benches/iai_benchmarks.rs`. The two primary
benchmarks in the active benchmark group are:

- **bench_core11_evaluate**: Runs a single MCTS evaluation of a game state
- **bench_core11_search_action_candidate**: Runs a full action candidate search

The search action candidate benchmark is the primary performance indicator.

## Running Benchmarks

### Criterion (any platform)

- `just benchmark` — run all Criterion benchmarks
- `just bench-c11` — core 11 AI benchmark only
- `just bench-evaluate` — evaluation benchmark only
- `just bench-s5` — starting 5 AI benchmark
- `just bench-full` — full AI benchmark
- `just bench-parser` — parser benchmarks (separate package)
- `just bench-lattice` — lattice benchmarks

### IAI (Linux only)

- `just bench-iai` — run IAI benchmarks directly (requires Valgrind and
  iai-callgrind-runner installed locally on Linux)
- `just iai` — run IAI benchmarks inside a Docker container (works from macOS or
  any platform with Docker). Uses `scripts/benchmarking/benchmark_on_linux.py`
  to build an Ubuntu 24.04 container with Valgrind and iai-callgrind-runner,
  rsync the source, and execute benchmarks.

### IAI Prerequisites

Running `just bench-iai` directly requires:

- Valgrind installed on the Linux host
- The iai-callgrind-runner binary: install with
  `cargo install --locked --version 0.16.1 iai-callgrind-runner`

Running `just iai` only requires Docker.

## Interpreting IAI Output

IAI benchmarks report several metrics per benchmark function:

- **Instructions**: total CPU instructions executed
- **L1 Hits / LL Hits / RAM Hits**: cache hierarchy access counts
- **Total read+write**: total memory operations
- **Estimated Cycles**: approximate CPU cycles (the primary metric)

The `bench_core11_search_action_candidate` Estimated Cycles value is the
source-of-truth performance number. It should stay at or below approximately 10
million cycles. Any increase over 10% is a significant regression that should be
investigated.

## Profiling

- `just samply-battle-benchmark <name>` — profile a Criterion benchmark binary
  with Samply (e.g. `just samply-battle-benchmark ai_core_11`)
- `just most-called-functions` — analyze the most-called functions in a
  benchmark using callgrind data

## Crate Layout

Benchmark crates live under `rules_engine/benchmarks/`:

- `battle` — IAI and Criterion benchmarks for AI search
- `benchmark_battles` — shared battle state fixtures (e.g. the core 11 scenario)
- `parser_benchmarks` — Criterion benchmarks for the parser

Regenerate the core 11 battle fixture with `just regenerate-test-battles`.
