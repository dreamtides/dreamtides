use action_data::game_action_data::GameAction;
use asset_paths::ui_assets;
use bon::Builder;
use core_data::display_color;
use masonry::flex_enums::{FlexAlign, FlexJustify};
use masonry::flex_style::{FlexGrow, FlexScale, FlexStyle};

use crate::box_component::BoxComponent;
use crate::component::Component;
use crate::text_component::TextComponent;
use crate::typography::Typography;

#[derive(Clone, Builder)]
pub struct ButtonComponent {
    #[builder(into)]
    pub label: String,
    #[builder(into)]
    pub action: GameAction,
    pub flex_grow: Option<FlexGrow>,
    #[builder(default)]
    pub is_primary: bool,
}

impl Component for ButtonComponent {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name(format!("{} Button", self.label))
                .style(
                    FlexStyle::builder()
                        .align_items(FlexAlign::Center)
                        .background_image(if self.is_primary {
                            ui_assets::primary_button_background()
                        } else {
                            ui_assets::secondary_button_background()
                        })
                        .flex_grow(self.flex_grow.unwrap_or_default())
                        .flex_shrink(0)
                        .height(20)
                        .image_slice(45)
                        .justify_content(FlexJustify::Center)
                        .min_width(20)
                        .padding((0, 6))
                        .build(),
                )
                .hover_style(
                    FlexStyle::builder()
                        .background_image_tint_color(display_color::GRAY_300)
                        .build(),
                )
                .pressed_style(
                    FlexStyle::builder()
                        .background_image_tint_color(display_color::GRAY_500)
                        .scale(FlexScale::new(0.97))
                        .build(),
                )
                .on_click(self.action)
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
