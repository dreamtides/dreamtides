use battle_data::battle::battle_data::BattleData;
use core_data::source::Source;

use crate::turn_step_mutations::start_turn;

/// End the current player's turn and start the next turn, running a judgment
/// phase & dreamwell activation.
pub fn run(battle: &mut BattleData, source: Source) {
    let next_player = battle.turn.active_player.opponent();
    start_turn::run(battle, next_player, source);
}
