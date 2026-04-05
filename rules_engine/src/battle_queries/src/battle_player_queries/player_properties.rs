use battle_state::battle::battle_state::BattleState;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

/// Returns the total spark value for a player.
pub fn spark_total(battle: &BattleState, player: PlayerName) -> Spark {
    let mut total = Spark(0);
    for character_id in battle.cards.battlefield(player).all_characters() {
        total += battle.cards.spark(player, character_id).unwrap_or(Spark(0));
    }
    total
}
