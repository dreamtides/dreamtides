use action_data::debug_action_data::DebugAction;
use action_data::game_action_data::GameAction;
use ai_data::game_ai::GameAI;
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
                                .flex_direction(FlexDirection::Column)
                                .flex_grow(1)
                                .justify_content(FlexJustify::Center)
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
                        .child(
                            DebugButton::builder()
                                .label("UCT 1000")
                                .action(DebugAction::SetOpponentAgent(GameAI::Uct1MaxIterations(
                                    1000,
                                )))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("UCT 5000")
                                .action(DebugAction::SetOpponentAgent(GameAI::Uct1MaxIterations(
                                    5000,
                                )))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("UCT 10,000")
                                .action(DebugAction::SetOpponentAgent(GameAI::Uct1MaxIterations(
                                    10000,
                                )))
                                .build(),
                        )
                        .child(
                            DebugButton::builder()
                                .label("Iterative Deepening")
                                .action(DebugAction::SetOpponentAgent(GameAI::IterativeDeepening))
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
