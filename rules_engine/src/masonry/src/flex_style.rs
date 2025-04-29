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
#[serde(rename_all = "camelCase")]
pub struct FlexVector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl FlexVector3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexRotate {
    pub degrees: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexTranslate {
    pub x: Dimension,
    pub y: Dimension,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexScale {
    pub amount: FlexVector3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TextShadow {
    pub offset: FlexVector2,
    pub blur_radius: f32,
    pub color: DisplayColor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ImageSlice {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexPickingMode {
    Position,
    Ignore,
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct FlexGrow(pub f32);

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

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct FlexShrink(pub f32);

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

pub struct Opacity(pub f32);

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

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct FlexStyle {
    pub align_content: Option<FlexAlign>,
    pub align_items: Option<FlexAlign>,
    pub align_self: Option<FlexAlign>,
    pub background_color: Option<DisplayColor>,
    pub background_image: Option<SpriteAddress>,
    pub background_image_tint_color: Option<DisplayColor>,
    #[builder(into)]
    pub border_color: Option<BorderColor>,
    #[builder(into)]
    pub border_radius: Option<BorderRadius>,
    #[builder(into)]
    pub border_width: Option<BorderWidth>,
    #[builder(into)]
    pub inset: Option<FlexInsets>,
    pub color: Option<DisplayColor>,
    pub display: Option<FlexDisplayStyle>,
    #[builder(into)]
    pub flex_basis: Option<Dimension>,
    pub flex_direction: Option<FlexDirection>,
    #[builder(into)]
    pub flex_grow: Option<FlexGrow>,
    #[builder(into)]
    pub flex_shrink: Option<FlexShrink>,
    pub wrap: Option<FlexWrap>,
    #[builder(into)]
    pub font_size: Option<Dimension>,
    #[builder(into)]
    pub height: Option<Dimension>,
    pub image_slice: Option<ImageSlice>,
    pub justify_content: Option<FlexJustify>,
    #[builder(into)]
    pub letter_spacing: Option<Dimension>,
    #[builder(into)]
    pub margin: Option<DimensionGroup>,
    #[builder(into)]
    pub max_height: Option<Dimension>,
    #[builder(into)]
    pub max_width: Option<Dimension>,
    #[builder(into)]
    pub min_height: Option<Dimension>,
    #[builder(into)]
    pub min_width: Option<Dimension>,
    #[builder(into)]
    pub opacity: Option<f32>,
    pub overflow: Option<FlexVisibility>,
    #[builder(into)]
    pub padding: Option<DimensionGroup>,
    pub picking_mode: Option<FlexPickingMode>,
    pub position: Option<FlexPosition>,
    pub rotate: Option<FlexRotate>,
    pub scale: Option<FlexScale>,
    pub text_overflow: Option<TextOverflow>,
    pub text_shadow: Option<TextShadow>,
    pub transform_origin: Option<FlexTranslate>,
    #[builder(default)]
    pub transition_delays: Vec<Milliseconds>,
    #[builder(default)]
    pub transition_durations: Vec<Milliseconds>,
    #[builder(default)]
    pub transition_properties: Vec<String>,
    #[builder(default)]
    pub transition_easing_modes: Vec<EasingMode>,
    pub translate: Option<FlexTranslate>,
    pub font: Option<FontAddress>,
    pub font_style: Option<FontStyle>,
    pub overflow_clip_box: Option<OverflowClipBox>,
    #[builder(into)]
    pub paragraph_spacing: Option<Dimension>,
    pub text_align: Option<TextAlign>,
    pub text_outline_color: Option<DisplayColor>,
    #[builder(into)]
    pub text_outline_width: Option<f32>,
    pub text_overflow_position: Option<TextOverflowPosition>,
    pub visibility: Option<FlexVisibility>,
    pub white_space: Option<WhiteSpace>,
    #[builder(into)]
    pub width: Option<Dimension>,
    #[builder(into)]
    pub word_spacing: Option<Dimension>,
}
