use asset_paths::poneti_ui;
use bon::Builder;
use core_data::display_types::SpriteAddress;
use masonry::flex_enums::{FlexAlign, FlexJustify};
use masonry::flex_style::{FlexGrow, FlexStyle};

use crate::box_component::BoxComponent;
use crate::component::Component;
use crate::style_options::StyleOptions;
use crate::text_component::TextComponent;
use crate::typography::Typography;

#[derive(Clone, Builder)]
pub struct ButtonComponent {
    #[builder(into)]
    pub label: String,
    pub style_options: Option<StyleOptions>,
    pub flex_grow: Option<FlexGrow>,
}

impl Component for ButtonComponent {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(format!("{} Button", self.label))
                .style(
                    FlexStyle::builder()
                        .align_items(FlexAlign::Center)
                        .background_image(poneti_ui::primary_button_background())
                        .flex_grow(self.flex_grow.unwrap_or_default())
                        .flex_shrink(0)
                        .height(32)
                        .justify_content(FlexJustify::Center)
                        .min_width(32)
                        .padding((12, 0))
                        .image_slice(45)
                        .build(),
                )
                .child(
                    TextComponent::builder()
                        .text(self.label)
                        .typography(Typography::ButtonLabel)
                        .build(),
                )
                .build(),
        )
    }
}
