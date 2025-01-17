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
fn fetch_battle(id: ClientBattleId) -> BattleView {
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
            cards_in_position(Position::OnBattlefield(DisplayPlayer::User), 0, 4),
            cards_in_position(Position::InHand(DisplayPlayer::User), 4, 4),
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
