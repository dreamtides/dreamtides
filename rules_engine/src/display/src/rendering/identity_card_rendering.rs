use battle_state::battle::battle_state::BattleState;
use core_data::display_types::PrefabAddress;
use core_data::types::{CardFacing, PlayerName};
use display_data::battle_view::DisplayPlayer;
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardView, DisplayImage, DisplayPrefabImage,
    RevealedCardView,
};
use display_data::command::StudioType;
use display_data::object_position::{ObjectPosition, Position};

use crate::core::response_builder::ResponseBuilder;
use crate::rendering::position_overrides;

pub fn identity_card_view(
    builder: &ResponseBuilder,
    _battle: &BattleState,
    player: PlayerName,
) -> CardView {
    let name = builder.to_display_player(player);
    let position = Position::InPlayerStatus(name);
    CardView {
        id: format!("{name:?}"),
        position: ObjectPosition {
            position: position_overrides::for_browser(builder, position),
            sorting_key: 0,
            sorting_sub_key: 0,
        },
        revealed: Some(RevealedCardView {
            image: DisplayImage::Prefab(DisplayPrefabImage {
                prefab: PrefabAddress::new(match name {
                    DisplayPlayer::User => {
                        "Assets/Content/Characters/PirateCaptain/PirateCaptain.prefab"
                    }
                    DisplayPlayer::Enemy => {
                        "Assets/Content/Characters/WarriorKing/WarriorKing.prefab"
                    }
                }),
                studio_type: match name {
                    DisplayPlayer::User => StudioType::UserIdentityCard,
                    DisplayPlayer::Enemy => StudioType::EnemyIdentityCard,
                },
            }),
            name: match name {
                DisplayPlayer::User => "Blackbeard\n<size=75%>Cunning Navigator</size>".to_string(),
                DisplayPlayer::Enemy => {
                    "The Black Knight\n<size=75%>Malignant Usurper</size>".to_string()
                }
            },
            cost: None,
            produced: None,
            spark: None,
            card_type: "Identity".to_string(),
            rules_text: match name {
                DisplayPlayer::User => {
                    "At the end of your turn, if you played no characters this turn, draw a card."
                        .to_string()
                }
                DisplayPlayer::Enemy => {
                    "Whenever you discard your second card in a turn, draw a card.".to_string()
                }
            },
            outline_color: None,
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
            info_zoom_data: None,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        backless: true,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Identity,
    }
}
