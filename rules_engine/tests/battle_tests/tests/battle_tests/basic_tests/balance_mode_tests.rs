use battle_mutations::actions::apply_battle_action;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_rules_config::BalanceMode;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle_cards::dreamwell_data::Dreamwell;
use battle_state::battle_player::battle_player_state::{
    CreateBattlePlayer, PlayerType, TestDeckName,
};
use core_data::identifiers::{BattleId, UserId};
use core_data::types::PlayerName;
use game_creation::new_test_battle;
use state_provider::display_state_provider::DisplayStateProvider;
use state_provider::state_provider::StateProvider;
use state_provider::test_state_provider::TestStateProvider;
use tabula_generated::card_lists::DreamwellCardIdList;
use uuid::Uuid;

#[test]
fn bonus_energy_no_draw_skips_second_players_first_turn_draw() {
    let mut bonus_energy = create_battle(BalanceMode::BonusEnergy);
    let mut bonus_energy_no_draw = create_battle(BalanceMode::BonusEnergyNoDraw);

    assert_eq!(bonus_energy.cards.hand(PlayerName::One).len(), 5);
    assert_eq!(bonus_energy.cards.hand(PlayerName::Two).len(), 5);
    assert_eq!(bonus_energy_no_draw.cards.hand(PlayerName::One).len(), 5);
    assert_eq!(bonus_energy_no_draw.cards.hand(PlayerName::Two).len(), 5);

    advance_to_second_players_turn(&mut bonus_energy);
    advance_to_second_players_turn(&mut bonus_energy_no_draw);

    assert_eq!(bonus_energy.turn.active_player, PlayerName::Two);
    assert_eq!(bonus_energy_no_draw.turn.active_player, PlayerName::Two);
    assert_eq!(
        bonus_energy.players.player(PlayerName::Two).current_energy,
        bonus_energy_no_draw.players.player(PlayerName::Two).current_energy
    );
    assert_eq!(
        bonus_energy.players.player(PlayerName::Two).produced_energy,
        bonus_energy_no_draw.players.player(PlayerName::Two).produced_energy
    );
    assert_eq!(bonus_energy.cards.hand(PlayerName::Two).len(), 6);
    assert_eq!(bonus_energy_no_draw.cards.hand(PlayerName::Two).len(), 5);
}

#[test]
fn four_six_cards_sets_opening_hands_to_four_and_six() {
    let mut four_six_cards = create_battle(BalanceMode::FourSixCards);

    assert_eq!(four_six_cards.cards.hand(PlayerName::One).len(), 4);
    assert_eq!(four_six_cards.cards.hand(PlayerName::Two).len(), 6);

    advance_to_second_players_turn(&mut four_six_cards);

    assert_eq!(four_six_cards.turn.active_player, PlayerName::Two);
    assert_eq!(four_six_cards.cards.hand(PlayerName::Two).len(), 7);
}

fn advance_to_second_players_turn(battle: &mut BattleState) {
    apply_battle_action::execute_without_tracking_history(
        battle,
        PlayerName::One,
        BattleAction::EndTurn,
    );
    apply_battle_action::execute_without_tracking_history(
        battle,
        PlayerName::Two,
        BattleAction::StartNextTurn,
    );
}

fn create_battle(balance_mode: BalanceMode) -> BattleState {
    let provider = TestStateProvider::new();
    provider
        .initialize("", &logging::get_developer_mode_streaming_assets_path())
        .expect("Failed to initialize test state provider");
    let tabula = provider.tabula();

    new_test_battle::create_and_start(
        BattleId(Uuid::new_v4()),
        tabula.clone(),
        3141592653,
        Dreamwell::from_card_list(&tabula, DreamwellCardIdList::TestDreamwellNoAbilities),
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::Vanilla,
        },
        CreateBattlePlayer {
            player_type: PlayerType::User(UserId(Uuid::new_v4())),
            deck_name: TestDeckName::Vanilla,
        },
        RequestContext::default(),
        PlayerName::One,
        None,
        None,
        balance_mode,
    )
}
