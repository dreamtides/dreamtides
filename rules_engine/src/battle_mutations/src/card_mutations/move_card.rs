use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, HandCardId};
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

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
    // assert_that!(battle.cards.contains_card(controller, id.card_id(), old),
    // battle, || format!(
    //     "Card {:?} is not present in 'old' zone",
    //     id
    // ));
    battle.cards.move_card(controller, id.card_id(), old, new);
}
