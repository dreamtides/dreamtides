use std::sync::Arc;

use action_data::panel_address::PanelAddress;
use core_data::identifiers::{BattleId, UserId};
use core_data::numerics::Energy;
use core_data::types::PlayerName;
use display_data::object_position::Position;
use tabula_data::tabula::Tabula;

pub trait DisplayStateProvider: Send + Sync {
    fn get_display_state(&self, user_id: UserId) -> DisplayState;

    fn set_display_state(&self, user_id: UserId, state: DisplayState);

    fn tabula(&self) -> Arc<Tabula>;

    fn can_undo(&self, battle_id: BattleId, player: PlayerName) -> bool;
}

#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    pub card_browser_source: Option<Position>,
    pub selected_energy_additional_cost: Option<Energy>,
    pub current_panel_address: Option<PanelAddress>,
    pub overlay_hidden: bool,
}
