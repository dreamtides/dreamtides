use std::collections::HashMap;
use std::time::Duration;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardId, CardIdType, HandCardId};
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::{BattleId, CardName};
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use criterion::{criterion_group, BatchSize, Criterion};
use game_creation::new_test_battle;
use rand::seq::IteratorRandom;
use tracing::{subscriber, Level};
use tracing_macros::{panic_with, write_tracing_event};
use uuid::Uuid;

criterion_group!(playout_benchmarks, random_playout, uct1_first_action, uct_1k_action);

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

pub fn uct1_first_action(c: &mut Criterion) {
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
                    criterion::black_box(agent_search::select_action_unchecked(
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

pub fn uct_1k_action(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct_1k_action");
    group.significance_level(0.01).sample_size(500).measurement_time(Duration::from_secs(15));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("uct_1k_action", |b| {
            b.iter_batched(
                || build_benchmark_battle(),
                |battle| {
                    criterion::black_box(agent_search::select_action_unchecked(
                        &battle,
                        PlayerName::One,
                        &GameAI::NewUct(1000),
                    ))
                },
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

fn build_benchmark_battle() -> BattleState {
    let mut battle = new_test_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        12345678912345,
        PlayerType::Agent(GameAI::AlwaysPanic),
        PlayerType::Agent(GameAI::AlwaysPanic),
    );

    let mut player_one_cards = HashMap::new();
    for id in battle.cards.hand(PlayerName::One).iter() {
        let card_name = battle.cards.name(id);
        player_one_cards.insert(id.card_id().0, card_name);
    }

    let mut player_two_cards = HashMap::new();
    for id in battle.cards.hand(PlayerName::Two).iter() {
        let card_name = battle.cards.name(id);
        player_two_cards.insert(id.card_id().0, card_name);
    }

    let mut expected_player_one = HashMap::new();
    expected_player_one.insert(0, CardName::MinstrelOfFallingLight);
    expected_player_one.insert(4, CardName::MinstrelOfFallingLight);
    expected_player_one.insert(6, CardName::Immolate);
    expected_player_one.insert(11, CardName::Abolish);
    expected_player_one.insert(14, CardName::RippleOfDefiance);

    let mut expected_player_two = HashMap::new();
    expected_player_two.insert(19, CardName::MinstrelOfFallingLight);
    expected_player_two.insert(28, CardName::Abolish);
    expected_player_two.insert(30, CardName::RippleOfDefiance);
    expected_player_two.insert(31, CardName::RippleOfDefiance);
    expected_player_two.insert(34, CardName::Dreamscatter);

    assert_eq!(
        player_one_cards, expected_player_one,
        "Player One's hand does not match expected cards"
    );
    assert_eq!(
        player_two_cards, expected_player_two,
        "Player Two's hand does not match expected cards"
    );

    apply_battle_action::execute(
        &mut battle,
        PlayerName::One,
        BattleAction::PlayCardFromHand(HandCardId(CardId(0))),
    );
    apply_battle_action::execute(&mut battle, PlayerName::Two, BattleAction::PassPriority);
    apply_battle_action::execute(&mut battle, PlayerName::One, BattleAction::EndTurn);
    apply_battle_action::execute(&mut battle, PlayerName::Two, BattleAction::StartNextTurn);
    apply_battle_action::execute(
        &mut battle,
        PlayerName::Two,
        BattleAction::PlayCardFromHand(HandCardId(CardId(19))),
    );
    apply_battle_action::execute(&mut battle, PlayerName::One, BattleAction::PassPriority);
    apply_battle_action::execute(&mut battle, PlayerName::Two, BattleAction::EndTurn);
    apply_battle_action::execute(&mut battle, PlayerName::One, BattleAction::StartNextTurn);
    apply_battle_action::execute(
        &mut battle,
        PlayerName::One,
        BattleAction::PlayCardFromHand(HandCardId(CardId(4))),
    );
    apply_battle_action::execute(&mut battle, PlayerName::Two, BattleAction::PassPriority);

    for _ in 0..3 {
        apply_battle_action::execute(&mut battle, PlayerName::One, BattleAction::EndTurn);
        apply_battle_action::execute(&mut battle, PlayerName::Two, BattleAction::StartNextTurn);
        apply_battle_action::execute(&mut battle, PlayerName::Two, BattleAction::EndTurn);
        apply_battle_action::execute(&mut battle, PlayerName::One, BattleAction::StartNextTurn);
    }

    assert_eq!(battle.players.one.current_energy, Energy(6));
    assert_eq!(battle.status, BattleStatus::Playing);
    assert_eq!(battle.phase, BattleTurnPhase::Main);
    assert_eq!(battle.turn.active_player, PlayerName::One);
    assert_eq!(legal_actions::next_to_act(&battle), Some(PlayerName::One));

    let legal = legal_actions::compute(&battle, PlayerName::One);
    assert_eq!(legal.len(), 5);

    let expected_actions = vec![
        BattleAction::EndTurn,
        BattleAction::PlayCardFromHand(HandCardId(CardId(2))),
        BattleAction::PlayCardFromHand(HandCardId(CardId(6))),
        BattleAction::PlayCardFromHand(HandCardId(CardId(7))),
        BattleAction::PlayCardFromHand(HandCardId(CardId(17))),
    ];

    let legal_actions_vec: Vec<BattleAction> = legal.all().iter().cloned().collect();
    assert_eq!(
        legal_actions_vec.len(),
        expected_actions.len(),
        "Number of legal actions doesn't match expected"
    );

    for action in &expected_actions {
        assert!(
            legal_actions_vec.contains(action),
            "Expected action {:?} not found in legal actions {:?}",
            action,
            legal_actions_vec
        );
    }

    for action in &legal_actions_vec {
        assert!(
            expected_actions.contains(action),
            "Unexpected action {:?} found in legal actions",
            action
        );
    }

    panic_with!("Done setting up battle", &battle);

    battle
}
