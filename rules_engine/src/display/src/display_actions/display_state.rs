use std::sync::{LazyLock, Mutex};

use action_data::panel_address::PanelAddress;
use core_data::numerics::Energy;
use display_data::object_position::Position;

/// Display state that should not be serialized as part of the battle state.
#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    /// The source position for the currently active card browser, if any.
    pub card_browser_source: Option<Position>,
    /// The currently selected amount of energy to pay as an additional cost.
    pub selected_energy_additional_cost: Option<Energy>,
    /// The currently open panel, if any.
    pub current_panel_address: Option<PanelAddress>,
    /// Whether the stack is currently hidden.
    pub stack_hidden: bool,
}

static DISPLAY_STATE: LazyLock<Mutex<DisplayState>> =
    LazyLock::new(|| Mutex::new(DisplayState::default()));

/// Updates the card browser source in the display state.
pub fn set_card_browser_source(source: Option<Position>) {
    DISPLAY_STATE.lock().unwrap().card_browser_source = source;
}

/// Gets the current card browser source.
pub fn get_card_browser_source() -> Option<Position> {
    DISPLAY_STATE.lock().unwrap().card_browser_source.clone()
}

/// Updates the selected energy additional cost in the display state.
pub fn set_selected_energy_additional_cost(energy: Option<Energy>) {
    DISPLAY_STATE.lock().unwrap().selected_energy_additional_cost = energy;
}

/// Gets the currently selected energy additional cost.
pub fn get_selected_energy_additional_cost() -> Option<Energy> {
    DISPLAY_STATE.lock().unwrap().selected_energy_additional_cost
}

/// Clears the selected energy additional cost.
pub fn clear_selected_energy_additional_cost() {
    DISPLAY_STATE.lock().unwrap().selected_energy_additional_cost = None;
}

/// Updates the current panel address in the display state.
pub fn set_current_panel_address(address: Option<PanelAddress>) {
    DISPLAY_STATE.lock().unwrap().current_panel_address = address;
}

/// Gets the current panel address.
pub fn get_current_panel_address() -> Option<PanelAddress> {
    DISPLAY_STATE.lock().unwrap().current_panel_address.clone()
}

/// Updates whether the stack is hidden.
pub fn set_stack_hidden(hidden: bool) {
    DISPLAY_STATE.lock().unwrap().stack_hidden = hidden;
}

/// Gets whether the stack is currently hidden.
pub fn is_stack_hidden() -> bool {
    DISPLAY_STATE.lock().unwrap().stack_hidden
}
