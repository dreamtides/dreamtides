use std::sync::{LazyLock, Mutex};

use action_data::battle_action::BattleAction;
use action_data::debug_action::DebugAction;
use action_data::user_action::UserAction;
use core_data::identifiers::{BattleId, CardId};
use core_data::numerics::{Energy, Points, Spark};
use core_data::types::{CardFacing, Url};
use display_data::battle_view::{BattleView, DisplayPlayer, PlayerView};
use display_data::card_view::{CardFrame, CardView, DisplayImage, RevealedCardView};
use display_data::command::{Command, CommandSequence};
use display_data::object_position::{ObjectPosition, Position};
use display_data::request_data::{
    ConnectRequest, ConnectResponse, Metadata, PerformActionRequest, PerformActionResponse,
};
use masonry::flex_enums::WhiteSpace;
use masonry::flex_node::{FlexNode, NodeType, Text};
use masonry::flex_style::{
    BorderRadius, Dimension, DimensionGroup, DimensionUnit, FlexColor, FlexStyle,
};
use uuid::Uuid;

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleView>>> = LazyLock::new(|| Mutex::new(None));

pub fn connect(request: &ConnectRequest) -> ConnectResponse {
    let battle = scene_0(BattleId(Uuid::new_v4()));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
    ConnectResponse {
        metadata: request.metadata,
        commands: CommandSequence::from_command(Command::UpdateBattle(battle)),
    }
}

pub fn perform_action(request: &PerformActionRequest) -> PerformActionResponse {
    match request.action {
        UserAction::DebugAction(action) => perform_debug_action(action, request.metadata),
        UserAction::BattleAction(action) => perform_battle_action(action, request.metadata),
    }
}

fn perform_debug_action(action: DebugAction, metadata: Metadata) -> PerformActionResponse {
    match action {
        DebugAction::DrawCard => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
            if let Some(deck_card) = battle
                .cards
                .iter()
                .find(|c| matches!(c.position.position, Position::InDeck(DisplayPlayer::User)))
            {
                let card_id = deck_card.id;
                let sorting_key = deck_card.position.sorting_key;
                if let Some(card_index) = battle.cards.iter().position(|c| c.id == card_id) {
                    let mut shown_drawn = battle.clone();
                    shown_drawn.cards[card_index] = card_view(Position::Drawn, sorting_key);
                    battle.cards[card_index] =
                        card_view(Position::InHand(DisplayPlayer::User), sorting_key);

                    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());

                    // Return both updates in sequence
                    return PerformActionResponse {
                        metadata,
                        commands: CommandSequence::from_sequence(vec![
                            Command::UpdateBattle(shown_drawn),
                            Command::UpdateBattle(battle),
                        ]),
                    };
                }
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            PerformActionResponse {
                metadata,
                commands: CommandSequence::from_command(Command::UpdateBattle(battle)),
            }
        }
    }
}

fn perform_battle_action(action: BattleAction, metadata: Metadata) -> PerformActionResponse {
    match action {
        BattleAction::PlayCard(card_id) => {
            let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
            if let Some((card_index, card)) =
                battle.cards.iter().enumerate().find(|(_, c)| c.id == card_id)
            {
                let sorting_key = card.position.sorting_key;
                battle.cards[card_index] =
                    card_view(Position::OnBattlefield(DisplayPlayer::User), sorting_key);
            }

            *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
            PerformActionResponse {
                metadata,
                commands: CommandSequence::from_command(Command::UpdateBattle(battle)),
            }
        }
    }
}

fn scene_0(id: BattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView { score: Points(0), can_act: false },
        enemy: PlayerView { score: Points(0), can_act: false },
        cards: [
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 0, 5),
            cards_in_position(Position::InHand(DisplayPlayer::User), 5, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 8, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::User), 14, 20),
            cards_in_position(Position::OnBattlefield(DisplayPlayer::Enemy), 100, 8),
            cards_in_position(Position::InHand(DisplayPlayer::Enemy), 105, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::Enemy), 108, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::Enemy), 114, 20),
        ]
        .concat()
        .to_vec(),
        status_description: "Status".to_string(),
        controls: vec![],
    }
}

fn cards_in_position(position: Position, start_key: u32, count: u32) -> Vec<CardView> {
    (0..count).map(|i| card_view(position, start_key + i)).collect()
}

fn card_view(position: Position, sorting_key: u32) -> CardView {
    if sorting_key % 5 == 0 {
        card1(position, sorting_key)
    } else if sorting_key % 5 == 1 {
        card2(position, sorting_key)
    } else if sorting_key % 5 == 2 {
        card3(position, sorting_key)
    } else if sorting_key % 5 == 3 {
        card4(position, sorting_key)
    } else {
        card5(position, sorting_key)
    }
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
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("/assets/2521694543.jpg".to_string()),
                image_offset_x: Some(25),
                image_offset_y: Some(50)
            },
            name: "Titan of Forgotten Echoes".to_string(),
            rules_text: "When you materialize your second character in a turn, return this character from your void to play.".to_string(),
            status: None,
            cost: Energy(6),
            spark: Some(Spark(4)),
            card_type: "Ancient".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: flex_node("<b>Materialize</b>: A character entering play."),
            is_fast: false,
            can_play: position == Position::InHand(DisplayPlayer::User),
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card2(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("/assets/1633431262.jpg".to_string()),
                image_offset_x: None,
                image_offset_y: None,
            },
            name: "Beacon of Tomorrow".to_string(),
            rules_text: "Discover a card with cost (2).".to_string(),
            status: None,
            can_play: position == Position::InHand(DisplayPlayer::User),
            cost: Energy(2),
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: flex_node(
                "<b>Discover</b>: Pick one of 4 cards with different types to put into your hand.",
            ),
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
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
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("/assets/2269064817.jpg".to_string()),
                image_offset_x: None,
                image_offset_y: None,
            },
            name: "Scrap Reclaimer".to_string(),
            rules_text: "Judgment: Return this character from your void to your hand. Born from rust and resilience.".to_string(),
            status: None,
            can_play: position == Position::InHand(DisplayPlayer::User),
            cost: Energy(4),
            spark: Some(Spark(0)),
            card_type: "Tinkerer".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: flex_node(
                "<b>Judgment</b>: Triggers at the start of your turn."),
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card4(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("/assets/2269064809.jpg".to_string()),
                image_offset_x: None,
                image_offset_y: None,
            },
            name: "Evacuation Enforcer".to_string(),
            rules_text: "> Draw 2 cards. Discard 3 cards.\nPromises under a stormy sky."
                .to_string(),
            status: None,
            can_play: position == Position::InHand(DisplayPlayer::User),
            cost: Energy(2),
            spark: Some(Spark(0)),
            card_type: "Trooper".to_string(),
            frame: CardFrame::Character,
            supplemental_card_info: None,
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card5(position: Position, sorting_key: u32) -> CardView {
    let revealed = !matches!(position, Position::InDeck(_));
    CardView {
        id: CardId::from_int(sorting_key as u64),
        position: ObjectPosition { position, sorting_key, sorting_sub_key: 0 },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("/assets/2027158310.jpg".to_string()),
                image_offset_x: None,
                image_offset_y: None,
            },
            name: "Moonlit Voyage".to_string(),
            rules_text: "Draw 2 cards. Discard 2 cards.\nReclaim".to_string(),
            status: None,
            can_play: position == Position::InHand(DisplayPlayer::User),
            cost: Energy(2),
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            supplemental_card_info: flex_node(
                "<b>Reclaim</b>: You may play this card from your void, then banish it.",
            ),
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn flex_node(text: impl Into<String>) -> Option<FlexNode> {
    let style = FlexStyle {
        background_color: Some(FlexColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.95 }),
        border_radius: Some(BorderRadius {
            top_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            top_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_right: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
            bottom_left: Dimension { unit: DimensionUnit::Pixels, value: 4.0 },
        }),
        padding: Some(DimensionGroup {
            top: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            right: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            bottom: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
            left: Dimension { unit: DimensionUnit::Pixels, value: 8.0 },
        }),
        color: Some(FlexColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 }),
        font_size: Some(Dimension { unit: DimensionUnit::Pixels, value: 7.0 }),
        white_space: Some(WhiteSpace::Normal),
        ..Default::default()
    };
    Some(FlexNode {
        node_type: Some(NodeType::Text(Text { label: text.into() })),
        style: Some(style),
        ..Default::default()
    })
}
