use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use battle_state::battle_cards::zone::Zone;
use core_data::types::PlayerName;
use display_data::object_position::{ObjectPosition, Position, StackType};

use crate::core::response_builder::ResponseBuilder;

pub fn calculate(
    builder: &ResponseBuilder,
    battle: &BattleState,
    card_id: CardId,
) -> ObjectPosition {
    let ControllerAndZone { controller, zone } = controller_and_zone(battle, card_id);
    let player = builder.to_display_player(controller);
    let position = match zone {
        Zone::Hand => Position::InHand(player),
        Zone::Deck => Position::InDeck(player),
        Zone::Battlefield => Position::OnBattlefield(player),
        Zone::Stack => Position::OnStack(StackType::Default),
        Zone::Void => Position::InVoid(player),
        Zone::Banished => Position::InBanished(player),
    };

    for_card(battle, card_id, position)
}

pub fn for_card(battle: &BattleState, card_id: CardId, position: Position) -> ObjectPosition {
    let object_id = battle.cards.card(card_id).object_id;
    ObjectPosition { position, sorting_key: object_id.0 as u32, sorting_sub_key: 0 }
}

pub struct ControllerAndZone {
    pub controller: PlayerName,
    pub zone: Zone,
}

pub fn controller_and_zone(battle: &BattleState, card_id: CardId) -> ControllerAndZone {
    if battle.cards.contains_card(PlayerName::One, card_id, Zone::Battlefield) {
        ControllerAndZone { controller: PlayerName::One, zone: Zone::Battlefield }
    } else if battle.cards.contains_card(PlayerName::Two, card_id, Zone::Battlefield) {
        ControllerAndZone { controller: PlayerName::Two, zone: Zone::Battlefield }
    } else if battle.cards.contains_card(PlayerName::One, card_id, Zone::Hand) {
        ControllerAndZone { controller: PlayerName::One, zone: Zone::Hand }
    } else if battle.cards.contains_card(PlayerName::Two, card_id, Zone::Hand) {
        ControllerAndZone { controller: PlayerName::Two, zone: Zone::Hand }
    } else if battle.cards.contains_card(PlayerName::One, card_id, Zone::Deck) {
        ControllerAndZone { controller: PlayerName::One, zone: Zone::Deck }
    } else if battle.cards.contains_card(PlayerName::Two, card_id, Zone::Deck) {
        ControllerAndZone { controller: PlayerName::Two, zone: Zone::Deck }
    } else if battle.cards.contains_card(PlayerName::One, card_id, Zone::Void) {
        ControllerAndZone { controller: PlayerName::One, zone: Zone::Void }
    } else if battle.cards.contains_card(PlayerName::Two, card_id, Zone::Void) {
        ControllerAndZone { controller: PlayerName::Two, zone: Zone::Void }
    } else if battle.cards.contains_card(PlayerName::One, card_id, Zone::Banished) {
        ControllerAndZone { controller: PlayerName::One, zone: Zone::Banished }
    } else if battle.cards.contains_card(PlayerName::Two, card_id, Zone::Banished) {
        ControllerAndZone { controller: PlayerName::Two, zone: Zone::Banished }
    } else {
        for stack_card in battle.cards.all_cards_on_stack() {
            if stack_card.id.0 == card_id {
                return ControllerAndZone { controller: stack_card.controller, zone: Zone::Stack };
            }
        }
        panic!("Card not found in any zone: {:?}", card_id)
    }
}
