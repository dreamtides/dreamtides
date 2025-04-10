pub mod basic_scene;

use std::sync::{LazyLock, Mutex};

use core_data::identifiers::BattleId;
use display_data::battle_view::BattleView;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use display_data::request_data::{
    ConnectRequest, ConnectResponse, PerformActionRequest, PerformActionResponse,
};
use uuid::Uuid;

static CURRENT_BATTLE: LazyLock<Mutex<Option<BattleView>>> = LazyLock::new(|| Mutex::new(None));

pub fn connect(request: &ConnectRequest, _scenario: &str) -> ConnectResponse {
    let battle = basic_scene::create(BattleId(Uuid::new_v4()));
    *CURRENT_BATTLE.lock().unwrap() = Some(battle.clone());
    ConnectResponse {
        metadata: request.metadata,
        commands: CommandSequence::from_command(Command::UpdateBattle(UpdateBattleCommand::new(
            battle,
        ))),
    }
}

pub fn perform_action(request: &PerformActionRequest, _scenario: &str) -> PerformActionResponse {
    PerformActionResponse {
        metadata: request.metadata,
        commands: CommandSequence::sequential(vec![]),
    }
}
