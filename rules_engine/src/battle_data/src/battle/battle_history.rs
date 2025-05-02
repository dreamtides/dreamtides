use core_data::types::PlayerName;

#[derive(Clone, Debug)]
pub struct BattleHistory {
    pub actions: Vec<BattleHistoryAction>,
}

#[derive(Clone, Debug)]
pub struct BattleHistoryAction {
    pub player: PlayerName,
}
