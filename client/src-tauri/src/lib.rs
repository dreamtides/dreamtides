use std::env;

use core_data::numerics::{Energy, Points, Spark};
use core_data::types::{CardFacing, Url};
use display_data::battle_view::{BattleView, ClientBattleId, DisplayPlayer, PlayerView};
use display_data::card_view::{CardFrame, CardView, ClientCardId, DisplayImage, RevealedCardView};
use display_data::object_position::{ObjectPosition, Position};
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<_> = env::args().collect();
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![fetch_battle,]);

    if args.len() > 1 && args[1] == "--generate-bindings" {
        eprintln!(">>> Generating bindings");
        #[cfg(debug_assertions)]
        specta_builder
            .export(Typescript::default(), "../src/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .invoke_handler(tauri::generate_handler![fetch_battle])
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
#[specta::specta]
fn fetch_battle(id: ClientBattleId, scene: u32) -> BattleView {
    match scene {
        0 => scene_0(id),
        1 => scene_1(id),
        _ => panic!("Invalid scene number"),
    }
}

fn scene_1(id: ClientBattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView {
            score: Points(0),
            can_act: false,
        },
        enemy: PlayerView {
            score: Points(0),
            can_act: false,
        },
        cards: [
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 0, 5),
            cards_in_position(Position::InHand(DisplayPlayer::User), 5, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 8, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::User), 14, 25),
            vec![card(Position::InHand(DisplayPlayer::User), 39)],
        ]
        .concat()
        .to_vec(),
        status_description: "Status".to_string(),
        controls: vec![],
    }
}

fn scene_0(id: ClientBattleId) -> BattleView {
    BattleView {
        id,
        user: PlayerView {
            score: Points(0),
            can_act: false,
        },
        enemy: PlayerView {
            score: Points(0),
            can_act: false,
        },
        cards: [
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 0, 5),
            cards_in_position(Position::InHand(DisplayPlayer::User), 5, 3),
            cards_in_position(Position::InVoid(DisplayPlayer::User), 8, 6),
            cards_in_position(Position::InDeck(DisplayPlayer::User), 14, 26),
        ]
        .concat()
        .to_vec(),
        status_description: "Status".to_string(),
        controls: vec![],
    }
}

fn cards_in_position(position: Position, start_key: u32, count: u32) -> Vec<CardView> {
    (0..count)
        .map(|i| card(position.clone(), start_key + i))
        .collect()
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
        id: ClientCardId::CardId(format!("#{}", sorting_key)),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("https://www.shutterstock.com/shutterstock/photos/2521694543/display_1500/stock-photo-traveller-in-a-land-of-ancient-statue-digital-art-style-illustration-painting-2521694543.jpg".to_string()),
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
        id: ClientCardId::CardId(format!("#{}", sorting_key)),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("https://www.shutterstock.com/shutterstock/photos/1633431262/display_1500/stock-photo-two-brothers-walking-on-floating-mountain-and-looking-at-a-little-star-in-the-beautiful-sky-1633431262.jpg".to_string()),
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
        id: ClientCardId::CardId(format!("#{}", sorting_key)),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("https://www.shutterstock.com/shutterstock/photos/2269064817/display_1500/stock-photo-futuristic-man-standing-on-a-large-pile-of-scrap-metal-pieces-digital-art-style-illustration-2269064817.jpg".to_string()),
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
        id: ClientCardId::CardId(format!("#{}", sorting_key)),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("https://www.shutterstock.com/shutterstock/photos/2269064809/display_1500/stock-photo-soldier-waiting-for-a-woman-bidding-farewell-to-a-child-digital-art-style-illustration-painting-2269064809.jpg".to_string()),
                image_offset_x: None,
                image_offset_y: None,
            },
            name: "Evacuation Enforcer".to_string(),
            rules_text: "> Draw 2 cards. Discard 3 cards.\nPromises under a stormy sky.".to_string(),
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
        id: ClientCardId::CardId(format!("#{}", sorting_key)),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: revealed.then_some(RevealedCardView {
            image: DisplayImage {
                image: Url("https://www.shutterstock.com/shutterstock/photos/2027158310/display_1500/stock-photo-young-man-rowing-a-boat-in-the-sea-looking-at-the-crescent-digital-art-style-illustration-painting-2027158310.jpg".to_string()),
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
