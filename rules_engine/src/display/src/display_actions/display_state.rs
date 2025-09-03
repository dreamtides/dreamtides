use action_data::panel_address::PanelAddress;
use core_data::numerics::Energy;
use display_data::object_position::Position;

use crate::core::response_builder::ResponseBuilder;

/// Updates the card browser source in the display state.
pub fn set_card_browser_source(builder: &ResponseBuilder, source: Option<Position>) {
    builder.update_display_state(|state| {
        state.card_browser_source = source;
    });
}

/// Gets the current card browser source.
pub fn get_card_browser_source(builder: &ResponseBuilder) -> Option<Position> {
    builder.get_display_state().card_browser_source
}

/// Updates the selected energy additional cost in the display state.
pub fn set_selected_energy_additional_cost(builder: &ResponseBuilder, energy: Option<Energy>) {
    builder.update_display_state(|state| {
        state.selected_energy_additional_cost = energy;
    });
}

/// Gets the currently selected energy additional cost.
pub fn get_selected_energy_additional_cost(builder: &ResponseBuilder) -> Option<Energy> {
    builder.get_display_state().selected_energy_additional_cost
}

/// Clears the selected energy additional cost.
pub fn clear_selected_energy_additional_cost(builder: &ResponseBuilder) {
    builder.update_display_state(|state| {
        state.selected_energy_additional_cost = None;
    });
}

/// Updates the current panel address in the display state.
pub fn set_current_panel_address(builder: &ResponseBuilder, address: Option<PanelAddress>) {
    builder.update_display_state(|state| {
        state.current_panel_address = address;
    });
}

/// Gets the current panel address.
pub fn get_current_panel_address(builder: &ResponseBuilder) -> Option<PanelAddress> {
    builder.get_display_state().current_panel_address
}

/// Updates whether the prominent overlay (stack, browsers, selectors) is
/// hidden.
pub fn set_overlay_hidden(builder: &ResponseBuilder, hidden: bool) {
    builder.update_display_state(|state| {
        state.overlay_hidden = hidden;
    });
}

/// Gets whether the prominent overlay is currently hidden.
pub fn is_overlay_hidden(builder: &ResponseBuilder) -> bool {
    builder.get_display_state().overlay_hidden
}
