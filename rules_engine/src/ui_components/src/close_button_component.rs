use action_data::game_action::GameAction;
use asset_paths::poneti_ui;
use bon::Builder;
use masonry::flex_enums::{FlexAlign, FlexJustify};
use masonry::flex_style::FlexStyle;

use crate::box_component::BoxComponent;
use crate::component::Component;
use crate::style_options::StyleOptions;
use crate::text_component::TextComponent;
use crate::typography::Typography;

#[derive(Clone, Builder, Default)]
pub struct CloseButtonComponent {
    pub style_options: Option<StyleOptions>,
}

impl Component for CloseButtonComponent {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name("Close Button")
                .style(
                    FlexStyle::builder()
                        .align_items(FlexAlign::Center)
                        .background_image(poneti_ui::close_button_background())
                        .flex_shrink(0)
                        .height(24)
                        .image_slice(100)
                        .justify_content(FlexJustify::Center)
                        .width(24)
                        .build(),
                )
                .on_click(GameAction::CloseCurrentPanel)
                .child(
                    TextComponent::builder()
                        .text("\u{f00d}")
                        .typography(Typography::ButtonLabel)
                        .build(),
                )
                .build(),
        )
    }
}
