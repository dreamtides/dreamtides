use action_data::battle_display_action::BattleDisplayAction;
use action_data::game_action_data::GameAction;
use asset_paths::poneti_ui;
use core_data::display_color;
use masonry::flex_enums::{FlexAlign, FlexJustify};
use masonry::flex_style::{FlexScale, FlexStyle};

use crate::box_component::BoxComponent;
use crate::component::Component;
use crate::text_component::TextComponent;
use crate::typography::Typography;

#[derive(Clone)]
pub struct CloseButtonComponent;

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
                        .height(18)
                        .image_slice(100)
                        .justify_content(FlexJustify::Center)
                        .width(18)
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
                .on_click(GameAction::BattleDisplayAction(BattleDisplayAction::CloseCurrentPanel))
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
