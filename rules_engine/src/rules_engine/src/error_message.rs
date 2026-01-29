use display_data::command::{Command, CommandSequence, UpdateScreenOverlayCommand};
use masonry::flex_enums::{TextAlign, WhiteSpace};
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

/// Attempts to display an error message to the player describing a rules engine
/// error.
#[expect(clippy::print_stderr)]
pub fn display_error_message(message: impl Into<String>) -> CommandSequence {
    let message = message.into();
    eprintln!("ERROR: {message}");
    let overlay = render_message(message).flex_node();
    CommandSequence::from_command(Command::UpdateScreenOverlay(Box::new(
        UpdateScreenOverlayCommand { screen_overlay: overlay },
    )))
}

fn render_message(text: String) -> impl Component {
    PanelComponent::builder()
        .title("Error".to_string())
        .content(
            TextComponent::builder()
                .text(text)
                .text_align(TextAlign::UpperLeft)
                .typography(Typography::StackTrace)
                .white_space(WhiteSpace::Normal)
                .build(),
        )
        .build()
}
