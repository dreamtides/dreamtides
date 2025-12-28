use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum DimensionUnit {
    Pixels,
    Percentage,
    ViewportWidth,
    ViewportHeight,
    SafeAreaTopInset,
    SafeAreaRightInset,
    SafeAreaBottomInset,
    SafeAreaLeftInset,
}

pub struct Percent(pub i32);

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Dimension {
    pub unit: DimensionUnit,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DimensionGroup {
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, Builder)]
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, Builder)]
pub struct SafeAreaInsets {
    #[builder(into)]
    pub top: Option<i32>,
    #[builder(into)]
    pub right: Option<i32>,
    #[builder(into)]
    pub bottom: Option<i32>,
    #[builder(into)]
    pub left: Option<i32>,
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

impl FlexInsets {
    pub fn all(value: i32) -> Self {
        Self {
            top: Some(value.into()),
            right: Some(value.into()),
            bottom: Some(value.into()),
            left: Some(value.into()),
        }
    }
}

impl From<SafeAreaInsets> for FlexInsets {
    fn from(value: SafeAreaInsets) -> Self {
        Self {
            top: value.top.map(|value| Dimension {
                unit: DimensionUnit::SafeAreaTopInset,
                value: value as f32,
            }),
            right: value.right.map(|value| Dimension {
                unit: DimensionUnit::SafeAreaRightInset,
                value: value as f32,
            }),
            bottom: value.bottom.map(|value| Dimension {
                unit: DimensionUnit::SafeAreaBottomInset,
                value: value as f32,
            }),
            left: value.left.map(|value| Dimension {
                unit: DimensionUnit::SafeAreaLeftInset,
                value: value as f32,
            }),
        }
    }
}
