use bon::Builder;
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

pub struct Percent(pub i32);

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

impl From<Percent> for Dimension {
    fn from(value: Percent) -> Self {
        Self { unit: DimensionUnit::Percentage, value: value.0 as f32 }
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
    fn from((vertical, horizontal): (i32, i32)) -> Self {
        Self {
            top: vertical.into(),
            right: horizontal.into(),
            bottom: vertical.into(),
            left: horizontal.into(),
        }
    }
}

impl From<(f32, f32)> for DimensionGroup {
    fn from((vertical, horizontal): (f32, f32)) -> Self {
        Self {
            top: vertical.into(),
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, Builder)]
#[serde(rename_all = "camelCase")]
pub struct FlexInsets {
    #[builder(into)]
    pub top: Option<Dimension>,
    #[builder(into)]
    pub right: Option<Dimension>,
    #[builder(into)]
    pub bottom: Option<Dimension>,
    #[builder(into)]
    pub left: Option<Dimension>,
}
