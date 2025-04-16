use action_data::battle_action::BattleAction;
use action_data::game_action::GameAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::prompts::prompt_data::Prompt;
use battle_queries::legal_action_queries::legal_actions;
use core_data::display_color;
use core_data::display_types::SpriteAddress;
use core_data::identifiers::CardId;
use core_data::types::{CardFacing, PlayerName};
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, RevealedCardView,
};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::positions;

pub fn card_view(builder: &ResponseBuilder, context: &CardViewContext) -> CardView {
    CardView {
        id: context.card().id.card_identifier_for_display(),
        position: positions::calculate(context.card()),
        revealed: context
            .card()
            .is_revealed_to(PlayerName::User)
            .then(|| revealed_card_view(builder, context)),
        revealed_to_opponents: context.card().revealed_to_opponent,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Character,
    }
}

fn revealed_card_view(_builder: &ResponseBuilder, context: &CardViewContext) -> RevealedCardView {
    let battle = context.battle();
    let card_id = context.card().id.card_identifier_for_display();

    let can_play =
        legal_actions::compute(battle, PlayerName::User, legal_actions::LegalActions::default())
            .into_iter()
            .any(|action| matches!(action, BattleAction::PlayCard(id) if id == card_id));
    let can_select = is_selection_target(context.battle(), card_id);

    RevealedCardView {
        image: DisplayImage {
            address: SpriteAddress::new(
                "Assets/ThirdParty/GameAssets/CardImages/Standard/2521694543.png",
            ),
        },
        name: format!("{:?}", context.card().id.card_identifier_for_display()),
        cost: context.card().properties.cost,
        produced: None,
        spark: context.card().properties.spark,
        card_type: "Character".to_string(),
        rules_text: format!("{:?}", context.card().abilities),
        outline_color: match () {
            _ if can_play => Some(display_color::GREEN),
            _ if can_select => Some(display_color::RED_500),
            _ => None,
        },
        supplemental_card_info: None,
        is_fast: false,
        actions: CardActions {
            can_play,
            on_click: can_select
                .then_some(GameAction::BattleAction(BattleAction::SelectCard(card_id))),
            ..Default::default()
        },
        effects: CardEffects::default(),
    }
}

fn is_selection_target(battle: &BattleData, card_id: CardId) -> bool {
    if let Some(prompt_data) = &battle.prompt {
        if prompt_data.player == PlayerName::User {
            match &prompt_data.prompt {
                Prompt::ChooseCharacter { valid } => {
                    return valid.iter().any(|target_id| target_id.0.card_id == card_id)
                }
            }
        }
    }

    false
}
