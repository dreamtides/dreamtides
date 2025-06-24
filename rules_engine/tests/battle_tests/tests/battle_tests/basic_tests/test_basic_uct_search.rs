use ai_data::game_ai::GameAI;
use battle_state::actions::battle_actions::BattleAction;
use display_data::command::GameMessageType;
use test_utils::battle::test_battle::TestBattle;

#[test]
fn test_monte_carlo_agent_basic_game() {
    let mut s = TestBattle::builder().enemy_agent(GameAI::MonteCarlo(1)).connect();
    s.perform_user_action(BattleAction::EndTurn);
    assert_eq!(
        s.user_client.last_game_message,
        Some(GameMessageType::YourTurn),
        "Enemy should have completed their turn"
    );
}
