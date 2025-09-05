/// Benchmark group for AI playout performance.
///
/// Building just this benchmark target (without running it) using Cargo:
///
/// ```bash
/// # Build only this bench target
/// cargo build --manifest-path benchmarks/battle/Cargo.toml -p battle_benchmarks --bench playout_benchmarks
///
/// # (Alternative) build all benchmarks in the package
/// cargo build --manifest-path benchmarks/battle/Cargo.toml -p battle_benchmarks --benches
///
/// # Run only this benchmark via Criterion harness
/// cargo bench --manifest-path benchmarks/battle/Cargo.toml -p battle_benchmarks --bench playout_benchmarks -- ai_core_11/ai_core_11
///
/// # With LLVM source-based coverage instrumentation (example)
/// RUSTFLAGS="-Cinstrument-coverage -Ccodegen-units=1 -Clink-dead-code -Coverflow-checks=off" \\
///    cargo +nightly build --release --manifest-path benchmarks/battle/Cargo.toml -p battle_benchmarks --bench playout_benchmarks
/// ```
///
/// The actual bench function defined here is `ai_core_11` and is grouped under
/// the Criterion group name `ai_core_11` inside the bench target
/// `playout_benchmarks`.
use std::time::Duration;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use benchmark_battles::core_11_battle;
use core_data::types::PlayerName;
use criterion::{BatchSize, Criterion, criterion_group};
use tracing::{Level, subscriber};

criterion_group!(playout_benchmarks, ai_core_11);

pub fn ai_core_11(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_core_11");
    group.significance_level(0.01).measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("ai_core_11", |b| {
            b.iter_batched(
                core_11_battle::generate_core_11_battle,
                |battle| {
                    criterion::black_box(agent_search::select_action_unchecked(
                        &battle,
                        PlayerName::One,
                        &GameAI::MonteCarloSingleThreaded(1),
                        Some(1.0),
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}
