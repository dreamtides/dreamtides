use asset_paths::poneti_ui;
use bon::Builder;
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
                        .align_items(FlexAlign::Center)
                        .background_image(poneti_ui::window_background())
                        .image_slice(500)
                        .inset(FlexInsets::builder().top(12).bottom(12).left(8).right(8).build())
                        .justify_content(FlexJustify::Center)
                        .padding(4)
                        .position(FlexPosition::Absolute)
                        .build(),
                )
                .child(self.content)
                .build(),
        )
    }
}
