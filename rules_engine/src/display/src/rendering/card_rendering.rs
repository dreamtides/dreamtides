use battle_data::battle_cards::zone::Zone;
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
    let can_play = context.card().owner == PlayerName::User && context.card().zone() == Zone::Hand;
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
