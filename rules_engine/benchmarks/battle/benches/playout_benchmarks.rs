use std::time::Duration;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use ai_uct::uct_search;
use benchmark_battles::{core_11_battle, starting_5_battle};
use core_data::types::PlayerName;
use criterion::{BatchSize, Criterion, criterion_group};
use tracing::{Level, subscriber};

criterion_group!(playout_benchmarks, ai_full, ai_starting_5, ai_core_11, ai_evaluate);

pub fn ai_evaluate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_evaluate");
    group.measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("ai_evaluate", |b| {
            b.iter_batched(
                starting_5_battle::benchmark_battle,
                |mut battle| {
                    criterion::black_box(uct_search::evaluate_for_benchmarking(
                        &mut battle,
                        PlayerName::One,
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}

pub fn ai_starting_5(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_starting_5");
    group.significance_level(0.01).measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("ai_starting_5", |b| {
            b.iter_batched(
                starting_5_battle::benchmark_battle,
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

pub fn ai_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_full");
    group.significance_level(0.01).sample_size(100);
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("ai_full", |b| {
            b.iter_batched(
                starting_5_battle::benchmark_battle,
                |battle| {
                    criterion::black_box(agent_search::select_action_unchecked(
                        &battle,
                        PlayerName::One,
                        &GameAI::MonteCarlo(50),
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}
