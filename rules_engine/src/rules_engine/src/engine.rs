use std::sync::{LazyLock, Mutex};

use action_data::user_action::UserAction;
use actions::battle_actions;
use battle_data::battle::battle_data::BattleData;
use core_data::identifiers::BattleId;
use core_data::types::PlayerName;
use display::rendering::renderer;
use display_data::request_data::{
    ConnectRequest, ConnectResponse, PerformActionRequest, PerformActionResponse,
};
use game_creation::new_battle;
use uuid::Uuid;

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleData>>> = LazyLock::new(|| Mutex::new(None));

pub fn connect(request: &ConnectRequest) -> ConnectResponse {
    let id = BattleId(Uuid::new_v4());
    let battle = new_battle::create_and_start(id);
    let commands = renderer::connect(&battle);
    *CURRENT_BATTLE.lock().unwrap() = Some(battle);
    ConnectResponse { metadata: request.metadata, commands }
}

pub fn perform_action(request: &PerformActionRequest) -> PerformActionResponse {
    let mut battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    match request.action {
        UserAction::BattleAction(action) => {
            battle_actions::execute(&mut battle, PlayerName::User, action)
        }
        _ => todo!("Implement other actions"),
    }
    let commands = renderer::render_updates(&battle);
    *CURRENT_BATTLE.lock().unwrap() = Some(battle);
    PerformActionResponse { metadata: request.metadata, commands }
}
