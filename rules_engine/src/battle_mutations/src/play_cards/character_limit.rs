use battle_queries::battle_card_queries::card_properties;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::character_mutations::abandon;

const CHARACTER_LIMIT: usize = 8;

/// Applies the character limit, if necessary, for a player.
///
/// If a character resolves in excess of the limit, one of the controller's
/// other characters is abandoned. This operation happens after the new
/// character is in play and causes triggers to fire, but before any other game
/// action can be taken.
///
/// The abandoned character is selected based on the following ranking:
///
/// - The lowest-spark character with no activated or triggered abilities,
/// - The lowest-spark character with a '>Materialized' ability
/// - The lowest-spark character with a triggered ability
/// - The lowest-spark character with an activated ability
///
/// Ties are broken by selecting the lowest-cost character.
///
/// The current spark value of the abandoned character is permanently granted to
/// its controller as a 'spark bonus'.
pub fn apply(battle: &mut BattleState, source: EffectSource, player: PlayerName) -> Option<()> {
    if battle.cards.battlefield(player).len() < CHARACTER_LIMIT {
        return None;
    }

    let (target_id, _) = battle
        .cards
        .battlefield(player)
        .iter()
        .min_by_key(|(&id, state)| (state.spark, card_properties::cost(battle, id)))?;

    let spark_value = battle.cards.spark(player, *target_id)?;
    abandon::apply(battle, source, player, *target_id);
    battle.players.player_mut(player).spark_bonus += spark_value;
    Some(())
}
