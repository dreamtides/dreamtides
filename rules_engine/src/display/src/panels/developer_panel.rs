use action_data::debug_action::DebugAction;
use ui_components::button_component::ButtonComponent;
use ui_components::component::Component;
use ui_components::panel_component::PanelComponent;

#[derive(Clone)]
pub struct DeveloperPanel;

impl Component for DeveloperPanel {
    fn render(self) -> Option<impl Component> {
        Some(
            PanelComponent::builder()
                .title("Developer")
                .content(
                    ButtonComponent::builder()
                        .label("Draw Card")
                        .action(DebugAction::DrawCard)
                        .build(),
                )
                .build(),
        )
    }
}
