use bon::Builder;
use core_data::display_color::{self, DisplayColor};
use core_data::display_types::FontAddress;
use masonry::dimension::Dimension;
use masonry::flex_enums::FontStyle;
use masonry::flex_style::{FlexStyle, TextShadow};

/// A named configuration of typography values, such as font size and color.
#[derive(Clone)]
pub enum Typography {
    StackTrace,
    ButtonLabel,
    InterfaceMessage,
    SupplementalCardInfo,
    Body2,
}

/// Mutates a [FlexStyle] to apply typography options to it.
pub fn apply(typography: &Typography, style: &mut FlexStyle) {
    let options = match typography {
        Typography::StackTrace => {
            TypographyOptions::builder().color(display_color::WHITE).font_size(6).build()
        }
        Typography::ButtonLabel => {
            TypographyOptions::builder().color(display_color::WHITE).font_size(8).build()
        }
        Typography::InterfaceMessage => {
            TypographyOptions::builder().color(display_color::WHITE).font_size(10).build()
        }
        Typography::SupplementalCardInfo => {
            TypographyOptions::builder().color(display_color::WHITE).font_size(10).build()
        }
        Typography::Body2 => {
            TypographyOptions::builder().color(display_color::WHITE).font_size(10).build()
        }
    };

    options.apply_to_style(style);
}

#[derive(Builder)]
struct TypographyOptions {
    pub color: DisplayColor,
    #[builder(into)]
    pub font_size: Dimension,
    #[builder(into)]
    pub letter_spacing: Option<Dimension>,
    pub text_shadow: Option<TextShadow>,
    pub font: Option<FontAddress>,
    pub font_style: Option<FontStyle>,
    pub text_outline_color: Option<DisplayColor>,
    #[builder(into)]
    pub text_outline_width: Option<f32>,
    #[builder(into)]
    pub word_spacing: Option<Dimension>,
}

impl TypographyOptions {
    pub fn apply_to_style(self, style: &mut FlexStyle) {
        style.color = Some(self.color);
        style.font_size = Some(self.font_size);
        style.letter_spacing = self.letter_spacing;
        style.text_shadow = self.text_shadow;
        style.font = self.font;
        style.font_style = self.font_style;
        style.text_outline_color = self.text_outline_color;
        style.text_outline_width = self.text_outline_width;
        style.word_spacing = self.word_spacing;
    }
}
