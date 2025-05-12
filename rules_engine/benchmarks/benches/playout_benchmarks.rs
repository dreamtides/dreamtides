use std::time::Duration;

use ai_data::game_ai::GameAI;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::BattleId;
use criterion::{criterion_group, BatchSize, Criterion};
use game_creation::new_test_battle;
use rand::seq::IteratorRandom;
use tracing::{subscriber, Level};
use tracing_macros::write_tracing_event;
use uuid::Uuid;

criterion_group!(playout_benchmarks, random_playout);

pub fn random_playout(c: &mut Criterion) {
    write_tracing_event::clear_log_file();
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

fn run_battle_until_completion(battle: &mut BattleState) {
    while let Some(player) = legal_actions::next_to_act(battle) {
        let actions = legal_actions::compute(battle, player).all();
        let action = *actions.iter().choose(&mut battle.rng).unwrap();
        apply_battle_action::execute(battle, player, action);
    }
}
