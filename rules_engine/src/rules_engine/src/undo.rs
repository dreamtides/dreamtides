use std::panic::{self, AssertUnwindSafe};

use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_trace;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::macros::write_tracing_event;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::animation_data::AnimationData;
use battle_state::battle::battle_state::{BattleState, RequestContext};
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle_trace::battle_tracing::BattleTracing;
use core_data::types::PlayerName;
use game_creation::new_battle;
use state_provider::state_provider::StateProvider;
use tracing::error;

use crate::handle_battle_action::should_auto_execute_action;

#[derive(Debug, Clone)]
struct ReplayAction {
    player: PlayerName,
    action: BattleAction,
    action_index: usize,
    turn_id: u32,
    phase: BattleTurnPhase,
}

pub fn undo<P>(
    provider: &P,
    current_battle: &BattleState,
    player: PlayerName,
    request_context: RequestContext,
) -> Option<BattleState>
where
    P: StateProvider + 'static,
{
    let mut battle = new_battle::create_and_start(
        current_battle.id,
        provider.tabula(),
        current_battle.seed,
        current_battle.dreamwell.clone(),
        current_battle.players.one.as_create_battle_player(),
        current_battle.players.two.as_create_battle_player(),
        request_context,
    );
    battle.animations = None;
    battle.tracing = None;

    let mut last_non_auto_battle = None;
    let mut actions = Vec::new();
    let history = current_battle.action_history.as_ref()?;
    for (action_index, history_action) in history.actions.iter().enumerate() {
        let is_undo_player = player == history_action.player;
        let legal = legal_actions::compute(&battle, history_action.player);
        let auto = should_auto_execute_action(&legal);
        if is_undo_player
            && auto != Some(history_action.action)
            && !should_skip_action_for_undo(history_action.action)
        {
            last_non_auto_battle = Some(battle.clone());
        }

        actions.push(ReplayAction {
            player: history_action.player,
            action: history_action.action,
            action_index,
            turn_id: battle.turn.turn_id.0,
            phase: battle.phase,
        });

        let battle_clone = battle.clone();
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            apply_battle_action::execute(&mut battle, history_action.player, history_action.action);
        }));

        if let Err(panic_info) = result {
            error!("Panic during undo at action {}", action_index);

            write_undo_panic_trace(
                &battle_clone,
                &actions,
                action_index,
                &format!("{panic_info:?}"),
            );
            panic::resume_unwind(panic_info);
        }
    }

    if let Some(battle) = &mut last_non_auto_battle {
        battle.tracing = Some(BattleTracing::default());
        battle.animations = Some(AnimationData::default());
        battle_trace!("Completed undo operation", battle);
    }

    last_non_auto_battle
}

fn should_skip_action_for_undo(action: BattleAction) -> bool {
    matches!(
        action,
        BattleAction::SelectOrderForDeckCard(..)
            | BattleAction::SelectVoidCardTarget(..)
            | BattleAction::SelectModalEffectChoice(..)
    )
}

fn write_undo_panic_trace(
    battle: &BattleState,
    actions: &[ReplayAction],
    panic_action_index: usize,
    panic_info: &str,
) {
    let action_history: Vec<serde_json::Value> = actions
        .iter()
        .map(|a| {
            serde_json::json!({
                "index": a.action_index,
                "player": format!("{:?}", a.player),
                "action": format!("{:?}", a.action),
                "turnId": a.turn_id,
                "phase": format!("{:?}", a.phase)
            })
        })
        .collect();

    let mut panic_details = serde_json::Map::new();
    if let Some(action) = actions.get(panic_action_index) {
        panic_details
            .insert("panic_player".to_string(), serde_json::json!(format!("{:?}", action.player)));
        panic_details
            .insert("panic_action".to_string(), serde_json::json!(format!("{:?}", action.action)));
        panic_details.insert("panic_turn".to_string(), serde_json::json!(action.turn_id));
        panic_details
            .insert("panic_phase".to_string(), serde_json::json!(format!("{:?}", action.phase)));
    }

    write_tracing_event::write_undo_panic(
        battle,
        panic_action_index,
        actions.len(),
        panic_info.to_string(),
        action_history,
        panic_details,
    );
}
