use action_data::debug_action::DebugAction;
use action_data::game_action::GameAction;
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::{AudioClipAddress, SpriteAddress};
use core_data::identifiers::{BattleId, CardId};
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::CardFacing;
use display_data::battle_view::{
    ActionButtonView, BattlePreviewView, BattleView, DisplayPlayer, InterfaceView,
    PlayerPreviewView, PlayerView,
};
use display_data::card_view::{
    CardActions, CardEffects, CardPrefab, CardPreviewView, CardView, DisplayImage, RevealedCardView,
};
use display_data::object_position::{ObjectPosition, Position};
use masonry::flex_enums::{FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_node::{FlexNode, NodeType, Text};
use masonry::flex_style::{
    BorderRadius, Dimension, DimensionGroup, DimensionUnit, FlexInsets, FlexStyle,
};

pub fn create(id: BattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView {
            score: Points(0),
            can_act: false,
            energy: Energy(2),
            produced_energy: Energy(2),
            total_spark: Spark(0),
        },
        enemy: PlayerView {
            score: Points(0),
            can_act: false,
            energy: Energy(2),
            produced_energy: Energy(2),
            total_spark: Spark(0),
        },
        cards: [
            cards_in_position(Position::InHand(DisplayPlayer::User), 5, 5),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 16, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::User), 22, 20),
            cards_in_position(Position::InHand(DisplayPlayer::Enemy), 105, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::Enemy), 108, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::Enemy), 114, 20),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 150, 4),
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 533, 7),
            cards_in_position(Position::OnBattlefield(DisplayPlayer::Enemy), 633, 8),
            cards_in_position(Position::InHand(DisplayPlayer::Enemy), 733, 5),
            vec![enemy_card(Position::InPlayerStatus(DisplayPlayer::Enemy), 738)],
            vec![dreamsign_card(Position::InPlayerStatus(DisplayPlayer::User), 739)],
            vec![dreamwell_card(Position::InDreamwell(DisplayPlayer::User), 740)],
            vec![dreamwell_card(Position::InDreamwell(DisplayPlayer::Enemy), 741)],
            vec![game_modifier_card(Position::GameModifier, 742)],
        ]
        .concat()
        .to_vec(),
        interface: InterfaceView {
            primary_action_button: Some(ActionButtonView {
                label: "End Turn".to_string(),
                action: Some(GameAction::DebugAction(DebugAction::ApplyTestScenarioAction)),
                show_on_idle_duration: None,
            }),
            ..Default::default()
        },
    }
}

fn cards_in_position(position: Position, start_key: u32, count: u32) -> Vec<CardView> {
    (0..count).map(|i| card_view(position, start_key + i)).collect()
}

pub fn card_view(position: Position, sorting_key: u32) -> CardView {
    let mut card = if sorting_key % 5 == 0 {
        card1(position, sorting_key)
    } else if sorting_key % 5 == 1 {
        card2(position, sorting_key)
    } else if sorting_key % 5 == 2 {
        card3(position, sorting_key)
    } else if sorting_key % 5 == 3 {
        card4(position, sorting_key)
    } else {
        card5(position, sorting_key)
    };

    if position == Position::InHand(DisplayPlayer::Enemy) {
        card.revealed = None;
        card.revealed_to_opponents = false;
    }

    card
}

fn card1(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new("Assets/ThirdParty/GameAssets/CardImages/Standard/2521694543.png"),
            },
            name: "Titan of Forgotten Echoes".to_string(),
            rules_text: "When you \u{f0a3} materialize your second character in a turn, return this character from your void to play.".to_string(),
            outline_color: (position == Position::InHand(DisplayPlayer::User)).then_some(display_color::GREEN),
            cost: Some(Energy(6)),
            produced: None,
            spark: Some(Spark(4)),
            card_type: "Ancient".to_string(),
            supplemental_card_info: flex_node("<b>Materialize</b>: A character entering play."),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                play_effect_preview: Some(BattlePreviewView {
                    preview_message: Some(character_limit_message()),
                    user: PlayerPreviewView {
                        total_spark: Some(Spark(5)),
                        ..Default::default()
                    },
                    cards: vec![
                        CardPreviewView {
                            card_id: CardId::from_int(539),
                            battlefield_icon: Some("\u{f06a}".to_string()),
                            battlefield_icon_color: Some(display_color::RED_900),
                            ..Default::default()
                        }
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Character,
    }
}

fn card2(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/1633431262.png",
                ),
            },
            name: "Beacon of Tomorrow".to_string(),
            rules_text: "Discover a card with cost (2).".to_string(),
            outline_color: (position == Position::InHand(DisplayPlayer::User)).then_some(display_color::GREEN),
            cost: Some(Energy(2)),
            produced: None,
            spark: None,
            card_type: "Event".to_string(),
            supplemental_card_info: flex_node(
                "<b>Discover</b>: Pick one of 4 cards with different types to put into your hand.",
            ),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                on_play_sound: Some(AudioClipAddress::new("Assets/ThirdParty/WowSound/RPG Magic Sound Effects Pack 3/Electric Magic/RPG3_ElectricMagic_Cast01.wav")),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Event,
    }
}

fn card3(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2269064817.png",
                ),
            },
            name: "Scrap Reclaimer".to_string(),
            rules_text: "Judgment: Return this character from your void to your hand. Born from rust and resilience.".to_string(),
            outline_color: (position == Position::InHand(DisplayPlayer::User)).then_some(display_color::GREEN),
            cost: Some(Energy(4)),
            produced: None,
            spark: Some(Spark(0)),
            card_type: "Tinkerer".to_string(),
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of your turn."),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                play_effect_preview: Some(BattlePreviewView {
                    preview_message: Some(hand_size_limit_message()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Event,
    }
}

fn card4(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2269064809.png",
                ),
            },
            name: "Evacuation Enforcer".to_string(),
            rules_text: "> Draw 2 cards. Discard 3 cards.\nPromises under a stormy sky."
                .to_string(),
            outline_color: (position == Position::InHand(DisplayPlayer::User))
                .then_some(display_color::GREEN),
            cost: Some(Energy(2)),
            produced: None,
            spark: Some(Spark(0)),
            card_type: "Trooper".to_string(),
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Character,
    }
}

fn card5(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2027158310.png",
                ),
            },
            name: "Moonlit Voyage".to_string(),
            rules_text: "Draw 2 cards. Discard 2 cards.\nReclaim".to_string(),
            outline_color: (position == Position::InHand(DisplayPlayer::User))
                .then_some(display_color::GREEN),
            cost: Some(Energy(2)),
            produced: None,
            spark: None,
            card_type: "Event".to_string(),
            supplemental_card_info: flex_node(
                "<b>Reclaim</b>: You may play this card from your void, then banish it.",
            ),
            is_fast: false,
            actions: CardActions {
                can_play: position == Position::InHand(DisplayPlayer::User),
                ..Default::default()
            },
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Token,
    }
}

fn enemy_card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: Some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Enemy/Korrak.png",
                ),
            },
            name: "<size=200%>Korrak</size>\nHellfire Sovereign".to_string(),
            rules_text: ">Judgment: A character you control gains +2 spark.".to_string(),
            outline_color: None,
            cost: None,
            produced: None,
            spark: None,
            card_type: "Enemy".to_string(),
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of enemy's turn.",
            ),
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Enemy,
    }
}

fn dreamsign_card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: Some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Dreamsign/DragonEgg.png",
                ),
            },
            name: "Dragon Egg".to_string(),
            rules_text: ">Judgment: If you control 3 characters with the same type, draw a card."
                .to_string(),
            outline_color: None,
            cost: None,
            produced: None,
            spark: None,
            card_type: "Dreamsign".to_string(),
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of enemy's turn.",
            ),
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Dreamsign,
    }
}

fn dreamwell_card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: Some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Dreamwell/1963305268.png",
                ),
            },
            name: "Rising Dawn".to_string(),
            rules_text: "Draw a card.".to_string(),
            outline_color: None,
            cost: None,
            produced: Some(Energy(2)),
            spark: None,
            card_type: "Dreamwell".to_string(),
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Dreamwell,
    }
}

fn game_modifier_card(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                address: SpriteAddress::new(
                    "Assets/ThirdParty/GameAssets/CardImages/Standard/2027158310.png",
                ),
            },
            name: "Celestial Reverie".to_string(),
            rules_text: "Until end of turn, whenever you play a character, draw a card. "
                .to_string(),
            outline_color: None,
            cost: None,
            produced: None,
            spark: None,
            card_type: "Game Modifier".to_string(),
            supplemental_card_info: None,
            is_fast: false,
            actions: CardActions::default(),
            effects: CardEffects::default(),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
        prefab: CardPrefab::Token,
    }
}

fn flex_node(text: impl Into<String>) -> Option<FlexNode> {
    let style = FlexStyle {
        background_color: Some(DisplayColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 2.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 5.0 }),
        white_space: Some(WhiteSpace::Normal),
        ..Default::default()
    };
    Some(FlexNode {
        node_type: Some(NodeType::Text(Text { label: text.into() })),
        style: Some(style),
        ..Default::default()
    })
}

fn hand_size_limit_message() -> FlexNode {
    let style = FlexStyle {
        background_color: Some(DisplayColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
        min_height: Some(Dimension { unit: DimensionUnit::Pixels, value: 22.0 }),
        white_space: Some(WhiteSpace::Normal),
        text_align: Some(TextAlign::MiddleCenter),
        ..Default::default()
    };

    let message = FlexNode {
        node_type: Some(NodeType::Text(Text {
            label: "Note: cards drawn in excess of 10 become \u{f7e4} instead.".into(),
        })),
        style: Some(style),
        ..Default::default()
    };

    FlexNode {
        style: Some(FlexStyle {
            position: Some(FlexPosition::Absolute),
            inset: Some(FlexInsets {
                top: Some(Dimension { unit: DimensionUnit::Pixels, value: 50.0 }),
                right: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
                bottom: None,
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}

fn character_limit_message() -> FlexNode {
    let style = FlexStyle {
        background_color: Some(DisplayColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        color: Some(DisplayColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
        min_height: Some(Dimension { unit: DimensionUnit::Pixels, value: 22.0 }),
        white_space: Some(WhiteSpace::Normal),
        text_align: Some(TextAlign::MiddleCenter),
        ..Default::default()
    };

    let message = FlexNode {
        node_type: Some(NodeType::Text(Text { label: "You have 8 characters in play. A character will be dissolved, with its spark permanently added to your total.".into() })),
        style: Some(style),
        ..Default::default()
    };

    FlexNode {
        style: Some(FlexStyle {
            position: Some(FlexPosition::Absolute),
            inset: Some(FlexInsets {
                top: Some(Dimension { unit: DimensionUnit::Pixels, value: 50.0 }),
                right: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
                bottom: None,
                left: Some(Dimension { unit: DimensionUnit::Pixels, value: 8.0 }),
            }),
            ..Default::default()
        }),
        children: vec![message],
        ..Default::default()
    }
}
