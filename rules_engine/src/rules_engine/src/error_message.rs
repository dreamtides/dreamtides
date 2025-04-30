use battle_data::battle::battle_data::BattleData;
use core_data::types::PlayerName;
use display::core::response_builder::ResponseBuilder;
use display::rendering::battle_rendering;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

/// Attempts to display an error message to the player describing a rules engine
/// error.
pub fn display_error_message(battle: &BattleData, message: String) -> CommandSequence {
    let mut builder = ResponseBuilder::new(PlayerName::One, false);
    let mut view = battle_rendering::battle_view(&builder, battle);
    view.interface.screen_overlay = render_message(message).flex_node();
    builder.push(Command::UpdateBattle(UpdateBattleCommand { battle: view, update_sound: None }));
    builder.commands()
}

fn render_message(text: String) -> impl Component {
    PanelComponent::builder()
        .title("Error Message")
        .content(TextComponent::builder().text(text).typography(Typography::StackTrace).build())
        .build()
}
