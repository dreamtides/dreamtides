use std::time::Duration;

use actions::battle_actions;
use ai_agents_old::agent_search;
use ai_data::game_ai::GameAI;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_player::player_data::PlayerType;
use battle_queries_old::legal_action_queries::legal_actions;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use criterion::{criterion_group, BatchSize, Criterion};
use game_creation_old::new_test_battle;
use tracing::{subscriber, Level};
use uuid::Uuid;

criterion_group!(old_playout_benchmarks, old_random_playout, old_uct1_first_action);

pub fn old_random_playout(c: &mut Criterion) {
    let mut group = c.benchmark_group("random_playout");
    group
        .significance_level(0.01)
        .sample_size(500)
        .noise_threshold(0.03)
        .measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("random_playout", |b| {
            b.iter_batched(
                || {
                    new_test_battle::create_and_start(
                        BattleId(Uuid::new_v4()),
                        3141592653589793,
                        PlayerType::Agent(GameAI::RandomAction),
                        PlayerType::Agent(GameAI::RandomAction),
                    )
                },
                |mut battle| run_battle_until_completion(&mut battle),
                BatchSize::SmallInput,
            );
        });
    });
}

pub fn old_uct1_first_action(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct1_first_action");
    group
        .significance_level(0.01)
        .sample_size(500)
        .noise_threshold(0.03)
        .measurement_time(Duration::from_secs(15));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("uct1_first_action", |b| {
            b.iter_batched(
                || {
                    new_test_battle::create_and_start(
                        BattleId(Uuid::new_v4()),
                        3141592653589793,
                        PlayerType::Agent(GameAI::AlwaysPanic),
                        PlayerType::Agent(GameAI::AlwaysPanic),
                    )
                },
                |battle| {
                    criterion::black_box(agent_search::select_action(
                        &battle,
                        PlayerName::One,
                        &GameAI::Uct1MaxIterations(1000),
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}

fn run_battle_until_completion(battle: &mut BattleData) {
    while let Some(player) = legal_actions::next_to_act(battle) {
        let PlayerType::Agent(agent) = &battle.player(player).player_type else {
            panic!("Player has no agent");
        };
        let action = agent_search::select_action(battle, player, agent);
        battle_actions::execute(battle, player, action);
    }
}
