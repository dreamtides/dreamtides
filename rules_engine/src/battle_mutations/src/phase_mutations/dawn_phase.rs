use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

/// Runs the Dawn phase for the indicated player, firing Dawn triggers.
pub fn run(battle: &mut BattleState, player: PlayerName, source: EffectSource) {
    battle.triggers.push(source, Trigger::Dawn(player));
}
