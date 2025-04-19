use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::battle_cards::card_id::{
    CardIdType, CharacterId, HandCardId, ObjectId, StackCardId, VoidCardId,
};
use battle_data::battle_cards::zone::Zone;
use tracing::debug;

/// Moves a card to the hand, updates indices, assigns a new
/// [HandCardId] to it, and fires all relevant events.
///
/// Returns the new [HandCardId] for the card if it was moved to the hand, or
/// None if the card no longer exists.
pub fn to_hand(
    battle: &mut BattleData,
    source: EffectSource,
    id: impl CardIdType,
) -> Option<HandCardId> {
    to_zone(battle, source, id, Zone::Hand)?;
    Some(HandCardId(id.card_id()))
}

/// Moves a card to the stack, updates indices, assigns a new
/// [StackCardId] to it, and fires all relevant events.
///
/// Returns the new [StackCardId] for the card if it was moved to the stack, or
/// None if the card no longer exists.
pub fn to_stack(
    battle: &mut BattleData,
    source: EffectSource,
    id: impl CardIdType,
) -> Option<StackCardId> {
    to_zone(battle, source, id, Zone::Stack)?;
    Some(StackCardId(id.card_id()))
}

/// Moves a card to the battlefield, updates indices, assigns a new
/// [CharacterId] to it, and fires all relevant events.
///
/// Returns the new [CharacterId] for the card if it was moved to the
/// battlefield, or None if the card no longer exists.
pub fn to_battlefield(
    battle: &mut BattleData,
    source: EffectSource,
    id: impl CardIdType,
) -> Option<CharacterId> {
    to_zone(battle, source, id, Zone::Battlefield)?;
    Some(CharacterId(id.card_id()))
}

/// Moves a card to the void, updates indices, assigns a new
/// [VoidCardId] to it, and fires all relevant events.
///
/// Returns the new [VoidCardId] for the card if it was moved to the void, or
/// None if the card no longer exists.
pub fn to_void(
    battle: &mut BattleData,
    source: EffectSource,
    id: impl CardIdType,
) -> Option<VoidCardId> {
    to_zone(battle, source, id, Zone::Void)?;
    Some(VoidCardId(id.card_id()))
}

/// Moves a card to a new zone, updates indices, assigns a new
/// [ObjectId] to it, and fires all relevant events.
///
/// The card is added as the top card of the target zone if it is ordered.
///
/// Returns None if the card no longer exists, otherwise returns the new
/// [ObjectId] for the card.
fn to_zone(
    battle: &mut BattleData,
    _source: EffectSource,
    id: impl CardIdType,
    new: Zone,
) -> Option<ObjectId> {
    let card = battle.cards.card(id)?;
    let card_id = card.id;
    let old = card.zone;
    debug!(?card_id, ?old, ?new, "Moving card to zone");
    battle.cards.move_card(card_id, new)
}
