use std::collections::HashMap;

use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::{card, card_properties, stack_card_queries};
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CardIdType, CharacterId, HandCardId, StackCardId};
use battle_state::battle_cards::stack_card_state::{
    StackCardAdditionalCostsPaid, StackCardTargets,
};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::card_types::CardType;
use core_data::display_color;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::CardName;
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, InfoZoomData, InfoZoomIcon,
    RevealedCardView,
};
use ui_components::component::Component;
use ui_components::icon;

use crate::core::adapter;
use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::display_actions::outcome_simulation;
use crate::rendering::positions::ControllerAndZone;
use crate::rendering::supplemental_card_info::SupplementalCardInfo;
use crate::rendering::{card_display_state, positions};

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    CardView {
        id: adapter::client_card_id(context.card_id()),
        position: positions::calculate(builder, context.battle(), context.card_id()),
        revealed: card_display_state::is_revealed_to(
            context.battle(),
            context.card_id(),
            builder.display_for_player(),
        )
        .then(|| revealed_card_view(builder, context)),
        revealed_to_opponents: card_display_state::is_revealed_to(
            context.battle(),
            context.card_id(),
            builder.display_for_player().opponent(),
        ),
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Character,
    }
}

fn revealed_card_view(builder: &ResponseBuilder, context: &CardViewContext) -> RevealedCardView {
    let battle = context.battle();
    let card_id = context.card_id();
    let legal_actions = legal_actions::compute(battle, builder.act_for_player());
    let play = BattleAction::PlayCardFromHand(HandCardId(card_id));
    let play_action = legal_actions.contains(play).then_some(GameAction::BattleAction(play));
    let can_play = play_action.is_some();
    let selection_action = selection_action(&legal_actions, card_id);
    let ControllerAndZone { controller, .. } = positions::controller_and_zone(battle, card_id);

    RevealedCardView {
        image: DisplayImage::Sprite(card_image(battle, card_id)),
        name: card_name(battle, card_id),
        cost: card_properties::cost(battle, card_id),
        produced: None,
        spark: card_properties::spark(battle, controller, CharacterId(card_id))
            .or_else(|| card_properties::base_spark(battle, card_id)),
        card_type: card_type(battle, card_id),
        rules_text: rules_text(battle, card_id),
        outline_color: match () {
            _ if can_play => Some(display_color::GREEN),
            _ if selection_action.is_some() => Some(display_color::RED_500),
            _ => None,
        },
        info_zoom_data: build_info_zoom_data(battle, card_id),
        is_fast: false,
        actions: CardActions {
            can_play: play_action,
            on_click: selection_action,
            play_effect_preview: if can_play {
                Some(outcome_simulation::action_effect_preview(
                    battle,
                    builder.act_for_player(),
                    BattleAction::PlayCardFromHand(HandCardId(card_id)),
                ))
            } else {
                None
            },
            ..Default::default()
        },
        effects: CardEffects::default(),
    }
}

fn selection_action(legal_actions: &LegalActions, card_id: CardId) -> Option<GameAction> {
    if legal_actions.contains(BattleAction::SelectCharacterTarget(CharacterId(card_id))) {
        return Some(GameAction::BattleAction(BattleAction::SelectCharacterTarget(CharacterId(
            card_id,
        ))));
    }

    if legal_actions.contains(BattleAction::SelectStackCardTarget(StackCardId(card_id))) {
        return Some(GameAction::BattleAction(BattleAction::SelectStackCardTarget(StackCardId(
            card_id,
        ))));
    }

    None
}

fn card_image(battle: &BattleState, card_id: CardId) -> SpriteAddress {
    match card::get(battle, card_id).name {
        CardName::MinstrelOfFallingLight => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        CardName::Immolate => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1907487244.png",
        ),
        CardName::RippleOfDefiance => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_2123360837.png",
        ),
        CardName::Abolish => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1282908322.png",
        ),
        CardName::Dreamscatter => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        CardName::TestDrawOne => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
    }
}

fn card_name(battle: &BattleState, card_id: CardId) -> String {
    card_properties::display_name(card::get(battle, card_id).name)
}

fn card_type(battle: &BattleState, card_id: CardId) -> String {
    let result = match card_properties::card_type(battle, card_id) {
        CardType::Character(t) => t.to_string(),
        CardType::Event => "Event".to_string(),
        CardType::Dreamsign => "Dreamsign".to_string(),
        CardType::Enemy => "Enemy".to_string(),
        CardType::Dreamwell => "Dreamwell".to_string(),
    };

    if card_properties::is_fast(battle, card_id) { format!("\u{f0e7} {result}") } else { result }
}

fn rules_text(battle: &BattleState, card_id: CardId) -> String {
    let base_text = match card::get(battle, card_id).name {
        CardName::MinstrelOfFallingLight => "<i>As the stars wept fire across the sky, he strummed the chords that once taught the heavens to sing.</i>".to_string(),
        CardName::Immolate => "<b>Dissolve</b> an enemy character.".to_string(),
        CardName::RippleOfDefiance => {
            "<b>Prevent</b> an enemy event unless the enemy pays 2\u{f7e4}.".to_string()
        }
        CardName::Abolish => "<b>Prevent</b> an enemy card.".to_string(),
        CardName::Dreamscatter => {
            "Pay one or more \u{f7e4}: Draw a card for each \u{f7e4} spent.".to_string()
        }
        CardName::TestDrawOne => "Draw a card.".to_string(),
    };

    if card::get(battle, card_id).name == CardName::Dreamscatter
        && let Some(stack_card) = battle.cards.stack_card(StackCardId(card_id))
        && let StackCardAdditionalCostsPaid::Energy(energy) = &stack_card.additional_costs_paid
    {
        return format!("{} <b><color=\"blue\">({}\u{f7e4} paid)</color></b>", base_text, energy.0);
    }

    base_text
}

fn supplemental_card_info(battle: &BattleState, card_id: CardId) -> Option<String> {
    match card::get(battle, card_id).name {
        CardName::Immolate => Some("<b>Dissolve:</b> Send a character to the void".to_string()),
        CardName::RippleOfDefiance => Some(
            "<b>Prevent:</b> Send a card to the void in response to it being played".to_string(),
        ),
        CardName::Abolish => Some(
            "<b>Prevent:</b> Send a card to the void in response to it being played".to_string(),
        ),
        _ => None,
    }
}

fn build_info_zoom_data(battle: &BattleState, card_id: CardId) -> Option<InfoZoomData> {
    let targeting_icons = get_targeting_icons(battle, card_id);
    let supplemental_info = supplemental_card_info(battle, card_id)
        .and_then(|text| SupplementalCardInfo::builder().text(text).build().render()?.flex_node());

    if targeting_icons.is_empty() && supplemental_info.is_none() {
        None
    } else {
        Some(InfoZoomData { supplemental_card_info: supplemental_info, icons: targeting_icons })
    }
}

fn get_targeting_icons(battle: &BattleState, card_id: CardId) -> Vec<InfoZoomIcon> {
    let mut icons = HashMap::new();

    if let Some(prompt) = &battle.prompt
        && prompt.source.card_id() == Some(card_id)
        && let PromptType::Choose { choices } = &prompt.prompt_type
    {
        // This card is currently the source of a choice prompt, check for
        // effect targets.
        for choice in choices {
            if let Some(targets) = &choice.targets {
                let target_card_id = match targets {
                    StackCardTargets::Character(target_character_id, _) => {
                        target_character_id.card_id()
                    }
                    StackCardTargets::StackCard(target_stack_card_id, _) => {
                        target_stack_card_id.card_id()
                    }
                };

                icons.insert(target_card_id, InfoZoomIcon {
                    card_id: adapter::client_card_id(target_card_id),
                    icon: icon::CHEVRON_UP.to_string(),
                    color: display_color::RED_500,
                });
            }
        }
    }

    if let Some(targets) = stack_card_queries::targets(battle, card_id) {
        // This card is currently on the stack with targets.
        match targets {
            StackCardTargets::Character(target_character_id, _) => {
                icons.insert(target_character_id.card_id(), InfoZoomIcon {
                    card_id: adapter::client_card_id(target_character_id.card_id()),
                    icon: icon::CHEVRON_UP.to_string(),
                    color: display_color::RED_500,
                });
            }
            StackCardTargets::StackCard(target_stack_card_id, _) => {
                icons.insert(target_stack_card_id.card_id(), InfoZoomIcon {
                    card_id: adapter::client_card_id(target_stack_card_id.card_id()),
                    icon: icon::CHEVRON_UP.to_string(),
                    color: display_color::RED_500,
                });
            }
        }
    } else if let Some(stack_card) = battle.cards.stack_card(StackCardId(card_id))
        && stack_card.targets.is_some()
        && stack_card_queries::validate_targets(battle, stack_card.targets.as_ref()).is_none()
    {
        // This card is currently on the stack with invalid targets.
        icons.insert(card_id, InfoZoomIcon {
            card_id: adapter::client_card_id(card_id),
            icon: icon::XMARK.to_string(),
            color: display_color::RED_500,
        });
    }

    icons.into_values().collect()
}
