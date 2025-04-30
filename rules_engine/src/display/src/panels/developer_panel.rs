use core_data::display_color;
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
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
                        .name("Box")
                        .style(
                            FlexStyle::builder()
                                .width(100)
                                .height(100)
                                .background_color(display_color::RED_500)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}
