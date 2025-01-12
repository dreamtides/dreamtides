use core_data::numerics::Points;
use display_data::battle_view::{BattleView, ClientBattleId, PlayerView};
use specta_typescript::Typescript;
use tauri_specta::{collect_commands, Builder};

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
        cards: vec![],
        status_description: "Status".to_string(),
        controls: vec![],
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![fetch_battle,]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

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
