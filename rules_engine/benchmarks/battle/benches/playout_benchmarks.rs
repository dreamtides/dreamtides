use std::time::Duration;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::all_cards::AllCards;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardId, CharacterId, HandCardId};
use battle_state::battle::turn_data::TurnData;
use battle_state::battle_cards::zone::Zone;
use battle_state::battle_player::battle_player_state::{BattlePlayerState, PlayerType};
use battle_state::battle_player::player_map::PlayerMap;
use core_data::identifiers::{BattleId, CardName};
use core_data::numerics::{Energy, Points, Spark, TurnId};
use core_data::types::PlayerName;
use criterion::{criterion_group, BatchSize, Criterion};
use game_creation::new_test_battle;
use rand::seq::IteratorRandom;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tracing::{subscriber, Level};
use tracing_macros::write_tracing_event;
use uuid::Uuid;

criterion_group!(
    playout_benchmarks,
    random_playout,
    uct1_first_action,
    uct_1k_action,
    uct_50k_action,
    uct_single_threaded
);

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
                        &GameAI::OldUct1MaxIterations(1000),
                        false,
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}

pub fn uct_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct_single_threaded");
    group.significance_level(0.01).sample_size(500).measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("uct_single_threaded", |b| {
            b.iter_batched(
                benchmark_battle,
                |battle| {
                    criterion::black_box(agent_search::select_action_unchecked(
                        &battle,
                        PlayerName::One,
                        &GameAI::Uct1SingleThreaded(100),
                        false,
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
                benchmark_battle,
                |battle| {
                    criterion::black_box(agent_search::select_action_unchecked(
                        &battle,
                        PlayerName::One,
                        &GameAI::Uct1(1000),
                        false,
                    ))
                },
                BatchSize::SmallInput,
            );
        });
    });
}

pub fn uct_50k_action(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct_50k_action");
    group.significance_level(0.01).measurement_time(Duration::from_secs(100));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("uct_50k_action", |b| {
            b.iter_batched(
                benchmark_battle,
                |battle| {
                    criterion::black_box(agent_search::select_action_unchecked(
                        &battle,
                        PlayerName::One,
                        &GameAI::Uct1(50_000),
                        false,
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

struct BenchmarkCardSpec {
    id: usize,
    name: CardName,
    owner: PlayerName,
    zone: Zone,
}

fn benchmark_battle() -> BattleState {
    let card_specs = vec![
        // Player One
        BenchmarkCardSpec {
            id: 0,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::One,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 1,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 2,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 3,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 4,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::One,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 5,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 6,
            name: CardName::Immolate,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 7,
            name: CardName::Immolate,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 8,
            name: CardName::Immolate,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 9,
            name: CardName::Abolish,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 10,
            name: CardName::Abolish,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 11,
            name: CardName::Abolish,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 12,
            name: CardName::RippleOfDefiance,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 13,
            name: CardName::RippleOfDefiance,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 14,
            name: CardName::RippleOfDefiance,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 15,
            name: CardName::Dreamscatter,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 16,
            name: CardName::Dreamscatter,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 17,
            name: CardName::Dreamscatter,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        // Player Two
        BenchmarkCardSpec {
            id: 18,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 19,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::Two,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 20,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 21,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 22,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 23,
            name: CardName::MinstrelOfFallingLight,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 24,
            name: CardName::Immolate,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 25,
            name: CardName::Immolate,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 26,
            name: CardName::Immolate,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 27,
            name: CardName::Abolish,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 28,
            name: CardName::Abolish,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 29,
            name: CardName::Abolish,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 30,
            name: CardName::RippleOfDefiance,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 31,
            name: CardName::RippleOfDefiance,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 32,
            name: CardName::RippleOfDefiance,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 33,
            name: CardName::Dreamscatter,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 34,
            name: CardName::Dreamscatter,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 35,
            name: CardName::Dreamscatter,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
    ];

    let seed = 12345678912345;
    let mut battle = BattleState {
        id: BattleId(Uuid::new_v4()),
        cards: AllCards::default(),
        players: PlayerMap {
            one: BattlePlayerState {
                player_type: PlayerType::Agent(GameAI::AlwaysPanic),
                points: Points(15),
                current_energy: Energy(6),
                produced_energy: Energy(6),
                spark_bonus: Spark(0),
            },
            two: BattlePlayerState {
                player_type: PlayerType::Agent(GameAI::AlwaysPanic),
                points: Points(0),
                current_energy: Energy(5),
                produced_energy: Energy(5),
                spark_bonus: Spark(0),
            },
        },
        status: BattleStatus::Playing,
        stack_priority: None,
        turn: TurnData { active_player: PlayerName::One, turn_id: TurnId(9) },
        phase: BattleTurnPhase::Main,
        seed,
        rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        prompt: None,
        animations: None,
        tracing: None,
        history: None,
    };

    for spec in &card_specs {
        assert_eq!(battle.cards.all_cards().count(), spec.id, "Card ID mismatch during creation");
        battle.cards.create_cards_in_deck(spec.owner, vec![spec.name]);
    }

    for spec in &card_specs {
        let card_id = CardId(spec.id);
        if spec.zone != Zone::Deck {
            battle.cards.move_card(spec.owner, card_id, Zone::Deck, spec.zone);
        }

        if spec.zone == Zone::Battlefield {
            let character_id = CharacterId(card_id);
            let card_name = battle.cards.name(character_id);
            if card_name == CardName::MinstrelOfFallingLight {
                if let Some(char_state) =
                    battle.cards.battlefield_state_mut(spec.owner).get_mut(&character_id)
                {
                    char_state.spark = Spark(5);
                }
            }
        }
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

    let legal_actions_vec: Vec<BattleAction> = legal.all().to_vec();
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

    battle
}
