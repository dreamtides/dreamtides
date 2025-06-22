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
use core_data::identifiers::QuestId;
use core_data::types::PlayerName;
use database::save_file::SaveFile;
use game_creation::new_battle;
use serde_json;
use tracing::{error, instrument, subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

use crate::handle_battle_action::should_auto_execute_action;

#[derive(Debug, Clone)]
struct DeserializationAction {
    player: PlayerName,
    action: BattleAction,
    action_index: usize,
    turn_id: u32,
    phase: BattleTurnPhase,
}

/// Returns a deserialized [BattleState] for the battle in this save
/// file, if one is present.
#[instrument(name = "deserialize_save_file", level = "debug", skip(file, request_context))]
pub fn battle(file: &SaveFile, request_context: RequestContext) -> Option<(BattleState, QuestId)> {
    get_battle_impl(file, None, request_context)
}

/// Returns a deserialized [BattleState] for an 'undo' operation on the battle
/// in this save file, if  one is present.
///
/// We advance the battle state to one which is immediately before the named
/// player's last battle action.
pub fn undo(
    file: &SaveFile,
    player: PlayerName,
    request_context: RequestContext,
) -> Option<(BattleState, QuestId)> {
    get_battle_impl(file, Some(player), request_context)
}

fn get_battle_impl(
    file: &SaveFile,
    undo: Option<PlayerName>,
    request_context: RequestContext,
) -> Option<(BattleState, QuestId)> {
    match file {
        SaveFile::V1(v1) => {
            let filter = EnvFilter::new("warn");
            let forest_subscriber =
                tracing_subscriber::registry().with(logging::create_forest_layer(filter));

            let mut result = subscriber::with_default(forest_subscriber, || {
                let quest_id = v1.quest.as_ref()?.id;
                let file = v1.quest.as_ref()?.battle.as_ref()?;
                let mut battle = new_battle::create_and_start_with_options(
                    file.id,
                    file.seed,
                    file.player_types.one.clone(),
                    file.player_types.two.clone(),
                    request_context,
                );
                battle.animations = None;
                battle.tracing = None;

                let mut last_non_auto_battle = None;
                let mut actions = Vec::new();
                for (action_index, history_action) in file.actions.iter().enumerate() {
                    let is_undo_player = undo == Some(history_action.player);
                    let legal = legal_actions::compute(&battle, history_action.player);
                    let auto = should_auto_execute_action(&legal);
                    if is_undo_player && auto != Some(history_action.action) {
                        last_non_auto_battle = Some((battle.clone(), quest_id));
                    }

                    actions.push(DeserializationAction {
                        player: history_action.player,
                        action: history_action.action,
                        action_index,
                        turn_id: battle.turn.turn_id.0,
                        phase: battle.phase,
                    });

                    let battle_clone = battle.clone();
                    let result = panic::catch_unwind(AssertUnwindSafe(|| {
                        apply_battle_action::execute(
                            &mut battle,
                            history_action.player,
                            history_action.action,
                        );
                    }));

                    if let Err(panic_info) = result {
                        error!("Panic during deserialization at action {}", action_index);

                        write_deserialization_panic_trace(
                            &battle_clone,
                            &actions,
                            action_index,
                            &format!("{:?}", panic_info),
                        );
                        panic::resume_unwind(panic_info);
                    }
                }

                if undo.is_some() {
                    last_non_auto_battle
                } else {
                    Some((battle, quest_id))
                }
            });

            if let Some((battle, _)) = &mut result {
                battle.tracing = Some(BattleTracing::default());
                battle.animations = Some(AnimationData::default());
                if undo.is_some() {
                    battle_trace!("Completed undo operation", battle);
                } else {
                    battle_trace!("Completed battle replay", battle);
                }
            }

            result
        }
    }
}

fn write_deserialization_panic_trace(
    battle: &BattleState,
    actions: &[DeserializationAction],
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

    write_tracing_event::write_deserialization_panic(
        battle,
        panic_action_index,
        actions.len(),
        panic_info.to_string(),
        action_history,
        panic_details,
    );
}
