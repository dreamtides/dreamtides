use action_data::game_action_data::GameAction;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::LegalActions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, CharacterId, HandCardId, StackCardId};
use core_data::card_types::CardType;
use core_data::display_color;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::CardName;
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, RevealedCardView,
};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::positions::ControllerAndZone;
use crate::rendering::{card_display_state, positions};

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    CardView {
        id: context.card_id(),
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
    let can_play = legal_actions.contains(BattleAction::PlayCardFromHand(HandCardId(card_id)));
    let selection_action = selection_action(&legal_actions, card_id);
    let ControllerAndZone { controller, .. } = positions::controller_and_zone(battle, card_id);

    RevealedCardView {
        image: DisplayImage { address: card_image(battle, card_id) },
        name: card_name(battle, card_id),
        cost: card_properties::cost(battle, card_id),
        produced: None,
        spark: card_properties::spark(battle, controller, CharacterId(card_id)),
        card_type: card_type(battle, card_id),
        rules_text: rules_text(battle, card_id),
        outline_color: match () {
            _ if can_play => Some(display_color::GREEN),
            _ if selection_action.is_some() => Some(display_color::RED_500),
            _ => None,
        },
        supplemental_card_info: None,
        is_fast: false,
        actions: CardActions { can_play, on_click: selection_action, ..Default::default() },
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
    match battle.cards.card(card_id).name {
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
    }
}

fn card_name(battle: &BattleState, card_id: CardId) -> String {
    match battle.cards.card(card_id).name {
        CardName::MinstrelOfFallingLight => "Minstrel of Falling Light".to_string(),
        CardName::Immolate => "Immolate".to_string(),
        CardName::RippleOfDefiance => "Ripple of Defiance".to_string(),
        CardName::Abolish => "Abolish".to_string(),
        CardName::Dreamscatter => "Dreamscatter".to_string(),
    }
}

fn card_type(battle: &BattleState, card_id: CardId) -> String {
    let result = match card_properties::card_type(battle, card_id) {
        CardType::Character(t) => t.to_string(),
        CardType::Event => "Event".to_string(),
        CardType::Dreamsign => "Dreamsign".to_string(),
        CardType::Enemy => "Enemy".to_string(),
        CardType::Dreamwell => "Dreamwell".to_string(),
    };

    if card_properties::is_fast(battle, card_id) {
        format!("\u{f0e7} {}", result)
    } else {
        result
    }
}

fn rules_text(battle: &BattleState, card_id: CardId) -> String {
    match battle.cards.card(card_id).name {
        CardName::MinstrelOfFallingLight => "<i>As the stars wept fire across the sky, he strummed the chords that once taught the heavens to sing.</i>".to_string(),
        CardName::Immolate => "Dissolve an enemy character.".to_string(),
        CardName::RippleOfDefiance => {
            "Negate an enemy event unless they pay 2\u{f7e4}.".to_string()
        }
        CardName::Abolish => "Negate an enemy card".to_string(),
        CardName::Dreamscatter => {
            "Pay one or more \u{f7e4}: Draw a card for each \u{f7e4} spent.".to_string()
        }
    }
}
