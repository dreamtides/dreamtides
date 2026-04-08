use battle_queries::battle_trace;
use battle_state::battle::battle_animation_data::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle_cards::ability_state::UntilEndOfTurn;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::numerics::TurnId;

use crate::card_mutations::battle_deck;
use crate::effects::apply_effect;
use crate::phase_mutations::{dawn_phase, dreamwell_phase, fire_triggers, judgment_phase};

/// End the current player's turn.
///
/// Transitions into the Ending phase for fast actions.
pub fn to_ending_phase(battle: &mut BattleState) {
    battle.phase = BattleTurnPhase::Ending;
    battle_trace!("Moving to ending phase for player", battle, player = battle.turn.active_player);
}

pub fn run_turn_state_machine_if_no_active_prompts(battle: &mut BattleState) {
    while battle.prompts.is_empty() && !battle.status.is_game_over() {
        match battle.phase {
            BattleTurnPhase::EndingPhaseFinished => {
                battle.phase = BattleTurnPhase::FiringEndOfTurnTriggers;
                let source = EffectSource::Game { controller: battle.turn.active_player };
                battle.triggers.push(source, Trigger::EndOfTurn(battle.turn.active_player));
                apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                fire_triggers::execute_if_no_active_prompt(battle);
            }
            BattleTurnPhase::FiringEndOfTurnTriggers => {
                battle.phase = BattleTurnPhase::Starting;
                let previous_player = battle.turn.active_player;
                let next_player = previous_player.opponent();

                battle_trace!("Starting turn for", battle, next_player);
                battle.ability_state.until_end_of_turn = UntilEndOfTurn::default();
                battle
                    .activated_abilities
                    .player_mut(previous_player)
                    .activated_this_turn_cycle
                    .clear();
                battle.turn.active_player = next_player;
                battle.turn.turn_id += TurnId(1);
                battle.turn.moved_this_turn.clear();
                battle.turn.positioning_started = false;
                battle.turn.positioning_character = None;
                battle.turn.judgment_participants.clear();
                if battle.turn.turn_id >= TurnId(50) {
                    // If the battle has lasted more than 50 turns (25 per player), it is a
                    // draw.
                    battle.status = BattleStatus::GameOver { winner: None };
                    break;
                }
                battle.push_animation(EffectSource::Game { controller: previous_player }, || {
                    BattleAnimation::StartTurn { player: next_player }
                });
            }
            BattleTurnPhase::Starting => {
                battle.phase = BattleTurnPhase::Dreamwell;
                dreamwell_phase::activate(battle, battle.turn.active_player);
                apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                fire_triggers::execute_if_no_active_prompt(battle);
            }
            BattleTurnPhase::Dreamwell => {
                battle.phase = BattleTurnPhase::Draw;
                if battle.turn.turn_id != TurnId(0) {
                    battle_deck::draw_card(
                        battle,
                        EffectSource::Game { controller: battle.turn.active_player },
                        battle.turn.active_player,
                    );
                }
                apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                fire_triggers::execute_if_no_active_prompt(battle);
            }
            BattleTurnPhase::Draw => {
                battle.phase = BattleTurnPhase::Dawn;
                dawn_phase::run(battle, battle.turn.active_player, EffectSource::Game {
                    controller: battle.turn.active_player,
                });
                apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                fire_triggers::execute_if_no_active_prompt(battle);
            }
            BattleTurnPhase::Dawn => {
                battle.phase = BattleTurnPhase::Main;
            }
            BattleTurnPhase::Judgment => {
                let player = battle.turn.active_player;
                let source = EffectSource::Game { controller: player };
                let finished = judgment_phase::run(battle, player, source);
                apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                fire_triggers::execute_if_no_active_prompt(battle);
                if finished && battle.prompts.is_empty() {
                    battle.triggers.push(source, Trigger::Judgment(player));
                    apply_effect::execute_pending_effects_if_no_active_prompt(battle);
                    fire_triggers::execute_if_no_active_prompt(battle);
                    battle.phase = BattleTurnPhase::EndingPhaseFinished;
                    battle_trace!(
                        "Judgment phase complete, moving to ending phase finished for player",
                        battle,
                        player
                    );
                }
            }
            _ => {
                break;
            }
        }
    }
}

/// Transition from the Ending phase to the Judgment phase.
///
/// Called when the non-active player passes during the Ending phase.
pub fn start_next_turn(battle: &mut BattleState) {
    battle.turn.judgment_position = 0;
    battle.turn.judgment_participants.clear();
    battle.phase = BattleTurnPhase::Judgment;
    battle_trace!(
        "Moving to judgment phase for player",
        battle,
        player = battle.turn.active_player
    );
}
