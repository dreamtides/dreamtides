use std::sync::{LazyLock, Mutex};

use core_data::numerics::Energy;
use display_data::object_position::Position;

/// Display state that should not be serialized as part of the battle state.
#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    /// The source position for the currently active card browser, if any.
    pub card_browser_source: Option<Position>,
    /// The currently selected amount of energy to pay as an additional cost.
    pub selected_energy_additional_cost: Option<Energy>,
}

static DISPLAY_STATE: LazyLock<Mutex<DisplayState>> =
    LazyLock::new(|| Mutex::new(DisplayState::default()));

/// Gets a copy of the current display state.
pub fn get() -> DisplayState {
    DISPLAY_STATE.lock().unwrap().clone()
}

/// Updates the card browser source in the display state.
pub fn set_card_browser_source(source: Option<Position>) {
    DISPLAY_STATE.lock().unwrap().card_browser_source = source;
}

/// Gets the current card browser source.
pub fn get_card_browser_source() -> Option<Position> {
    DISPLAY_STATE.lock().unwrap().card_browser_source
}

/// Updates the selected energy additional cost in the display state.
pub fn set_selected_energy_additional_cost(energy: Option<Energy>) {
    DISPLAY_STATE.lock().unwrap().selected_energy_additional_cost = energy;
}

/// Gets the currently selected energy additional cost.
pub fn get_selected_energy_additional_cost() -> Option<Energy> {
    DISPLAY_STATE.lock().unwrap().selected_energy_additional_cost
}

/// Clears the display state, resetting it to default values.
pub fn clear() {
    *DISPLAY_STATE.lock().unwrap() = DisplayState::default();
}

/// Clears the selected energy additional cost.
pub fn clear_selected_energy_additional_cost() {
    DISPLAY_STATE.lock().unwrap().selected_energy_additional_cost = None;
}
