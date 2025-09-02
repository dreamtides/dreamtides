use serde::Serialize;

use crate::debug::debug_all_cards::DebugAllCards;
use crate::debug::debug_battle_player_state::DebugBattlePlayerState;
use crate::debug::debug_dreamwell::DebugDreamwellState;
use crate::debug::debug_prompt_data::DebugPromptData;

#[derive(Debug, Clone, Serialize)]
pub struct DebugBattleState {
    pub id: String,
    pub player_one: DebugBattlePlayerState,
    pub player_two: DebugBattlePlayerState,
    pub cards: DebugAllCards,
    pub status: String,
    pub stack_priority: String,
    pub turn: String,
    pub phase: String,
    pub dreamwell: DebugDreamwellState,
    pub prompt: DebugPromptData,
}
