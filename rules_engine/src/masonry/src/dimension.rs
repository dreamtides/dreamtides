use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

impl From<i32> for Dimension {
    fn from(value: i32) -> Self {
        Self { unit: DimensionUnit::Pixels, value: value as f32 }
    }
}

impl From<f32> for Dimension {
    fn from(value: f32) -> Self {
        Self { unit: DimensionUnit::Pixels, value }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DimensionGroup {
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
}

impl From<i32> for DimensionGroup {
    fn from(value: i32) -> Self {
        Self { top: value.into(), right: value.into(), bottom: value.into(), left: value.into() }
    }
}

impl From<f32> for DimensionGroup {
    fn from(value: f32) -> Self {
        Self { top: value.into(), right: value.into(), bottom: value.into(), left: value.into() }
    }
}

impl From<(i32, i32)> for DimensionGroup {
    fn from((horizontal, vertical): (i32, i32)) -> Self {
        Self {
            top: horizontal.into(),
            right: horizontal.into(),
            bottom: vertical.into(),
            left: horizontal.into(),
        }
    }
}

impl From<(f32, f32)> for DimensionGroup {
    fn from((horizontal, vertical): (f32, f32)) -> Self {
        Self {
            top: horizontal.into(),
            right: horizontal.into(),
            bottom: vertical.into(),
            left: horizontal.into(),
        }
    }
}

impl From<(i32, i32, i32, i32)> for DimensionGroup {
    fn from((top, right, bottom, left): (i32, i32, i32, i32)) -> Self {
        Self { top: top.into(), right: right.into(), bottom: bottom.into(), left: left.into() }
    }
}

impl From<(f32, f32, f32, f32)> for DimensionGroup {
    fn from((top, right, bottom, left): (f32, f32, f32, f32)) -> Self {
        Self { top: top.into(), right: right.into(), bottom: bottom.into(), left: left.into() }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FlexInsets {
    pub top: Option<Dimension>,
    pub right: Option<Dimension>,
    pub bottom: Option<Dimension>,
    pub left: Option<Dimension>,
}
