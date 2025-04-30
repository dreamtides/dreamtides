use bon::Builder;
use masonry::dimension::{Dimension, DimensionGroup, FlexInsets};
use masonry::flex_enums::{FlexAlign, FlexDisplayStyle, FlexPosition, FlexVisibility};
use masonry::flex_style::{FlexStyle, FlexTranslate, Opacity};

/// A subset of the full style API, used in higher-level components that do not
/// require arbitrary styling.
#[derive(Clone, Builder)]
pub struct StyleOptions {
    pub align_self: Option<FlexAlign>,
    pub display: Option<FlexDisplayStyle>,
    pub inset: Option<FlexInsets>,
    #[builder(into)]
    pub margin: Option<DimensionGroup>,
    #[builder(into)]
    pub max_height: Option<Dimension>,
    #[builder(into)]
    pub max_width: Option<Dimension>,
    #[builder(into)]
    pub opacity: Option<Opacity>,
    pub position: Option<FlexPosition>,
    pub translate: Option<FlexTranslate>,
    pub visibility: Option<FlexVisibility>,
}

/// Mutates a [FlexStyle] to apply style options to it.
pub fn apply(options: &Option<StyleOptions>, style: &mut FlexStyle) {
    if let Some(options) = options {
        style.align_self = options.align_self;
        style.display = options.display;
        style.inset = options.inset;
        style.margin = options.margin;
        style.max_height = options.max_height;
        style.max_width = options.max_width;
        style.opacity = options.opacity;
        style.position = options.position;
        style.translate = options.translate;
        style.visibility = options.visibility;
    }
}
