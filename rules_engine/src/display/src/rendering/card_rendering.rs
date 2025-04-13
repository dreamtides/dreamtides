use action_data::battle_action::BattleAction;
use battle_queries::legal_action_queries::legal_actions;
use core_data::display_color;
use core_data::display_types::SpriteAddress;
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
        revealed_to_opponents: false,
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

    RevealedCardView {
        image: DisplayImage {
            address: SpriteAddress::new(
                "Assets/ThirdParty/GameAssets/CardImages/Standard/2521694543.png",
            ),
        },
        name: "Name".to_string(),
        cost: context.card().properties.cost,
        produced: None,
        spark: context.card().properties.spark,
        card_type: "Character".to_string(),
        rules_text: "Text".to_string(),
        outline_color: can_play.then_some(display_color::GREEN),
        supplemental_card_info: None,
        is_fast: false,
        actions: CardActions { can_play, ..Default::default() },
        effects: CardEffects::default(),
    }
}
