use std::env;

use core_data::numerics::Points;
use core_data::types::{CardFacing, Url};
use display_data::battle_view::{BattleView, ClientBattleId, DisplayPlayer, PlayerView};
use display_data::card_view::{CardView, ClientCardId, RevealedCardView};
use display_data::object_position::{ObjectPosition, Position};
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};
use uuid::Uuid;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<_> = env::args().collect();
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![fetch_battle,]);

    if args.len() > 1 && args[1] == "--generate-bindings" {
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
        cards: vec![
            user_hand_card(0),
            user_hand_card(1),
            user_hand_card(2),
            user_hand_card(3),
            user_battlefield_card(0),
            user_battlefield_card(1),
            enemy_battlefield_card(0),
            enemy_battlefield_card(1),
            enemy_hand_card(0),
            enemy_hand_card(1),
            enemy_hand_card(2),
            enemy_hand_card(3),
        ],
        status_description: "Status".to_string(),
        controls: vec![],
    }
}

fn user_hand_card(sorting_key: u32) -> CardView {
    card(Position::InHand(DisplayPlayer::User), sorting_key)
}

fn user_battlefield_card(sorting_key: u32) -> CardView {
    card(Position::OnBattlefield(DisplayPlayer::User), sorting_key)
}

fn enemy_hand_card(sorting_key: u32) -> CardView {
    card(Position::InHand(DisplayPlayer::Enemy), sorting_key)
}

fn enemy_battlefield_card(sorting_key: u32) -> CardView {
    card(Position::OnBattlefield(DisplayPlayer::Enemy), sorting_key)
}

fn card(position: Position, sorting_key: u32) -> CardView {
    CardView {
        id: ClientCardId::CardId(format!("{}", Uuid::new_v4())),
        position: ObjectPosition {
            position,
            sorting_key,
            sorting_sub_key: 0,
        },
        card_back: Url("".to_string()),
        revealed: Some(RevealedCardView {
            image: Url("".to_string()),
            name: "Card".to_string(),
            rules_text: "Rules".to_string(),
            status: None,
            is_ability: false,
            is_token: false,
            can_drag: true,
        }),
        revealed_to_opponents: true,
        card_facing: CardFacing::FaceUp,
        create_position: None,
        destroy_position: None,
    }
}
