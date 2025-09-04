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
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}
