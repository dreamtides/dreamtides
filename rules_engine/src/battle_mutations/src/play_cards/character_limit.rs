use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;

/// Returns true if the player's battlefield is full and no more characters can
/// be materialized.
pub fn is_full(battle: &BattleState, player: PlayerName) -> bool {
    battle.cards.battlefield(player).character_count() >= 9
}
