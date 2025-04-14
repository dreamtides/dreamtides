use std::time::Duration;

use actions::battle_actions;
use agent_data::agent::Agent;
use agents::agent_search;
use battle_data::battle::battle_data::BattleData;
use battle_queries::legal_action_queries::legal_actions;
use core_data::identifiers::BattleId;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use game_creation::new_test_battle;
use tracing::{subscriber, Level};
use uuid::Uuid;

criterion_main!(benches);
criterion_group!(benches, random_playout);

pub fn random_playout(c: &mut Criterion) {
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
                || new_test_battle::create_and_start(BattleId(Uuid::new_v4()), Agent::RandomAction),
                |mut battle| run_battle_until_completion(&mut battle),
                BatchSize::SmallInput,
            );
        });
    });
}

fn run_battle_until_completion(battle: &mut BattleData) {
    while let Some(player) = legal_actions::next_to_act(battle) {
        let Some(agent) = battle.player(player).agent.as_ref() else {
            panic!("Player has no agent");
        };
        let action = agent_search::select_action(battle, player, agent);
        battle_actions::execute(battle, player, action);
    }
}
