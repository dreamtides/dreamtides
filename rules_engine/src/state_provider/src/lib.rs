use action_data::panel_address::PanelAddress;
use core_data::identifiers::UserId;
use core_data::numerics::Energy;
use display_data::object_position::Position;

#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    pub card_browser_source: Option<Position>,
    pub selected_energy_additional_cost: Option<Energy>,
    pub current_panel_address: Option<PanelAddress>,
    pub stack_hidden: bool,
}

pub trait StateProvider: Send + Sync {
    fn get_display_state(&self, user_id: UserId) -> DisplayState;

    fn set_display_state(&self, user_id: UserId, state: DisplayState);
}
