use ai_data::game_ai::GameAI;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::actions::debug_battle_action::DebugBattleAction;
use battle_state::battle_player::battle_player_state::TestDeckName;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::game_action_data::GameAction;

/// Private actions for developer use
#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
pub enum DebugAction {
    ApplyTestScenarioAction(String),
    RestartBattle,
    RestartBattleWithDecks { one: TestDeckName, two: TestDeckName },
    SetOpponentAgent(GameAI),
    SetOpponentAsHuman,
    ApplyActionList(Vec<DebugBattleAction>),
    CloseCurrentPanelApplyAction(DebugBattleAction),
    PerformOpponentAction(BattleAction),
}

impl From<DebugAction> for GameAction {
    fn from(action: DebugAction) -> Self {
        GameAction::DebugAction(action)
    }
}
