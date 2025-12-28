use bon::Builder;
use core_data::display_color::DisplayColor;
use core_data::display_types::{FontAddress, Milliseconds, SpriteAddress};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::borders::{BorderColor, BorderRadius, BorderWidth};
use crate::dimension::{Dimension, DimensionGroup, FlexInsets};
use crate::flex_enums::{
    EasingMode, FlexAlign, FlexDirection, FlexDisplayStyle, FlexJustify, FlexPosition,
    FlexVisibility, FlexWrap, FontStyle, OverflowClipBox, TextAlign, TextOverflow,
    TextOverflowPosition, WhiteSpace,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FlexVector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FlexVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FlexRotate {
    pub degrees: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, Builder)]
pub struct FlexTranslate {
    #[builder(into)]
    pub x: Dimension,
    #[builder(into)]
    pub y: Dimension,
    #[builder(default)]
    pub z: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FlexScale {
    pub amount: FlexVector3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TextShadow {
    pub offset: FlexVector2,
    pub blur_radius: f32,
    pub color: DisplayColor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, Builder)]
pub struct ImageSlice {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum FlexPickingMode {
    Position,
    Ignore,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct FlexGrow(pub f32);

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct FlexShrink(pub f32);

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Opacity(pub f32);

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum ImageScaleMode {
    StretchToFill,
    ScaleAndCrop,
    ScaleToFit,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, Builder)]
pub struct FlexStyle {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_content: Option<FlexAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_items: Option<FlexAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<FlexAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<DisplayColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<SpriteAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image_tint_color: Option<DisplayColor>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_color: Option<BorderColor>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<BorderRadius>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_width: Option<BorderWidth>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inset: Option<FlexInsets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<DisplayColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<FlexDisplayStyle>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_basis: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<FlexDirection>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<FlexGrow>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_shrink: Option<FlexShrink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrap: Option<FlexWrap>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_slice: Option<ImageSlice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<FlexJustify>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<DimensionGroup>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Opacity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow: Option<FlexVisibility>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<DimensionGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picking_mode: Option<FlexPickingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<FlexPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotate: Option<FlexRotate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<FlexScale>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_overflow: Option<TextOverflow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_shadow: Option<TextShadow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform_origin: Option<FlexTranslate>,
    #[builder(default)]
    pub transition_delays: Vec<Milliseconds>,
    #[builder(default)]
    pub transition_durations: Vec<Milliseconds>,
    #[builder(default)]
    pub transition_properties: Vec<String>,
    #[builder(default)]
    pub transition_easing_modes: Vec<EasingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate: Option<FlexTranslate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<FontStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow_clip_box: Option<OverflowClipBox>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_spacing: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<TextAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_outline_color: Option<DisplayColor>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_outline_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_overflow_position: Option<TextOverflowPosition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<FlexVisibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub white_space: Option<WhiteSpace>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<Dimension>,
    #[builder(into)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub word_spacing: Option<Dimension>,
}

impl FlexVector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }
}

impl FlexScale {
    pub fn new(amount: f32) -> Self {
        Self { amount: FlexVector3::new(amount, amount, amount) }
    }
}

impl From<u32> for ImageSlice {
    fn from(value: u32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }
}

impl From<f32> for FlexGrow {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<i32> for FlexGrow {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}

impl From<f32> for FlexShrink {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<i32> for FlexShrink {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}

impl From<f32> for Opacity {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<i32> for Opacity {
    fn from(value: i32) -> Self {
        Self(value as f32)
    }
}
