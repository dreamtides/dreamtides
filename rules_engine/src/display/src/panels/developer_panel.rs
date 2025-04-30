use action_data::debug_action::DebugAction;
use action_data::game_action::GameAction;
use bon::Builder;
use masonry::flex_enums::{FlexAlign, FlexDirection, FlexJustify, FlexWrap};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
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
                    BoxComponent::builder()
                        .name("Developer Buttons")
                        .style(
                            FlexStyle::builder()
                                .align_items(FlexAlign::Center)
                                .flex_direction(FlexDirection::Row)
                                .flex_grow(1)
                                .justify_content(FlexJustify::SpaceBetween)
                                .wrap(FlexWrap::Wrap)
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Draw Card")
                                .action(DebugAction::DrawCard)
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Restart Battle")
                                .action(DebugAction::RestartBattle)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}

#[derive(Clone, Builder)]
pub struct DebugButton {
    #[builder(into)]
    pub label: String,
    #[builder(into)]
    pub action: GameAction,
}

impl Component for DebugButton {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(format!("{} Button", self.label))
                .style(FlexStyle::builder().margin(4).build())
                .child(ButtonComponent::builder().label(self.label).action(self.action).build())
                .build(),
        )
    }
}
