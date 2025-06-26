use battle_queries::battle_card_queries::{card, stack_card_queries};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType};
use battle_state::battle_cards::stack_card_state::StackCardTargets;
use battle_state::battle_cards::zone::Zone;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;
use display_data::object_position::{ObjectPosition, Position, StackType};

use crate::core::response_builder::ResponseBuilder;
use crate::rendering::position_overrides;

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
        Zone::Stack => Position::OnStack(current_stack_type(builder, battle)),
        Zone::Void => Position::InVoid(player),
        Zone::Banished => Position::InBanished(player),
    };

    for_card(battle, card_id, position_overrides::position(builder, battle, card_id, position))
}

pub fn for_card(battle: &BattleState, card_id: CardId, position: Position) -> ObjectPosition {
    let object_id = card::get(battle, card_id).object_id;
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
        panic!("Card not found in any zone: {card_id:?}")
    }
}

pub fn current_stack_type(builder: &ResponseBuilder, battle: &BattleState) -> StackType {
    let display_player = builder.display_for_player();
    let mut targeting_user_battlefield = false;
    let mut targeting_enemy_battlefield = false;

    for stack_card in battle.cards.all_cards_on_stack() {
        if let Some(StackCardTargets::Character(character_id, _)) =
            stack_card_queries::targets(battle, stack_card.id)
        {
            if battle.cards.contains_card(display_player, character_id.card_id(), Zone::Battlefield)
            {
                targeting_user_battlefield = true;
            } else if battle.cards.contains_card(
                display_player.opponent(),
                character_id.card_id(),
                Zone::Battlefield,
            ) {
                targeting_enemy_battlefield = true;
            }
        }
    }

    if let Some(ref prompt) = battle.prompt {
        if let PromptType::ChooseCharacter { ref valid } = prompt.prompt_type {
            for character_id in valid.iter() {
                if battle.cards.contains_card(
                    display_player,
                    character_id.card_id(),
                    Zone::Battlefield,
                ) {
                    targeting_user_battlefield = true;
                } else if battle.cards.contains_card(
                    display_player.opponent(),
                    character_id.card_id(),
                    Zone::Battlefield,
                ) {
                    targeting_enemy_battlefield = true;
                }
            }
        }
    }

    if targeting_user_battlefield && targeting_enemy_battlefield {
        StackType::TargetingBothBattlefields
    } else if targeting_user_battlefield {
        StackType::TargetingUserBattlefield
    } else if targeting_enemy_battlefield {
        StackType::TargetingEnemyBattlefield
    } else {
        StackType::Default
    }
}
