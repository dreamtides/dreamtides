use asset_paths::ui_assets;
use bon::Builder;
use masonry::dimension::{FlexInsets, SafeAreaInsets};
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition};
use masonry::flex_style::FlexStyle;

use crate::box_component::BoxComponent;
use crate::close_button_component::CloseButtonComponent;
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
                        .align_items(FlexAlign::Stretch)
                        .background_image(ui_assets::window_background())
                        // These slice offsets rely on large "pixels per unit"
                        // values in Unity, e.g. 2048 pixels per unit in this
                        // case. I don't really know why.
                        .image_slice(500)
                        .inset(
                            SafeAreaInsets::builder().top(12).bottom(12).left(8).right(8).build(),
                        )
                        .justify_content(FlexJustify::Center)
                        .padding((32, 12, 12, 12))
                        .position(FlexPosition::Absolute)
                        .build(),
                )
                .child(
                    BoxComponent::builder()
                        .name("Close Button Container")
                        .style(
                            FlexStyle::builder()
                                .position(FlexPosition::Absolute)
                                .inset(FlexInsets::builder().top(10).right(10).build())
                                .build(),
                        )
                        .child(CloseButtonComponent)
                        .build(),
                )
                .child(self.content)
                .build(),
        )
    }
}
