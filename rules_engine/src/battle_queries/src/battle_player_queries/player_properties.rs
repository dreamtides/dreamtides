use battle_state::battle::battle_state::BattleState;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

/// Returns the total spark value for a player.
pub fn spark_total(battle: &BattleState, player: PlayerName) -> Spark {
    battle
        .cards
        .battlefield(player)
        .iter()
        .map(|(_, character_state)| character_state.spark)
        .sum::<Spark>()
        + battle.players.player(player).spark_bonus
}
