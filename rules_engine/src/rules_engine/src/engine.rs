use std::sync::{LazyLock, Mutex};

use battle_data::battle::battle_data::BattleData;
use core_data::identifiers::BattleId;
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

pub fn perform_action(_request: &PerformActionRequest) -> PerformActionResponse {
    let _battle = CURRENT_BATTLE.lock().unwrap().clone().unwrap();
    todo!("")
}
