use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId, HandCardId, StackCardId};
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;
use tracing_macros::assert_that;

/// Moves a card from the 'controller' player's hand to the stack.
///
/// Panics if this card is not found in hand.
pub fn from_hand_to_stack(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: HandCardId,
) {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Hand, Zone::Stack);
}

/// Moves a card from the stack to the 'controller' player's battlefield.
///
/// Panics if this card is not found on the stack.
pub fn from_stack_to_battlefield(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: StackCardId,
) {
    to_destination_zone(
        battle,
        source,
        controller,
        card_id.card_id(),
        Zone::Stack,
        Zone::Battlefield,
    );
}

/// Moves a card from the stack to the 'controller' player's void.
///
/// Panics if this card is not found on the stack.
pub fn from_stack_to_void(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: StackCardId,
) {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Stack, Zone::Void);
}

/// Moves a character from the 'controller' player's battlefield to the void.
///
/// Panics if this character is not found.
pub fn from_battlefield_to_void(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: CharacterId,
) {
    to_destination_zone(
        battle,
        source,
        controller,
        card_id.card_id(),
        Zone::Battlefield,
        Zone::Void,
    );
}

/// Moves a card from the 'old' zone to the 'new' zone.
///
/// Panics if this card is not found in the 'old' zone.
pub fn to_destination_zone(
    battle: &mut BattleState,
    _source: EffectSource,
    controller: PlayerName,
    id: impl CardIdType,
    old: Zone,
    new: Zone,
) {
    assert_that!(
        battle.cards.contains_card(controller, id.card_id(), old),
        "Card not found",
        battle,
        id,
        old,
        new
    );
    battle.cards.move_card(controller, id.card_id(), old, new);
}
