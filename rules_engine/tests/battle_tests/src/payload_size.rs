use std::collections::BTreeMap;

use ai_data::game_ai::GameAI;
use battle_mutations::card_mutations::{battle_deck, move_card};
use battle_state::battle::battle_state::{BattleState, LoggingOptions, RequestContext};
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use battle_state::core::effect_source::EffectSource;
use core_data::identifiers::{BattleId, CardIdentity, UserId};
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::request_data::{ConnectResponse, Metadata};
use game_creation::new_test_battle;
use quest_state::quest::card_descriptor;
use quest_state::quest::deck::Deck;
use state_provider::display_state_provider::DisplayStateProvider;
use state_provider::state_provider::StateProvider;
use state_provider::test_state_provider::TestStateProvider;
use tabula_ids::test_card;
use uuid::Uuid;
use {logging, serde_json};

pub fn generate_payload_json(pretty: bool) -> String {
    let response = create_500_card_battle_json();
    if pretty {
        serde_json::to_string_pretty(&response).expect("Failed to serialize to JSON")
    } else {
        serde_json::to_string(&response).expect("Failed to serialize to JSON")
    }
}

fn create_500_card_battle_json() -> ConnectResponse {
    let battle_id = BattleId(Uuid::new_v4());
    let seed = 42u64;
    let user_id = UserId(Uuid::new_v4());

    let provider = TestStateProvider::new();
    let streaming_assets_path = logging::get_developer_mode_streaming_assets_path();
    let _ = provider.initialize("/tmp/test", &streaming_assets_path);
    let mut battle = new_test_battle::create_and_start(
        battle_id,
        provider.tabula(),
        seed,
        CreateBattlePlayer {
            player_type: PlayerType::User(user_id),
            deck_name: TestDeckName::CoreEleven,
        },
        CreateBattlePlayer {
            player_type: PlayerType::Agent(GameAI::AlwaysPanic),
            deck_name: TestDeckName::CoreEleven,
        },
        RequestContext { logging_options: LoggingOptions::default() },
    );

    add_500_cards(&mut battle);

    let commands = renderer::connect(&battle, user_id, provider, false);

    ConnectResponse {
        metadata: Metadata {
            user_id,
            battle_id: Some(battle_id),
            request_id: None,
            integration_test_id: None,
        },
        commands,
        response_version: Uuid::new_v4(),
    }
}

fn add_500_cards(battle: &mut BattleState) {
    let core_11_cards = get_core_11_card_mix();

    battle_deck::add_cards(battle, PlayerName::One, core_11_cards.clone());
    move_all_from_deck_to_void(battle, PlayerName::One);
    battle_deck::add_cards(battle, PlayerName::Two, core_11_cards);
    move_all_from_deck_to_void(battle, PlayerName::Two);
}

fn move_all_from_deck_to_void(battle: &mut BattleState, player_name: PlayerName) {
    let cards = battle.cards.all_deck_cards(player_name).collect::<Vec<_>>();
    for card_id in cards {
        move_card::from_deck_to_void(
            battle,
            EffectSource::Game { controller: player_name },
            player_name,
            card_id,
        );
    }
}

fn get_core_11_card_mix() -> Vec<CardIdentity> {
    let base_deck_map = create_500_card_core_11_deck();
    let mut cards = Vec::new();

    for (card_name, count) in base_deck_map.cards {
        for _ in 0..count {
            cards.push(card_name);
        }
    }

    cards
}

fn create_500_card_core_11_deck() -> Deck {
    let mut deck_cards = BTreeMap::new();

    #[expect(clippy::integer_division)]
    let scale_factor = 500_usize / 32_usize;
    let remainder = 500_usize % 32_usize;

    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_NAMED_DISSOLVE),
        4 * scale_factor + if remainder > 0 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL),
        3 * scale_factor + if remainder > 4 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL_UNLESS_PAYS),
        2 * scale_factor + if remainder > 7 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_VARIABLE_ENERGY_DRAW),
        3 * scale_factor + if remainder > 9 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(
            test_card::TEST_TRIGGER_GAIN_SPARK_ON_PLAY_CARD_ENEMY_TURN,
        ),
        4 * scale_factor + if remainder > 12 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(
            test_card::TEST_FAST_MULTI_ACTIVATED_ABILITY_DRAW_CARD_CHARACTER,
        ),
        5 * scale_factor + if remainder > 16 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(
            test_card::TEST_RETURN_ONE_OR_TWO_VOID_EVENT_CARDS_TO_HAND,
        ),
        2 * scale_factor + if remainder > 21 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_MODAL_RETURN_TO_HAND_OR_DRAW_TWO),
        2 * scale_factor + if remainder > 23 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_PREVENT_DISSOLVE_THIS_TURN),
        2 * scale_factor + if remainder > 25 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_FORESEE_ONE_DRAW_RECLAIM),
        3 * scale_factor + if remainder > 27 { 1 } else { 0 },
    );
    deck_cards.insert(
        card_descriptor::get_base_identity(test_card::TEST_COUNTERSPELL_CHARACTER),
        2 * scale_factor + if remainder > 30 { 1 } else { 0 },
    );

    Deck { cards: deck_cards }
}
