use battle_data::battle::battle_data::BattleData;
use core_data::display_color::{self};
use core_data::types::PlayerName;
use display::core::response_builder::ResponseBuilder;
use display::rendering::battle_rendering;
use display_data::command::{Command, CommandSequence, UpdateBattleCommand};
use masonry::dimension::FlexInsets;
use masonry::flex_enums::FlexPosition;
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

/// Attempts to display an error message to the player describing a rules engine
/// error.
pub fn display_error_message(battle: &BattleData, message: String) -> CommandSequence {
    let mut builder = ResponseBuilder::new(PlayerName::One, false);
    let mut view = battle_rendering::battle_view(&builder, battle);
    view.interface.screen_overlay = render_message(message).into();
    builder.push(Command::UpdateBattle(UpdateBattleCommand { battle: view, update_sound: None }));
    builder.commands()
}

fn render_message(text: String) -> BoxComponent {
    BoxComponent::builder()
        .style(
            FlexStyle::builder()
                .background_color(display_color::BLACK_ALPHA_95)
                .border_radius(4)
                .inset(FlexInsets::builder().top(12).left(8).right(8).build())
                .min_height(22)
                .padding(4)
                .position(FlexPosition::Absolute)
                .build(),
        )
        .child(TextComponent::builder().text(text).typography(Typography::StackTrace).build())
        .build()
}
