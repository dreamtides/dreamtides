use battle_data_old::battle::old_battle_data::BattleData;
use core_data::identifiers::QuestId;

/// Contains data types for the "quest" gameplay, which contains all card
/// drafting and deck building mechanics.
#[derive(Clone, Debug)]
pub struct QuestData {
    pub id: QuestId,
    pub current_battle: Option<BattleData>,
}
