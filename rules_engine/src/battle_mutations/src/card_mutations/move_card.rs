use battle_queries::battle_card_queries::{card_abilities, card_properties};
use battle_queries::panic_with;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{
    CardId, CardIdType, CharacterId, DeckCardId, HandCardId, StackCardId, VoidCardId,
};
use battle_state::battle_cards::character_state::CharacterState;
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

/// Moves a card from the 'controller' player's hand to the stack.
///
/// Panics if this card is not found in hand.
pub fn from_hand_to_stack(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: HandCardId,
) -> StackCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Hand, Zone::Stack);
    StackCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's void to the stack.
///
/// Panics if this card is not found in void.
pub fn from_void_to_stack(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: VoidCardId,
) -> StackCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Void, Zone::Stack);
    StackCardId(card_id.card_id())
}

/// Moves a card from the stack to the 'controller' player's battlefield.
///
/// Panics if this card is not found on the stack.
pub fn from_stack_to_battlefield(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: StackCardId,
) -> CharacterId {
    to_destination_zone(
        battle,
        source,
        controller,
        card_id.card_id(),
        Zone::Stack,
        Zone::Battlefield,
    );
    CharacterId(card_id.card_id())
}

/// Moves a card from the stack to the 'controller' player's void.
///
/// Panics if this card is not found on the stack.
pub fn from_stack_to_void(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: StackCardId,
) -> VoidCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Stack, Zone::Void);
    VoidCardId(card_id.card_id())
}

/// Moves a character from the 'controller' player's battlefield to the void.
///
/// Panics if this character is not found.
pub fn from_battlefield_to_void(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: CharacterId,
) -> VoidCardId {
    to_destination_zone(
        battle,
        source,
        controller,
        card_id.card_id(),
        Zone::Battlefield,
        Zone::Void,
    );
    VoidCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's deck to their hand.
///
/// Panics if this card is not found in the deck.
pub fn from_deck_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: DeckCardId,
) -> HandCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Deck, Zone::Hand);
    HandCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's hand to their deck.
///
/// Panics if this card is not found in the hand.
pub fn from_hand_to_deck(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: HandCardId,
) -> DeckCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Hand, Zone::Deck);
    DeckCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's deck to their battlefield.
///
/// Panics if this card is not found in the deck.
pub fn from_deck_to_battlefield(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: DeckCardId,
) -> CharacterId {
    to_destination_zone(
        battle,
        source,
        controller,
        card_id.card_id(),
        Zone::Deck,
        Zone::Battlefield,
    );
    CharacterId(card_id.card_id())
}

/// Moves a card from the 'controller' player's deck to their void.
///
/// Panics if this card is not found in the deck.
pub fn from_deck_to_void(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: DeckCardId,
) -> VoidCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Deck, Zone::Void);
    VoidCardId(card_id.card_id())
}

/// Moves a card from the 'old' zone to the 'new' zone.
///
/// Panics if this card is not found in the 'old' zone.
pub fn to_destination_zone(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    id: impl CardIdType,
    old: Zone,
    new: Zone,
) {
    let card_id = id.card_id();
    if !battle.cards.contains_card(controller, card_id, old) {
        panic_card_not_found(battle, controller, card_id, old, new);
    }

    if old == Zone::Battlefield {
        on_leave_battlefield(battle, controller, card_id);
    }

    battle.cards.move_card(controller, card_id, old, new);

    if new == Zone::Battlefield {
        on_enter_battlefield(battle, source, controller, card_id);
    }
}

fn on_leave_battlefield(battle: &mut BattleState, controller: PlayerName, card_id: CardId) {
    let ability_list = card_abilities::query(battle, card_id);
    let triggers = ability_list.battlefield_triggers;
    for trigger in triggers {
        battle.triggers.listeners.remove_listener(trigger, card_id);
    }

    if ability_list.has_battlefield_activated_abilities {
        battle.activated_abilities.player_mut(controller).characters.remove(CharacterId(card_id));
    }

    battle.cards.battlefield_state_mut(controller).remove(&CharacterId(card_id));
}

fn on_enter_battlefield(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: CardId,
) {
    let ability_list = card_abilities::query(battle, card_id);
    let triggers = ability_list.battlefield_triggers;
    for trigger in triggers {
        battle.triggers.listeners.add_listener(trigger, card_id);
    }

    let id = CharacterId(card_id);
    let Some(spark) = card_properties::base_spark(battle, id) else {
        panic_no_base_spark(battle, id);
    };
    battle.cards.battlefield_state_mut(controller).insert(id, CharacterState { spark });

    if ability_list.has_battlefield_activated_abilities {
        battle.activated_abilities.player_mut(controller).characters.insert(id);
    }

    battle.triggers.push(source, Trigger::Materialized(id));
}

#[cold]
fn panic_card_not_found(
    battle: &BattleState,
    controller: PlayerName,
    id: CardId,
    old: Zone,
    new: Zone,
) -> ! {
    panic_with!("Card not found", battle, controller, id, old, new);
}

#[cold]
fn panic_no_base_spark(battle: &BattleState, id: CharacterId) -> ! {
    panic_with!("Character has no base spark value", battle, id);
}
