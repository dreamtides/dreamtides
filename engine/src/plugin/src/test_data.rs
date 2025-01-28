use core_data::numerics::{Energy, Points, Spark};
use core_data::types::{CardFacing, Url};
use display_data::battle_view::{BattleView, ClientBattleId, DisplayPlayer, PlayerView};
use display_data::card_view::{CardFrame, CardView, ClientCardId, DisplayImage, RevealedCardView};
use display_data::command::{Command, CommandSequence};
use display_data::object_position::{ObjectPosition, Position};

pub fn get_scene(id: ClientBattleId, scene: u32) -> CommandSequence {
    CommandSequence::from_command(Command::UpdateBattle(match scene {
        0 => scene_0(id),
        n if n <= 15 => draw_n_cards(scene_0(id), 14, n),
        _ => panic!("Invalid scene number"),
    }))
}

fn draw_n_cards(mut view: BattleView, start_key: u32, count: u32) -> BattleView {
    for i in 0..count {
        view = draw_card(view, start_key + i);
    }
    view
}

fn draw_card(mut view: BattleView, sorting_key: u32) -> BattleView {
    if let Some(found) = view.cards.iter_mut().find(|card| {
        matches!(card.position.position, Position::InDeck(DisplayPlayer::User))
            && card.position.sorting_key == sorting_key
    }) {
        let new_position = Position::InHand(DisplayPlayer::User);
        *found = card(new_position, sorting_key);
    }
    view
}

fn scene_0(id: ClientBattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView { score: Points(0), can_act: false },
        enemy: PlayerView { score: Points(0), can_act: false },
        cards: [
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 0, 5),
            cards_in_position(Position::InHand(DisplayPlayer::User), 5, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 8, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::User), 14, 20),
        ]
        .concat()
        .to_vec(),
        status_description: "Status".to_string(),
        controls: vec![],
    }
}

fn cards_in_position(position: Position, start_key: u32, count: u32) -> Vec<CardView> {
    (0..count).map(|i| card(position.clone(), start_key + i)).collect()
}

fn card(position: Position, sorting_key: u32) -> CardView {
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
    let revealed = position != Position::InDeck(DisplayPlayer::User);
    CardView {
        id: ClientCardId(format!("#{}", sorting_key)),
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
            can_drag: true,
            cost: Energy(6),
            spark: Some(Spark(4)),
            card_type: "Ancient".to_string(),
            frame: CardFrame::Character,
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card2(position: Position, sorting_key: u32) -> CardView {
    let revealed = position != Position::InDeck(DisplayPlayer::User);
    CardView {
        id: ClientCardId(format!("#{}", sorting_key)),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("/assets/1633431262.jpg".to_string()),
                image_offset_x: None,
                image_offset_y: None,
            },
            name: "Beacon of Tomorrow".to_string(),
            rules_text: "Discover a card with cost (2). (pick one of 4 cards with different types to put into your hand.)".to_string(),
            status: None,
            can_drag: true,
            cost: Energy(2),
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card3(position: Position, sorting_key: u32) -> CardView {
    let revealed = position != Position::InDeck(DisplayPlayer::User);
    CardView {
        id: ClientCardId(format!("#{}", sorting_key)),
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
            can_drag: true,
            cost: Energy(4),
            spark: Some(Spark(0)),
            card_type: "Tinkerer".to_string(),
            frame: CardFrame::Character,
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card4(position: Position, sorting_key: u32) -> CardView {
    let revealed = position != Position::InDeck(DisplayPlayer::User);
    CardView {
        id: ClientCardId(format!("#{}", sorting_key)),
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
            can_drag: true,
            cost: Energy(2),
            spark: Some(Spark(0)),
            card_type: "Trooper".to_string(),
            frame: CardFrame::Character,
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}

fn card5(position: Position, sorting_key: u32) -> CardView {
    let revealed = position != Position::InDeck(DisplayPlayer::User);
    CardView {
        id: ClientCardId(format!("#{}", sorting_key)),
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
            can_drag: true,
            cost: Energy(2),
            spark: None,
            card_type: "Event".to_string(),
            frame: CardFrame::Event,
            is_fast: false,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}
