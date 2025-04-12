use battle_data::battle::battle_data::BattleData;
use battle_data::cards::card_id::{CardIdType, ObjectId};
use battle_data::cards::zone::Zone;
use core_data::source::Source;
use tracing::debug;

/// Moves a card to a new zone, updates indices, assigns a new
/// [ObjectId] to it, and fires all relevant events.
///
/// The card is added as the top card of the target zone if it is ordered.
///
/// Returns None if the card no longer exists, otherwise returns the new
/// [ObjectId] for the card.
pub fn run(
    battle: &mut BattleData,
    _source: Source,
    id: impl CardIdType,
    new: Zone,
) -> Option<ObjectId> {
    let card = battle.cards.card(id)?;
    let card_id = card.id;
    let old = card.zone();
    debug!(?card_id, ?old, ?new, "Moving card to zone");
    battle.cards.move_card(card_id, new)
}
