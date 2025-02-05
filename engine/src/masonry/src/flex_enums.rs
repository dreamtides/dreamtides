use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexAlign {
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexDisplayStyle {
    Flex,
    None,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexDirection {
    Column,
    ColumnReverse,
    Row,
    RowReverse,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexJustify {
    FlexStart,
    Center,
    FlexEnd,
    SpaceBetween,
    SpaceAround,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexOverflow {
    Visible,
    Hidden,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexPosition {
    Relative,
    Absolute,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum EasingMode {
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    Linear,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInCubic,
    EaseOutCubic,
    EaseInOutCubic,
    EaseInCirc,
    EaseOutCirc,
    EaseInOutCirc,
    EaseInElastic,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInBack,
    EaseOutBack,
    EaseInOutBack,
    EaseInBounce,
    EaseOutBounce,
    EaseInOutBounce,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ImageScaleMode {
    StretchToFill,
    ScaleAndCrop,
    ScaleToFit,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FontStyle {
    Normal,
    Bold,
    Italic,
    BoldAndItalic,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OverflowClipBox {
    PaddingBox,
    ContentBox,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    UpperLeft,
    UpperCenter,
    UpperRight,
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    LowerLeft,
    LowerCenter,
    LowerRight,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TextOverflowPosition {
    End,
    Start,
    Middle,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FlexVisibility {
    Visible,
    Hidden,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum WhiteSpace {
    Normal,
    NoWrap,
}
