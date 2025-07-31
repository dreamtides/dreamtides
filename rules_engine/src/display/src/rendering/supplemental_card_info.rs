use bon::Builder;
use core_data::display_color;
use masonry::flex_enums::{TextAlign, WhiteSpace};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::component::Component;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

#[derive(Clone, Builder)]
pub struct SupplementalCardInfo {
    #[builder(into)]
    pub text: String,
}

impl Component for SupplementalCardInfo {
    fn render(self) -> Option<impl Component> {
        Some(
            BoxComponent::builder()
                .name("Supplemental Card Info")
                .style(
                    FlexStyle::builder()
                        .background_color(display_color::BLACK_ALPHA_95)
                        .border_radius(2)
                        .padding(4)
                        .margin(2)
                        .build(),
                )
                .child(
                    TextComponent::builder()
                        .text(self.text)
                        .typography(Typography::SupplementalCardInfo)
                        .text_align(TextAlign::MiddleLeft)
                        .white_space(WhiteSpace::Normal)
                        .build(),
                )
                .build(),
        )
    }
}
