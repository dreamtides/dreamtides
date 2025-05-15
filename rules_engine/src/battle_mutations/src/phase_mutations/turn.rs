use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::TurnId;
use core_data::types::PlayerName;
use tracing_macros::battle_trace;

use crate::card_mutations::deck;
use crate::phase_mutations::{dreamwell, judgment};

/// End the current player's turn.
///
/// Their opponent may take 'fast' actions before beginning a new turn.
pub fn to_ending_phase(battle: &mut BattleState) {
    battle.phase = BattleTurnPhase::Ending;
    battle_trace!("Moving to end step for player", battle, player = battle.turn.active_player);
}

/// Start a turn for `player`.
pub fn start_turn(battle: &mut BattleState, player: PlayerName) {
    battle_trace!("Starting turn for", battle, player);
    battle.turn.active_player = player;
    battle.turn.turn_id += TurnId(1);
    if battle.turn.turn_id > TurnId(50) {
        // If the battle has lasted more than 50 turns (25 per player), it is a
        // draw.
        battle.status = BattleStatus::GameOver { winner: None };
        return;
    }

    let source = EffectSource::Game { controller: player };
    battle.push_animation(|| BattleAnimation::StartTurn { player });
    judgment::run(battle, battle.turn.active_player, source);
    dreamwell::activate(battle, battle.turn.active_player, source);
    battle.phase = BattleTurnPhase::Draw;

    if battle.turn.turn_id != TurnId(1) {
        deck::draw_card(battle, source, player);
    }

    battle.phase = BattleTurnPhase::Main;
}
