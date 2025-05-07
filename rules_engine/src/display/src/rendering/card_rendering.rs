use action_data::game_action_data::GameAction;
use battle_data::actions::battle_action_data::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_data::CardData;
use battle_data::battle_cards::card_id::{CardIdType, HandCardId};
use battle_data::battle_cards::card_identities;
use battle_data::prompt_types::prompt_data::PromptType;
use battle_queries::legal_action_queries::can_play_card;
use core_data::card_types::CardType;
use core_data::display_color;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::{CardId, CardIdentity};
use core_data::types::CardFacing;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, RevealedCardView,
};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::positions;

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    CardView {
        id: context.card().id.card_id(),
        position: positions::calculate(builder, context.card()),
        revealed: context
            .card()
            .is_revealed_to(builder.display_for_player())
            .then(|| revealed_card_view(builder, context)),
        revealed_to_opponents: context.card().revealed_to_opponent,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Character,
    }
}

fn revealed_card_view(builder: &ResponseBuilder, context: &CardViewContext) -> RevealedCardView {
    let battle = context.battle();
    let card_id = context.card().id.card_id();

    let can_play = context.card().controller() == builder.act_for_player()
        && can_play_card::from_hand(battle, HandCardId(card_id));
    let selection_target = selection_target(builder, context.battle(), card_id);

    RevealedCardView {
        image: DisplayImage { address: card_image(context.card().identity) },
        name: card_name(context.card().identity),
        cost: context.card().properties.cost,
        produced: None,
        spark: context.card().properties.spark,
        card_type: card_type(context.card()),
        rules_text: rules_text(context.card().identity),
        outline_color: match () {
            _ if can_play => Some(display_color::GREEN),
            _ if selection_target.is_some() => Some(display_color::RED_500),
            _ => None,
        },
        supplemental_card_info: None,
        is_fast: false,
        actions: CardActions { can_play, on_click: selection_target, ..Default::default() },
        effects: CardEffects::default(),
    }
}

fn selection_target(
    builder: &ResponseBuilder,
    battle: &BattleData,
    card_id: CardId,
) -> Option<GameAction> {
    if let Some(prompt_data) = &battle.prompt {
        if prompt_data.player == builder.act_for_player() {
            match &prompt_data.prompt_type {
                PromptType::ChooseCharacter { valid } => {
                    return valid.iter().find(|target_id| target_id.card_id() == card_id).map(
                        |&id| GameAction::BattleAction(BattleAction::SelectCharacterTarget(id)),
                    );
                }
                PromptType::ChooseStackCard { valid } => {
                    return valid.iter().find(|target_id| target_id.card_id() == card_id).map(
                        |&id| GameAction::BattleAction(BattleAction::SelectStackCardTarget(id)),
                    );
                }
                _ => {}
            }
        }
    }

    None
}

fn card_image(identity: CardIdentity) -> SpriteAddress {
    match identity {
        card_identities::MINSTREL_OF_FALLING_LIGHT => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1794244540.png",
        ),
        card_identities::IMMOLATE => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1907487244.png",
        ),
        card_identities::RIPPLE_OF_DEFIANCE => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_2123360837.png",
        ),
        card_identities::ABOLISH => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1282908322.png",
        ),
        card_identities::DREAMSCATTER => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_489056605.png",
        ),
        _ => SpriteAddress::new(
            "Assets/ThirdParty/GameAssets/CardImages/Standard/shutterstock_1486924805.png",
        ),
    }
}

fn card_name(identity: CardIdentity) -> String {
    match identity {
        card_identities::MINSTREL_OF_FALLING_LIGHT => "Minstrel of Falling Light".to_string(),
        card_identities::IMMOLATE => "Immolate".to_string(),
        card_identities::RIPPLE_OF_DEFIANCE => "Ripple of Defiance".to_string(),
        card_identities::ABOLISH => "Abolish".to_string(),
        card_identities::DREAMSCATTER => "Dreamscatter".to_string(),
        _ => format!("{:?}", identity),
    }
}

fn card_type(card: &CardData) -> String {
    let result = match card.properties.card_type {
        CardType::Character(t) => t.to_string(),
        CardType::Event => "Event".to_string(),
        CardType::Dreamsign => "Dreamsign".to_string(),
        CardType::Enemy => "Enemy".to_string(),
        CardType::Dreamwell => "Dreamwell".to_string(),
    };

    if card.properties.is_fast {
        format!("\u{f0e7} {}", result)
    } else {
        result
    }
}

fn rules_text(identity: CardIdentity) -> String {
    match identity {
        card_identities::MINSTREL_OF_FALLING_LIGHT => "<i>As the stars wept fire across the sky, he strummed the chords that once taught the heavens to sing.</i>".to_string(),
        card_identities::IMMOLATE => "Dissolve an enemy character.".to_string(),
        card_identities::RIPPLE_OF_DEFIANCE => {
            "Negate an enemy event unless they pay 2\u{f7e4}.".to_string()
        }
        card_identities::ABOLISH => "Negate an enemy card".to_string(),
        card_identities::DREAMSCATTER => {
            "Pay one or more \u{f7e4}: Draw a card for each \u{f7e4} spent.".to_string()
        }
        _ => format!("{:?}", identity),
    }
}
