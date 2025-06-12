use bon::Builder;
use core_data::display_color;
use core_data::display_types::Milliseconds;
use masonry::dimension::{Percent, SafeAreaInsets};
use masonry::flex_enums::{FlexAlign, FlexJustify, FlexPosition, TextAlign, WhiteSpace};
use masonry::flex_style::FlexStyle;
use ui_components::box_component::BoxComponent;
use ui_components::component::Component;
use ui_components::text_component::TextComponent;
use ui_components::typography::Typography;

#[derive(Clone, Copy, Debug)]
pub enum AnchorPosition {
    Top,
    Bottom,
}

#[derive(Clone, Builder)]
pub struct InterfaceMessage {
    #[builder(into)]
    pub text: String,
    pub anchor_position: AnchorPosition,
    pub temporary: bool,
}

impl Component for InterfaceMessage {
    fn render(self) -> Option<impl Component> {
        let inset = match self.anchor_position {
            AnchorPosition::Top => SafeAreaInsets::builder().top(8).left(8).right(8).build(),
            AnchorPosition::Bottom => SafeAreaInsets::builder().bottom(8).left(8).right(8).build(),
        };

        Some(
            BoxComponent::builder()
                .name("Interface Message Container")
                .style(
                    FlexStyle::builder()
                        .position(FlexPosition::Absolute)
                        .inset(inset)
                        .align_items(FlexAlign::Center)
                        .justify_content(FlexJustify::Center)
                        .opacity(if self.temporary { 0 } else { 1 })
                        .transition_durations(
                            self.temporary
                                .then(|| vec![Milliseconds::new(300)])
                                .unwrap_or_default(),
                        )
                        .transition_properties(
                            self.temporary.then(|| vec!["opacity".to_string()]).unwrap_or_default(),
                        )
                        .build(),
                )
                .maybe_on_attach_style(self.temporary.then(|| {
                    FlexStyle::builder()
                        .opacity(1)
                        .transition_durations(vec![Milliseconds::new(300)])
                        .transition_properties(vec!["opacity".to_string()])
                        .build()
                }))
                .maybe_on_attach_style_duration(self.temporary.then(|| Milliseconds::new(5000)))
                .child(
                    BoxComponent::builder()
                        .name("Interface Message")
                        .style(
                            FlexStyle::builder()
                                .background_color(display_color::BLACK_ALPHA_95)
                                .border_radius(4)
                                .padding(4)
                                .max_width(Percent(80))
                                .align_items(FlexAlign::Center)
                                .justify_content(FlexJustify::Center)
                                .build(),
                        )
                        .child(
                            TextComponent::builder()
                                .text(self.text)
                                .typography(Typography::InterfaceMessage)
                                .text_align(TextAlign::MiddleCenter)
                                .white_space(WhiteSpace::Normal)
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
    }
}
