use battle_queries::battle_trace;
use battle_state::battle::battle_animation_data::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle_cards::ability_state::UntilEndOfTurn;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::numerics::TurnId;
use core_data::types::PlayerName;

use crate::card_mutations::battle_deck;
use crate::phase_mutations::{dreamwell_phase, fire_triggers, judgment_phase};

/// End the current player's turn.
///
/// Their opponent may take 'fast' actions before beginning a new turn.
pub fn to_ending_phase(battle: &mut BattleState) {
    battle.phase = BattleTurnPhase::Ending;
    battle_trace!("Moving to end step for player", battle, player = battle.turn.active_player);
}

/// Start a turn for `player`.
pub fn start_turn(battle: &mut BattleState, player: PlayerName) {
    let source = EffectSource::Game { controller: player };

    battle.triggers.push(source, Trigger::EndOfTurn(player.opponent()));
    fire_triggers::execute_if_no_active_prompt(battle);
    if !battle.prompts.is_empty() {
        todo!("Handle prompts from end of turn");
    }

    battle.ability_state.until_end_of_turn = UntilEndOfTurn::default();

    battle_trace!("Starting turn for", battle, player);
    battle.activated_abilities.player_mut(player).activated_this_turn_cycle.clear();
    battle.turn.active_player = player;
    battle.phase = BattleTurnPhase::Starting;
    battle.turn.turn_id += TurnId(1);
    if battle.turn.turn_id > TurnId(50) {
        // If the battle has lasted more than 50 turns (25 per player), it is a
        // draw.
        battle.status = BattleStatus::GameOver { winner: None };
        return;
    }

    battle.push_animation(source, || BattleAnimation::StartTurn { player });

    judgment_phase::run(battle, battle.turn.active_player, source);
    if !battle.prompts.is_empty() {
        todo!("Handle prompts from judgment phase");
    }

    dreamwell_phase::activate(battle, battle.turn.active_player, source);
    if !battle.prompts.is_empty() {
        todo!("Handle prompts from dreamwell phase");
    }

    battle.phase = BattleTurnPhase::Draw;

    if battle.turn.turn_id != TurnId(1) {
        battle_deck::draw_card(battle, source, player);
    }

    if !battle.prompts.is_empty() {
        todo!("Handle prompts from draw phase");
    }
    battle.phase = BattleTurnPhase::Main;
}
