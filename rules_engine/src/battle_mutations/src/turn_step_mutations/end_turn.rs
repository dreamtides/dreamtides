use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_turn_step::BattleTurnStep;

/// End the current player's turn.
///
/// Their opponent may take 'fast' actions before beginning a new turn.
pub fn run(battle: &mut BattleData) {
    battle.step = BattleTurnStep::Ending;
}
