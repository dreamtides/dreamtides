use battle_data::battle::battle_data::BattleData;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

/// Returns the total spark value for a player.
pub fn query(battle: &BattleData, player: PlayerName) -> Spark {
    battle.cards.battlefield_cards(player).filter_map(|c| c.properties.spark).sum::<Spark>()
        + battle.player(player).spark_bonus
}
