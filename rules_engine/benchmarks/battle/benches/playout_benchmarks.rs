use std::time::Duration;

use ai_agents::agent_search;
use ai_data::game_ai::GameAI;
use ai_uct::uct_search;
use battle_mutations::actions::apply_battle_action;
use battle_mutations::card_mutations::deck;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::all_cards::AllCards;
use battle_state::battle::battle_state::{BattleState, LoggingOptions, RequestContext};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardId, CharacterId, HandCardId};
use battle_state::battle::turn_data::TurnData;
use battle_state::battle::turn_history::TurnHistory;
use battle_state::battle_cards::zone::Zone;
use battle_state::battle_player::battle_player_state::{BattlePlayerState, PlayerType};
use battle_state::battle_player::player_map::PlayerMap;
use battle_state::triggers::trigger_state::TriggerState;
use core_data::identifiers::{BattleId, CardName};
use core_data::numerics::{Energy, Points, Spark, TurnId};
use core_data::types::PlayerName;
use criterion::{BatchSize, Criterion, criterion_group};
use game_creation::new_test_battle;
use rand::SeedableRng;
use rand::seq::IteratorRandom;
use rand_xoshiro::Xoshiro256PlusPlus;
use tracing::{Level, subscriber};
use uuid::Uuid;

criterion_group!(playout_benchmarks, random_playout, ai_full, ai_single_threaded, ai_evaluate);

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
                || {
                    new_test_battle::create_and_start(
                        BattleId(Uuid::new_v4()),
                        3141592653589793,
                        PlayerType::Agent(GameAI::RandomAction),
                        PlayerType::Agent(GameAI::RandomAction),
                        RequestContext { logging_options: LoggingOptions::default() },
                    )
                },
                |mut battle| run_battle_until_completion(&mut battle),
                BatchSize::SmallInput,
            );
        });
    });
}

pub fn ai_evaluate(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_evaluate");
    group.measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("ai_evaluate", |b| {
            b.iter_batched(
                benchmark_battle,
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

pub fn ai_single_threaded(c: &mut Criterion) {
    let mut group = c.benchmark_group("ai_single_threaded");
    group.significance_level(0.01).measurement_time(Duration::from_secs(10));
    let error_subscriber = tracing_subscriber::fmt().with_max_level(Level::ERROR).finish();
    subscriber::with_default(error_subscriber, || {
        group.bench_function("ai_single_threaded", |b| {
            b.iter_batched(
                benchmark_battle,
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
                benchmark_battle,
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
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::One,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 1,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 2,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 3,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 4,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::One,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 5,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 6,
            name: CardName::TestDissolve,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 7,
            name: CardName::TestDissolve,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 8,
            name: CardName::TestDissolve,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 9,
            name: CardName::TestCounterspell,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 10,
            name: CardName::TestCounterspell,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 11,
            name: CardName::TestCounterspell,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 12,
            name: CardName::TestCounterspellUnlessPays,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 13,
            name: CardName::TestCounterspellUnlessPays,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 14,
            name: CardName::TestCounterspellUnlessPays,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 15,
            name: CardName::TestVariableEnergyDraw,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 16,
            name: CardName::TestVariableEnergyDraw,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 17,
            name: CardName::TestVariableEnergyDraw,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        // Player Two
        BenchmarkCardSpec {
            id: 18,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 19,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::Two,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 20,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 21,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 22,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 23,
            name: CardName::TestVanillaCharacter,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 24,
            name: CardName::TestDissolve,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 25,
            name: CardName::TestDissolve,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 26,
            name: CardName::TestDissolve,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 27,
            name: CardName::TestCounterspell,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 28,
            name: CardName::TestCounterspell,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 29,
            name: CardName::TestCounterspell,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 30,
            name: CardName::TestCounterspellUnlessPays,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 31,
            name: CardName::TestCounterspellUnlessPays,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 32,
            name: CardName::TestCounterspellUnlessPays,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 33,
            name: CardName::TestVariableEnergyDraw,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 34,
            name: CardName::TestVariableEnergyDraw,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 35,
            name: CardName::TestVariableEnergyDraw,
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
        triggers: TriggerState::default(),
        activated_abilities: PlayerMap::default(),
        animations: None,
        tracing: None,
        action_history: None,
        turn_history: TurnHistory::default(),
        request_context: RequestContext { logging_options: LoggingOptions::default() },
    };

    deck::add_cards(
        &mut battle,
        PlayerName::One,
        card_specs
            .iter()
            .filter(|spec| spec.owner == PlayerName::One)
            .map(|spec| spec.name)
            .collect(),
    );
    deck::add_cards(
        &mut battle,
        PlayerName::Two,
        card_specs
            .iter()
            .filter(|spec| spec.owner == PlayerName::Two)
            .map(|spec| spec.name)
            .collect(),
    );

    for spec in &card_specs {
        let card_id = CardId(spec.id);
        if spec.zone != Zone::Deck {
            battle.cards.move_card(spec.owner, card_id, Zone::Deck, spec.zone);
        }

        if spec.zone == Zone::Battlefield {
            let character_id = CharacterId(card_id);
            let card_name = card::get(&battle, character_id).name;
            if card_name == CardName::TestVanillaCharacter {
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

    let legal_actions_vec: Vec<BattleAction> = legal.all().clone();
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
