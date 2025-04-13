use battle_data::battle::battle_data::BattleData;
use core_data::numerics::TurnId;
use core_data::source::Source;

use crate::dreamwell_phase::dreamwell;
use crate::judgment_phase::judgment;

/// End the current player's turn and start the next turn, running a judgment
/// phase & dreamwell activation.
pub fn run(battle: &mut BattleData, source: Source) {
    let next_player = battle.turn.active_player.opponent();
    battle.turn.active_player = next_player;
    battle.turn.turn_id += TurnId(1);
    judgment::run(battle, battle.turn.active_player, source);
    dreamwell::activate(battle, battle.turn.active_player, source);
}
