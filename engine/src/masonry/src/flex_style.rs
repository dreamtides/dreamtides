use core_data::display_color::DisplayColor;
use core_data::display_types::{FontAddress, Milliseconds, SpriteAddress};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0, z: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DimensionUnit {
    Pixels,
    Percentage,
    ViewportWidth,
    ViewportHeight,
    SafeAreaTop,
    SafeAreaRight,
    SafeAreaBottom,
    SafeAreaLeft,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Dimension {
    pub unit: DimensionUnit,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DimensionGroup {
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexInsets {
    pub top: Option<Dimension>,
    pub right: Option<Dimension>,
    pub bottom: Option<Dimension>,
    pub left: Option<Dimension>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorderWidth {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorderColor {
    pub top: DisplayColor,
    pub right: DisplayColor,
    pub bottom: DisplayColor,
    pub left: DisplayColor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorderRadius {
    pub top_left: Dimension,
    pub top_right: Dimension,
    pub bottom_right: Dimension,
    pub bottom_left: Dimension,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexStyle {
    pub align_content: Option<FlexAlign>,
    pub align_items: Option<FlexAlign>,
    pub align_self: Option<FlexAlign>,
    pub background_color: Option<DisplayColor>,
    pub background_image: Option<SpriteAddress>,
    pub border_color: Option<BorderColor>,
    pub border_radius: Option<BorderRadius>,
    pub border_width: Option<BorderWidth>,
    pub inset: Option<FlexInsets>,
    pub color: Option<DisplayColor>,
    pub display: Option<FlexDisplayStyle>,
    pub flex_basis: Option<Dimension>,
    pub flex_direction: Option<FlexDirection>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub wrap: Option<FlexWrap>,
    pub font_size: Option<Dimension>,
    pub height: Option<Dimension>,
    pub justify_content: Option<FlexJustify>,
    pub letter_spacing: Option<Dimension>,
    pub margin: Option<DimensionGroup>,
    pub max_height: Option<Dimension>,
    pub max_width: Option<Dimension>,
    pub min_height: Option<Dimension>,
    pub min_width: Option<Dimension>,
    pub opacity: Option<f32>,
    pub overflow: Option<FlexVisibility>,
    pub padding: Option<DimensionGroup>,
    pub position: Option<FlexPosition>,
    pub rotate: Option<FlexRotate>,
    pub scale: Option<FlexScale>,
    pub text_overflow: Option<TextOverflow>,
    pub text_shadow: Option<TextShadow>,
    pub transform_origin: Option<FlexTranslate>,
    pub transition_delays: Vec<Milliseconds>,
    pub transition_durations: Vec<Milliseconds>,
    pub transition_properties: Vec<String>,
    pub transition_easing_modes: Vec<EasingMode>,
    pub translate: Option<FlexTranslate>,
    pub background_image_tint_color: Option<DisplayColor>,
    pub font: Option<FontAddress>,
    pub font_style: Option<FontStyle>,
    pub overflow_clip_box: Option<OverflowClipBox>,
    pub paragraph_spacing: Option<Dimension>,
    pub image_slice: Option<ImageSlice>,
    pub text_align: Option<TextAlign>,
    pub text_outline_color: Option<DisplayColor>,
    pub text_outline_width: Option<f32>,
    pub text_overflow_position: Option<TextOverflowPosition>,
    pub visibility: Option<FlexVisibility>,
    pub white_space: Option<WhiteSpace>,
    pub width: Option<Dimension>,
    pub word_spacing: Option<Dimension>,
    pub picking_mode: Option<FlexPickingMode>,
}
