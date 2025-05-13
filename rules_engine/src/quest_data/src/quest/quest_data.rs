use battle_state::battle::battle_state::BattleState;
use core_data::identifiers::QuestId;

/// Contains data types for the "quest" gameplay, which contains all card
/// drafting and deck building mechanics.
#[derive(Clone, Debug)]
pub struct QuestData {
    pub id: QuestId,
    pub current_battle: Option<BattleState>,
}
