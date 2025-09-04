use std::collections::VecDeque;
use std::sync::Arc;

use battle_mutations::card_mutations::battle_deck;
use battle_mutations::phase_mutations::turn;
use battle_queries::battle_card_queries::card_abilities;
use battle_queries::legal_action_queries::legal_actions_cache;
use battle_state::battle::all_cards::AllCards;
use battle_state::battle::battle_card_definitions::{
    BattleCardDefinitions, BattleCardDefinitionsCard,
};
use battle_state::battle::battle_rules_config::BattleRulesConfig;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::turn_data::TurnData;
use battle_state::battle::turn_history::TurnHistory;
use battle_state::battle_cards::ability_state::AbilityState;
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::{
    BattlePlayerState, CreateBattlePlayer, TestDeckName,
};
use battle_state::battle_player::player_map::PlayerMap;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger_state::TriggerState;
use core_data::identifiers::{BattleId, QuestId, UserId};
use core_data::numerics::{Energy, Essence, Points, Spark, TurnId};
use core_data::types::PlayerName;
use quest_state::quest::deck::Deck;
use quest_state::quest::quest_state::QuestState;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use tabula_data::tabula::Tabula;
use tabula_ids::card_lists::BaseCardIdList;
use tabula_ids::{card_lists, test_card};
use user_state::user::user_state::UserState;
use uuid::Uuid;

/// Creates a new test battle between two Agents and starts it.
pub fn create_and_start(
    id: BattleId,
    tabula: Arc<Tabula>,
    seed: u64,
    dreamwell: Dreamwell,
    player_one: CreateBattlePlayer,
    player_two: CreateBattlePlayer,
    request_context: RequestContext,
) -> BattleState {
    let quest_one = Arc::new(create_quest_state(&tabula, player_one.deck_name));
    let quest_two = Arc::new(create_quest_state(&tabula, player_two.deck_name));

    let mut cache_cards = Vec::new();
    for (quest_id, definition) in quest_one.deck.cards() {
        let ability_list = card_abilities::build_from_definition(definition);
        cache_cards.push(BattleCardDefinitionsCard {
            ability_list: Arc::new(ability_list),
            definition: Arc::new(definition.clone()),
            quest_deck_card_id: quest_id,
            owner: PlayerName::One,
        });
    }
    for (quest_id, definition) in quest_two.deck.cards() {
        let ability_list = card_abilities::build_from_definition(definition);
        cache_cards.push(BattleCardDefinitionsCard {
            ability_list: Arc::new(ability_list),
            definition: Arc::new(definition.clone()),
            quest_deck_card_id: quest_id,
            owner: PlayerName::Two,
        });
    }

    let ability_cache_response = BattleCardDefinitions::build(cache_cards);
    let ability_cache = Arc::new(ability_cache_response.cache);

    let created = ability_cache_response.created;
    let deck_one_len = quest_one.deck.cards.len();
    let deck_one = created.iter().take(deck_one_len).map(|c| c.identity).collect::<Vec<_>>();
    let deck_two = created.iter().skip(deck_one_len).map(|c| c.identity).collect::<Vec<_>>();

    let mut battle = BattleState {
        id,
        cards: AllCards::default(),
        rules_config: BattleRulesConfig { points_to_win: Points(12) },
        tabula,
        card_definitions: ability_cache,
        players: PlayerMap {
            one: BattlePlayerState {
                player_type: player_one.player_type,
                points: Points(0),
                spark_bonus: Spark(0),
                current_energy: Energy(0),
                produced_energy: Energy(0),
                deck_name: player_one.deck_name,
                deck: deck_one,
                quest: quest_one,
            },
            two: BattlePlayerState {
                player_type: player_two.player_type,
                points: Points(0),
                spark_bonus: Spark(0),
                current_energy: Energy(0),
                produced_energy: Energy(0),
                deck_name: player_two.deck_name,
                deck: deck_two,
                quest: quest_two,
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
        dreamwell,
        triggers: TriggerState::default(),
        activated_abilities: PlayerMap::default(),
        ability_state: AbilityState::default(),
        pending_effects: VecDeque::new(),
        tracing: None,
        action_history: None,
        turn_history: TurnHistory::default(),
        request_context,
        legal_actions_cache: Arc::new(PlayerMap::default()),
    };

    battle_deck::add_deck_copy(&mut battle, PlayerName::One);
    battle_deck::add_deck_copy(&mut battle, PlayerName::Two);

    legal_actions_cache::populate(&mut battle);

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

    battle.phase = BattleTurnPhase::Starting;
    turn::run_turn_state_machine_if_no_active_prompts(&mut battle);

    battle
}

/// Creates a new quest state
pub fn create_quest_state(tabula: &Tabula, deck_name: TestDeckName) -> QuestState {
    QuestState {
        id: QuestId(Uuid::new_v4()),
        user: UserState { id: UserId::default() },
        deck: create_test_deck(tabula, deck_name),
        essence: Essence(0),
    }
}

fn create_test_deck(tabula: &Tabula, name: TestDeckName) -> Deck {
    let mut deck = Deck::default();
    match name {
        TestDeckName::Vanilla => {
            deck.insert_copies(tabula, test_card::TEST_VANILLA_CHARACTER, 30);
        }
        TestDeckName::StartingFive => {
            deck.insert_copies(tabula, test_card::TEST_VANILLA_CHARACTER, 6);
            deck.insert_copies(tabula, test_card::TEST_DISSOLVE, 3);
            deck.insert_copies(tabula, test_card::TEST_COUNTERSPELL, 3);
            deck.insert_copies(tabula, test_card::TEST_COUNTERSPELL_UNLESS_PAYS, 3);
            deck.insert_copies(tabula, test_card::TEST_VARIABLE_ENERGY_DRAW, 3);
        }
        TestDeckName::Benchmark1 => {
            deck.insert_copies(tabula, test_card::TEST_NAMED_DISSOLVE, 4);
            deck.insert_copies(tabula, test_card::TEST_COUNTERSPELL, 3);
            deck.insert_copies(tabula, test_card::TEST_COUNTERSPELL_UNLESS_PAYS, 2);
            deck.insert_copies(tabula, test_card::TEST_VARIABLE_ENERGY_DRAW, 3);
            deck.insert_copies(
                tabula,
                test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN,
                4,
            );
            deck.insert_copies(
                tabula,
                test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
                5,
            );
            deck.insert_copies(
                tabula,
                test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND,
                2,
            );
            deck.insert_copies(tabula, test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO, 2);
            deck.insert_copies(tabula, test_card::TEST_PREVENT_DISSOLVE_THIS_TURN, 2);
            deck.insert_copies(tabula, test_card::TEST_FORESEE_ONE_DRAW_RECLAIM, 3);
            deck.insert_copies(tabula, test_card::TEST_COUNTERSPELL_CHARACTER, 2);
        }
        TestDeckName::Core11 => {
            populate_deck_from_base_card_id_list(tabula, &mut deck, BaseCardIdList::Core11);
        }
    }

    deck
}

fn populate_deck_from_base_card_id_list(tabula: &Tabula, deck: &mut Deck, list: BaseCardIdList) {
    for id in card_lists::base_card_id_list(list) {
        deck.insert_copies(tabula, *id, 1);
    }
}
