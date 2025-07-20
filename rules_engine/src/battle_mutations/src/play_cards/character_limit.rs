use battle_queries::battle_card_queries::card_properties;
use battle_state::battle::battle_state::BattleState;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::character_mutations::abandon;

const CHARACTER_LIMIT: usize = 8;

/// Applies the character limit, if necessary, for a player.
///
/// If a character resolves in excess of the limit, one of the controller's
/// other characters is abandoned. This operation happens before the new
/// character enters play.
///
/// The abandoned character is selected based on the following ranking:
///
/// - The lowest-spark character with no activated or triggered abilities,
/// - The lowest-spark character with a '>Materialized' ability
/// - The lowest-spark character with a triggered ability
/// - The lowest-spark character with an activated ability
///
/// Ties are broken by selecting the lowest-cost character, and then by internal
/// card ID.
///
/// The current spark value of the abandoned character is permanently granted to
/// its controller as a 'spark bonus'.
pub fn apply(battle: &mut BattleState, source: EffectSource, player: PlayerName) -> Option<()> {
    if battle.cards.battlefield_state(player).len() < CHARACTER_LIMIT {
        return None;
    }

    let (target_id, _) =
        battle.cards.battlefield_state(player).iter().min_by_key(|(id, state)| {
            (state.spark, card_properties::converted_energy_cost(battle, **id))
        })?;

    battle
        .turn_history
        .current_action_history
        .player_mut(player)
        .character_limit_characters_abandoned
        .insert(*target_id);

    let spark_value = battle.cards.spark(player, *target_id)?;
    abandon::apply(battle, source, *target_id);
    battle.players.player_mut(player).spark_bonus += spark_value;
    Some(())
}
