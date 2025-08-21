use std::collections::VecDeque;
use std::sync::Arc;

use ai_data::game_ai::GameAI;
use battle_mutations::card_mutations::battle_deck;
use battle_queries::battle_card_queries::card;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::all_cards::AllCards;
use battle_state::battle::battle_rules_config::BattleRulesConfig;
use battle_state::battle::battle_state::{BattleState, LoggingOptions, RequestContext};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardId, CharacterId, HandCardId};
use battle_state::battle::turn_data::TurnData;
use battle_state::battle::turn_history::TurnHistory;
use battle_state::battle_cards::ability_state::AbilityState;
use battle_state::battle_cards::zone::Zone;
use battle_state::battle_player::battle_player_state::{
    BattlePlayerState, PlayerType, TestDeckName,
};
use battle_state::battle_player::player_map::PlayerMap;
use battle_state::triggers::trigger_state::TriggerState;
use core_data::identifiers::{BaseCardId, BattleId};
use core_data::numerics::{Energy, Points, Spark, TurnId};
use core_data::types::PlayerName;
use game_creation::new_test_battle;
use quest_state::quest::card_descriptor;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tabula_ids::test_card;
use uuid::Uuid;

struct BenchmarkCardSpec {
    id: usize,
    name: BaseCardId,
    owner: PlayerName,
    zone: Zone,
}

pub fn benchmark_battle() -> BattleState {
    let card_specs = vec![
        BenchmarkCardSpec {
            id: 0,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::One,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 1,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 2,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 3,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 4,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::One,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 5,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 6,
            name: test_card::TEST_DISSOLVE,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 7,
            name: test_card::TEST_DISSOLVE,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 8,
            name: test_card::TEST_DISSOLVE,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 9,
            name: test_card::TEST_COUNTERSPELL,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 10,
            name: test_card::TEST_COUNTERSPELL,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 11,
            name: test_card::TEST_COUNTERSPELL,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 12,
            name: test_card::TEST_COUNTERSPELL_UNLESS_PAYS,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 13,
            name: test_card::TEST_COUNTERSPELL_UNLESS_PAYS,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 14,
            name: test_card::TEST_COUNTERSPELL_UNLESS_PAYS,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 15,
            name: test_card::TEST_VARIABLE_ENERGY_DRAW,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 16,
            name: test_card::TEST_VARIABLE_ENERGY_DRAW,
            owner: PlayerName::One,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 17,
            name: test_card::TEST_VARIABLE_ENERGY_DRAW,
            owner: PlayerName::One,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 18,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 19,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::Two,
            zone: Zone::Battlefield,
        },
        BenchmarkCardSpec {
            id: 20,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 21,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 22,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 23,
            name: test_card::TEST_VANILLA_CHARACTER,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 24,
            name: test_card::TEST_DISSOLVE,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 25,
            name: test_card::TEST_DISSOLVE,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 26,
            name: test_card::TEST_DISSOLVE,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 27,
            name: test_card::TEST_COUNTERSPELL,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 28,
            name: test_card::TEST_COUNTERSPELL,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 29,
            name: test_card::TEST_COUNTERSPELL,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 30,
            name: test_card::TEST_COUNTERSPELL_UNLESS_PAYS,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 31,
            name: test_card::TEST_COUNTERSPELL_UNLESS_PAYS,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 32,
            name: test_card::TEST_COUNTERSPELL_UNLESS_PAYS,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
        BenchmarkCardSpec {
            id: 33,
            name: test_card::TEST_VARIABLE_ENERGY_DRAW,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 34,
            name: test_card::TEST_VARIABLE_ENERGY_DRAW,
            owner: PlayerName::Two,
            zone: Zone::Hand,
        },
        BenchmarkCardSpec {
            id: 35,
            name: test_card::TEST_VARIABLE_ENERGY_DRAW,
            owner: PlayerName::Two,
            zone: Zone::Deck,
        },
    ];

    let seed = 12345678912345;
    let mut battle = BattleState {
        id: BattleId(Uuid::new_v4()),
        cards: AllCards::default(),
        rules_config: BattleRulesConfig { points_to_win: Points(25) },
        players: PlayerMap {
            one: BattlePlayerState {
                player_type: PlayerType::Agent(GameAI::AlwaysPanic),
                points: Points(15),
                current_energy: Energy(6),
                produced_energy: Energy(6),
                spark_bonus: Spark(0),
                deck_name: TestDeckName::StartingFive,
                quest: Arc::new(new_test_battle::create_quest_state(TestDeckName::StartingFive)),
            },
            two: BattlePlayerState {
                player_type: PlayerType::Agent(GameAI::AlwaysPanic),
                points: Points(0),
                current_energy: Energy(5),
                produced_energy: Energy(5),
                spark_bonus: Spark(0),
                deck_name: TestDeckName::StartingFive,
                quest: Arc::new(new_test_battle::create_quest_state(TestDeckName::StartingFive)),
            },
        },
        status: BattleStatus::Playing,
        stack_priority: None,
        turn: TurnData { active_player: PlayerName::One, turn_id: TurnId(9) },
        phase: BattleTurnPhase::Main,
        seed,
        rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        prompts: VecDeque::new(),
        triggers: TriggerState::default(),
        activated_abilities: PlayerMap::default(),
        ability_state: AbilityState::default(),
        pending_effects: VecDeque::new(),
        animations: None,
        tracing: None,
        action_history: None,
        turn_history: TurnHistory::default(),
        request_context: RequestContext { logging_options: LoggingOptions::default() },
    };

    battle_deck::add_cards(
        &mut battle,
        PlayerName::One,
        card_specs
            .iter()
            .filter(|spec| spec.owner == PlayerName::One)
            .map(|spec| card_descriptor::get_base_identity(spec.name))
            .collect(),
    );
    battle_deck::add_cards(
        &mut battle,
        PlayerName::Two,
        card_specs
            .iter()
            .filter(|spec| spec.owner == PlayerName::Two)
            .map(|spec| card_descriptor::get_base_identity(spec.name))
            .collect(),
    );

    for spec in &card_specs {
        let card_id = CardId(spec.id);
        if spec.zone != Zone::Deck {
            battle.cards.move_card(spec.owner, card_id, Zone::Deck, spec.zone);
        }

        if spec.zone == Zone::Battlefield {
            let character_id = CharacterId(card_id);
            let card_name = card::get(&battle, character_id).identity;
            if card_name == card_descriptor::get_base_identity(test_card::TEST_VANILLA_CHARACTER) {
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
            "Expected action {action:?} not found in legal actions {legal_actions_vec:?}"
        );
    }

    for action in &legal_actions_vec {
        assert!(
            expected_actions.contains(action),
            "Unexpected action {action:?} found in legal actions"
        );
    }

    battle
}
