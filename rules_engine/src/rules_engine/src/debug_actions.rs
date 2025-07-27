use action_data::debug_action_data::DebugAction;
use battle_mutations::actions::apply_battle_action;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_player::battle_player_state::{CreateBattlePlayer, PlayerType};
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use game_creation::new_battle;
use rand::RngCore;
use state_provider::state_provider::StateProvider;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use uuid::Uuid;

pub fn execute(
    provider: impl StateProvider + 'static,
    battle: &mut BattleState,
    user_id: UserId,
    user_player: PlayerName,
    action: DebugAction,
) {
    match action {
        DebugAction::ApplyTestScenarioAction => {}
        DebugAction::RestartBattle => {
            let seed = rand::rng().next_u64();
            *battle = new_battle::create_and_start(
                battle.id,
                seed,
                battle.players.one.as_create_battle_player(),
                battle.players.two.as_create_battle_player(),
                battle.request_context.clone(),
            );
        }
        DebugAction::RestartBattleWithDecks { one, two } => {
            let seed = rand::rng().next_u64();
            *battle = new_battle::create_and_start(
                battle.id,
                seed,
                CreateBattlePlayer {
                    player_type: battle.players.one.player_type.clone(),
                    deck_name: one,
                },
                CreateBattlePlayer {
                    player_type: battle.players.two.player_type.clone(),
                    deck_name: two,
                },
                battle.request_context.clone(),
            );
        }
        DebugAction::SetOpponentAgent(ai) => {
            battle.players.player_mut(user_player.opponent()).player_type = PlayerType::Agent(ai);
        }
        DebugAction::SetOpponentAsHuman => {
            let new_user_id = UserId(Uuid::new_v4());
            battle.players.player_mut(user_player.opponent()).player_type =
                PlayerType::User(new_user_id);
        }
        DebugAction::ApplyActionList(actions) => {
            let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
            tracing::subscriber::with_default(subscriber, || {
                for debug_action in actions {
                    apply_battle_action::execute(
                        battle,
                        user_player,
                        BattleAction::Debug(debug_action),
                    );
                }
            });
        }
        DebugAction::CloseCurrentPanelApplyAction(action) => {
            let mut display_state = provider.get_display_state(user_id);
            display_state.current_panel_address = None;
            provider.set_display_state(user_id, display_state);
            apply_battle_action::execute(battle, user_player, BattleAction::Debug(action));
        }
        DebugAction::PerformOpponentAction(action) => {
            apply_battle_action::execute(battle, user_player.opponent(), action);
        }
    }
}
