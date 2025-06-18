use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use core_data::display_types::PrefabAddress;
use core_data::types::{CardFacing, PlayerName};
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, RevealedCardView,
};
use display_data::object_position::{ObjectPosition, Position};

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;

pub fn identity_card_view(
    builder: &ResponseBuilder,
    _battle: &BattleState,
    player: PlayerName,
) -> CardView {
    let card_id = CardId(if player == builder.display_for_player() { 1000000 } else { 1000001 });

    CardView {
        id: adapter::client_card_id(card_id),
        position: ObjectPosition {
            position: Position::Browser,
            sorting_key: 0,
            sorting_sub_key: 0,
        },
        revealed: Some(RevealedCardView {
            image: DisplayImage::Prefab(PrefabAddress::new(
                "Assets/Content/Characters/PirateCaptain/PirateCaptain.prefab",
            )),
            name: "Blackbeard\n<size=75%>Cunning Navigator</size>".to_string(),
            cost: None,
            produced: None,
            spark: None,
            card_type: "Identity".to_string(),
            rules_text:
                "At the end of your turn, if you played no characters this turn, draw a card."
                    .to_string(),
            outline_color: None,
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
            info_zoom_data: None,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Identity,
    }
}
