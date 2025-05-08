use battle_data_old::battle::effect_source::EffectSource;
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle_cards::card_id::CharacterId;
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
pub fn apply(
    battle: &mut BattleData,
    source: EffectSource,
    player: PlayerName,
    new_character: CharacterId,
) -> Option<()> {
    if battle.cards.battlefield(player).len() <= CHARACTER_LIMIT {
        return None;
    }

    let target = battle
        .cards
        .battlefield(player)
        .iter()
        .filter(|&&id| id != new_character)
        .min_by_key(|&&id| {
            (
                battle.cards.card(id).and_then(|card| card.properties.spark),
                battle.cards.card(id).and_then(|card| card.properties.cost),
            )
        })
        .copied()?;

    let spark_value = battle.cards.card(target)?.properties.spark?;
    abandon::apply(battle, source, target);
    battle.player_mut(player).spark_bonus += spark_value;
    Some(())
}
