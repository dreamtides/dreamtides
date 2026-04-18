use battle_queries::battle_card_queries::card;
use battle_state::battle::battle_rules_config::BalanceMode;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation::new_battle;
use state_provider::display_state_provider::DisplayStateProvider;
use state_provider::state_provider::StateProvider;
use state_provider::test_state_provider::TestStateProvider;
use tabula_generated::card_lists::DreamwellCardIdList;
use uuid::Uuid;

const OPENING_HAND_CARD_NAME: &str = "Test Variable Energy Draw";

#[test]
fn opening_hand_debug_configuration_swaps_named_card_into_hand() {
    let provider = TestStateProvider::new();
    provider
        .initialize("", &logging::get_developer_mode_streaming_assets_path())
        .expect("Failed to initialize test state provider");
    let tabula = provider.tabula();
    let dreamwell =
        Dreamwell::from_card_list(&tabula, DreamwellCardIdList::TestDreamwellNoAbilities);

    let battle = new_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        tabula.clone(),
        1,
        dreamwell.clone(),
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::StartingFive,
        },
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::StartingFive,
        },
        RequestContext::default(),
        PlayerName::One,
        None,
        None,
        Some(OPENING_HAND_CARD_NAME),
    );

    let control = new_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        tabula.clone(),
        1,
        dreamwell,
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::StartingFive,
        },
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::StartingFive,
        },
        RequestContext::default(),
        PlayerName::One,
        None,
        None,
        None,
    );

    assert_eq!(battle.rules_config.balance_mode, BalanceMode::FourFiveCards);
    assert_eq!(battle.cards.hand(PlayerName::One).len(), 4);
    assert!(hand_names(&battle, PlayerName::One).contains(&OPENING_HAND_CARD_NAME.to_string()));
    assert!(!hand_names(&control, PlayerName::One).contains(&OPENING_HAND_CARD_NAME.to_string()));
}

#[test]
fn new_battle_defaults_to_four_five_cards_when_player_one_goes_first() {
    let battle = create_battle(PlayerName::One);

    assert_eq!(battle.rules_config.balance_mode, BalanceMode::FourFiveCards);
    assert_eq!(battle.cards.hand(PlayerName::One).len(), 4);
    assert_eq!(battle.cards.hand(PlayerName::Two).len(), 5);
}

#[test]
fn new_battle_defaults_to_four_five_cards_when_player_one_goes_second() {
    let battle = create_battle(PlayerName::Two);

    assert_eq!(battle.rules_config.balance_mode, BalanceMode::FourFiveCards);
    assert_eq!(battle.cards.hand(PlayerName::One).len(), 5);
    assert_eq!(battle.cards.hand(PlayerName::Two).len(), 4);
}

fn create_battle(first_player: PlayerName) -> BattleState {
    let provider = TestStateProvider::new();
    provider
        .initialize("", &logging::get_developer_mode_streaming_assets_path())
        .expect("Failed to initialize test state provider");
    let tabula = provider.tabula();

    new_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        tabula.clone(),
        1,
        Dreamwell::from_card_list(&tabula, DreamwellCardIdList::TestDreamwellNoAbilities),
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::StartingFive,
        },
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::StartingFive,
        },
        RequestContext::default(),
        first_player,
        None,
        None,
        None,
    )
}

fn hand_names(battle: &BattleState, player: PlayerName) -> Vec<String> {
    battle
        .cards
        .hand(player)
        .iter()
        .map(|id| card::get_definition(battle, id).displayed_name.clone())
        .collect()
}
