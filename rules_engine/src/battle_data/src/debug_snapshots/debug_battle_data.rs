use std::fmt;

use crate::battle::battle_data::BattleData;
use crate::debug_snapshots::debug_all_cards::DebugAllCards;
use crate::debug_snapshots::debug_player_data::DebugPlayerData;
use crate::debug_snapshots::debug_prompt_data::DebugPromptData;

pub struct DebugBattleData {
    pub id: String,
    pub request_context: String,
    pub user: DebugPlayerData,
    pub enemy: DebugPlayerData,
    pub cards: DebugAllCards,
    pub status: String,
    pub turn: String,
    pub step: String,
    pub prompt: Option<DebugPromptData>,
}

impl DebugBattleData {
    pub fn new(battle_data: BattleData) -> Self {
        Self {
            id: format!("{:?}", battle_data.id),
            request_context: format!("{:?}", battle_data.request_context),
            user: DebugPlayerData::new(battle_data.user),
            enemy: DebugPlayerData::new(battle_data.enemy),
            cards: DebugAllCards::new(battle_data.cards),
            status: format!("{:?}", battle_data.status),
            turn: format!(
                "TurnData {{ active_player: {:?}, turn_id: TurnId({}) }}",
                battle_data.turn.active_player, battle_data.turn.turn_id.0
            ),
            step: format!("{:?}", battle_data.step),
            prompt: battle_data.prompt.map(DebugPromptData::new),
        }
    }
}

impl fmt::Debug for DebugBattleData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Battle {}", self.id)
    }
}
