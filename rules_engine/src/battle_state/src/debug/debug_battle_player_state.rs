use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DebugBattlePlayerState {
    pub name: String,
    pub points: String,
    pub current_energy: String,
    pub produced_energy: String,
    pub spark_bonus: String,
}
