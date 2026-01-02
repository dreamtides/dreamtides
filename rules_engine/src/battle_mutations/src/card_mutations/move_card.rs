use battle_queries::battle_card_queries::{card, card_properties};
use battle_queries::panic_with;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{
    BattleDeckCardId, CardId, CardIdType, CharacterId, HandCardId, StackCardId, VoidCardId,
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
    card_id: BattleDeckCardId,
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
) -> BattleDeckCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Hand, Zone::Deck);
    BattleDeckCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's hand to their void.
///
/// Panics if this card is not found in the hand.
pub fn from_hand_to_void(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: HandCardId,
) -> VoidCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Hand, Zone::Void);
    VoidCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's deck to their battlefield.
///
/// Panics if this card is not found in the deck.
pub fn from_deck_to_battlefield(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: BattleDeckCardId,
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
    card_id: BattleDeckCardId,
) -> VoidCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Deck, Zone::Void);
    VoidCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's void to their hand.
///
/// Panics if this card is not found in the void.
pub fn from_void_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: VoidCardId,
) -> HandCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Void, Zone::Hand);

    // Mark the card as revealed to the opponent
    let card_state = card::get_mut(battle, card_id.card_id());
    *card_state.revealed_to_player_override.player_mut(controller.opponent()) = true;

    HandCardId(card_id.card_id())
}

/// Moves a card from the 'controller' player's void to their deck.
///
/// Panics if this card is not found in the void.
pub fn from_void_to_deck(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: VoidCardId,
) -> BattleDeckCardId {
    to_destination_zone(battle, source, controller, card_id.card_id(), Zone::Void, Zone::Deck);
    BattleDeckCardId(card_id.card_id())
}

/// Moves a character from the 'controller' player's battlefield to their hand.
///
/// Panics if this character is not found.
pub fn from_battlefield_to_hand(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: CharacterId,
) -> HandCardId {
    to_destination_zone(
        battle,
        source,
        controller,
        card_id.card_id(),
        Zone::Battlefield,
        Zone::Hand,
    );

    // Mark the card as revealed to the opponent
    let card_state = card::get_mut(battle, card_id.card_id());
    *card_state.revealed_to_player_override.player_mut(controller.opponent()) = true;

    HandCardId(card_id.card_id())
}

/// Moves a card from the 'old' zone to the 'new' zone.
///
/// Panics if this card is not found in the 'old' zone.
fn to_destination_zone(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    id: impl CardIdType,
    old: Zone,
    mut new: Zone,
) {
    let card_id = id.card_id();
    if !battle.cards.contains_card(controller, card_id, old) {
        panic_card_not_found(battle, controller, card_id, old, new);
    }

    match old {
        Zone::Stack => on_leave_stack(battle, card_id, &mut new),
        Zone::Battlefield => on_leave_battlefield(battle, controller, card_id, &mut new),
        _ => {}
    }

    battle.cards.move_card(controller, card_id, old, new);

    if new == Zone::Void {
        battle.triggers.push(source, Trigger::PutIntoVoid(VoidCardId(card_id)));
    }

    match new {
        Zone::Stack => on_enter_stack(battle, card_id),
        Zone::Battlefield => on_enter_battlefield(battle, source, controller, card_id),
        _ => {}
    }
}

fn on_enter_battlefield(
    battle: &mut BattleState,
    source: EffectSource,
    controller: PlayerName,
    card_id: CardId,
) {
    let ability_list = card::ability_list(battle, card_id);
    let triggers = ability_list.battlefield_triggers;
    for trigger in triggers {
        battle.triggers.listeners.add_listener(trigger, card_id);
    }

    let id = CharacterId(card_id);
    let Some(spark) = card_properties::base_spark(battle, id) else {
        panic_no_base_spark(battle, id);
    };
    battle.cards.battlefield_state_mut(controller).insert(id, CharacterState { spark });

    battle.triggers.push(source, Trigger::Materialized(id));
}

fn on_leave_battlefield(
    battle: &mut BattleState,
    controller: PlayerName,
    card_id: CardId,
    new: &mut Zone,
) {
    let ability_list = card::ability_list(battle, card_id);
    let triggers = ability_list.battlefield_triggers;
    for trigger in triggers {
        battle.triggers.listeners.remove_listener(trigger, card_id);
    }

    battle.cards.battlefield_state_mut(controller).remove(&CharacterId(card_id));

    if battle.ability_state.banish_when_leaves_play.contains(card_id) {
        battle.ability_state.banish_when_leaves_play.remove(card_id);
        *new = Zone::Banished;
    }
}

fn on_enter_stack(battle: &mut BattleState, card_id: CardId) {
    let ability_list = card::ability_list(battle, card_id);
    let triggers = ability_list.stack_triggers;
    for trigger in triggers {
        battle.triggers.listeners.add_listener(trigger, card_id);
    }
}

fn on_leave_stack(battle: &mut BattleState, card_id: CardId, new: &mut Zone) {
    let ability_list = card::ability_list(battle, card_id);
    let triggers = ability_list.stack_triggers;
    for trigger in triggers {
        battle.triggers.listeners.remove_listener(trigger, card_id);
    }

    if *new != Zone::Battlefield && battle.ability_state.banish_when_leaves_play.contains(card_id) {
        battle.ability_state.banish_when_leaves_play.remove(card_id);
        *new = Zone::Banished;
    }
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
