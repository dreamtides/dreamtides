use bon::Builder;
use core_data::display_color;
use masonry::dimension::FlexInsets;
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition};
use masonry::flex_style::FlexStyle;

use crate::box_component::BoxComponent;
use crate::component::Component;

#[derive(Clone, Builder)]
pub struct PanelComponent<T: Component> {
    #[builder(into)]
    pub title: String,
    pub content: T,
    #[builder(default)]
    pub show_close_button: bool,
}

impl<T: Component> Component for PanelComponent<T> {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(self.title)
                .style(
                    FlexStyle::builder()
                        .position(FlexPosition::Absolute)
                        .inset(FlexInsets::builder().top(12).bottom(12).left(8).right(8).build())
                        .padding(4)
                        .align_items(FlexAlign::Center)
                        .justify_content(FlexJustify::Center)
                        .background_color(display_color::BLACK_ALPHA_95)
                        .build(),
                )
                .child(self.content)
                .build(),
        )
    }
}
