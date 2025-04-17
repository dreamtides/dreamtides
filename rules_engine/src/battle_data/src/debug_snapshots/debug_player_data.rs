use crate::battle_player::player_data::PlayerData;

pub struct DebugPlayerData {
    pub name: String,
    pub points: String,
    pub current_energy: String,
    pub produced_energy: String,
    pub spark_bonus: String,
}

impl DebugPlayerData {
    pub fn new(player_data: PlayerData) -> Self {
        DebugPlayerData {
            name: format!("{:?}", player_data.name),
            points: player_data.points.to_string(),
            current_energy: player_data.current_energy.to_string(),
            produced_energy: player_data.produced_energy.to_string(),
            spark_bonus: player_data.spark_bonus.to_string(),
        }
    }
}
