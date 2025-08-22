use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use battle_mutations::card_mutations::battle_deck;
use battle_mutations::phase_mutations::turn;
use battle_state::battle::ability_cache::AbilityCache;
use battle_state::battle::all_cards::AllCards;
use battle_state::battle::battle_rules_config::BattleRulesConfig;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::turn_data::TurnData;
use battle_state::battle::turn_history::TurnHistory;
use battle_state::battle_cards::ability_state::AbilityState;
use battle_state::battle_player::battle_player_state::{
    BattlePlayerState, CreateBattlePlayer, TestDeckName,
};
use battle_state::battle_player::player_map::PlayerMap;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger_state::TriggerState;
use core_data::identifiers::{BattleId, QuestId, UserId};
use core_data::numerics::{Energy, Essence, Points, Spark, TurnId};
use core_data::types::PlayerName;
use quest_state::quest::card_descriptor;
use quest_state::quest::deck::Deck;
use quest_state::quest::quest_state::QuestState;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tabula_data::tabula::Tabula;
use tabula_ids::test_card;
use user_state::user::user_state::UserState;
use uuid::Uuid;

/// Creates a new test battle between two Agents and starts it.
pub fn create_and_start(
    id: BattleId,
    tabula: Arc<Tabula>,
    seed: u64,
    player_one: CreateBattlePlayer,
    player_two: CreateBattlePlayer,
    request_context: RequestContext,
) -> BattleState {
    let mut battle = BattleState {
        id,
        cards: AllCards::default(),
        rules_config: BattleRulesConfig { points_to_win: Points(12) },
        tabula,
        ability_cache: Arc::new(AbilityCache::default()),
        players: PlayerMap {
            one: BattlePlayerState {
                player_type: player_one.player_type,
                points: Points(0),
                spark_bonus: Spark(0),
                current_energy: Energy(0),
                produced_energy: Energy(0),
                deck_name: player_one.deck_name,
                quest: Arc::new(create_quest_state(player_one.deck_name)),
            },
            two: BattlePlayerState {
                player_type: player_two.player_type,
                points: Points(0),
                spark_bonus: Spark(0),
                current_energy: Energy(0),
                produced_energy: Energy(0),
                deck_name: player_two.deck_name,
                quest: Arc::new(create_quest_state(player_two.deck_name)),
            },
        },
        status: BattleStatus::Setup,
        stack_priority: None,
        turn: TurnData { active_player: PlayerName::One, turn_id: TurnId::default() },
        phase: BattleTurnPhase::Judgment,
        seed,
        rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        animations: None,
        prompts: VecDeque::new(),
        triggers: TriggerState::default(),
        activated_abilities: PlayerMap::default(),
        ability_state: AbilityState::default(),
        pending_effects: VecDeque::new(),
        tracing: None,
        action_history: None,
        turn_history: TurnHistory::default(),
        request_context,
    };

    battle_deck::add_deck_copy(&mut battle, PlayerName::One);
    battle_deck::add_deck_copy(&mut battle, PlayerName::Two);

    battle.status = BattleStatus::Playing;
    battle_deck::draw_cards(
        &mut battle,
        EffectSource::Game { controller: PlayerName::One },
        PlayerName::One,
        5,
    );
    battle_deck::draw_cards(
        &mut battle,
        EffectSource::Game { controller: PlayerName::Two },
        PlayerName::Two,
        5,
    );
    turn::start_turn(&mut battle, PlayerName::One);
    battle
}

/// Creates a new quest state
pub fn create_quest_state(deck_name: TestDeckName) -> QuestState {
    QuestState {
        id: QuestId(Uuid::new_v4()),
        user: UserState { id: UserId::default() },
        deck: create_test_deck(deck_name),
        essence: Essence(0),
    }
}

fn create_test_deck(name: TestDeckName) -> Deck {
    let mut deck_cards = BTreeMap::new();
    match name {
        TestDeckName::Vanilla => {
            deck_cards
                .insert(card_descriptor::get_base_identity(test_card::TEST_VANILLA_CHARACTER), 30);
        }
        TestDeckName::StartingFive => {
            deck_cards
                .insert(card_descriptor::get_base_identity(test_card::TEST_VANILLA_CHARACTER), 6);
            deck_cards.insert(card_descriptor::get_base_identity(test_card::TEST_DISSOLVE), 3);
            deck_cards.insert(card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL), 3);
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL_UNLESS_PAYS),
                3,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_VARIABLE_ENERGY_DRAW),
                3,
            );
        }
        TestDeckName::CoreEleven => {
            deck_cards
                .insert(card_descriptor::get_base_identity(test_card::TEST_NAMED_DISSOLVE), 4);
            deck_cards.insert(card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL), 3);
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL_UNLESS_PAYS),
                2,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_VARIABLE_ENERGY_DRAW),
                3,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(
                    test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN,
                ),
                4,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(
                    test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
                ),
                5,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(
                    test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND,
                ),
                2,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(
                    test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO,
                ),
                2,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_PREVENT_DISSOLVE_THIS_TURN),
                2,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_FORESEE_ONE_DRAW_RECLAIM),
                3,
            );
            deck_cards.insert(
                card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL_CHARACTER),
                2,
            );
        }
    }
    Deck { cards: deck_cards }
}
