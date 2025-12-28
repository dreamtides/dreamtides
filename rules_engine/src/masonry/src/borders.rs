use core_data::display_color::DisplayColor;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::dimension::Dimension;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct BorderWidth {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct BorderColor {
    pub top: DisplayColor,
    pub right: DisplayColor,
    pub bottom: DisplayColor,
    pub left: DisplayColor,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub struct BorderRadius {
    pub top_left: Dimension,
    pub top_right: Dimension,
    pub bottom_right: Dimension,
    pub bottom_left: Dimension,
}

impl From<f32> for BorderWidth {
    fn from(value: f32) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }
}

impl From<i32> for BorderWidth {
    fn from(value: i32) -> Self {
        Self { top: value as f32, right: value as f32, bottom: value as f32, left: value as f32 }
    }
}

impl From<(f32, f32)> for BorderWidth {
    fn from((horizontal, vertical): (f32, f32)) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
}

impl From<(i32, i32)> for BorderWidth {
    fn from((horizontal, vertical): (i32, i32)) -> Self {
        Self {
            top: vertical as f32,
            right: horizontal as f32,
            bottom: vertical as f32,
            left: horizontal as f32,
        }
    }
}

impl From<(f32, f32, f32, f32)> for BorderWidth {
    fn from((top, right, bottom, left): (f32, f32, f32, f32)) -> Self {
        Self { top, right, bottom, left }
    }
}

impl From<(i32, i32, i32, i32)> for BorderWidth {
    fn from((top, right, bottom, left): (i32, i32, i32, i32)) -> Self {
        Self { top: top as f32, right: right as f32, bottom: bottom as f32, left: left as f32 }
    }
}

impl From<DisplayColor> for BorderColor {
    fn from(value: DisplayColor) -> Self {
        Self { top: value, right: value, bottom: value, left: value }
    }
}

impl From<(DisplayColor, DisplayColor)> for BorderColor {
    fn from((horizontal, vertical): (DisplayColor, DisplayColor)) -> Self {
        Self { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
}

impl From<(DisplayColor, DisplayColor, DisplayColor, DisplayColor)> for BorderColor {
    fn from(
        (top, right, bottom, left): (DisplayColor, DisplayColor, DisplayColor, DisplayColor),
    ) -> Self {
        Self { top, right, bottom, left }
    }
}

impl From<f32> for BorderRadius {
    fn from(value: f32) -> Self {
        Self {
            top_left: value.into(),
            top_right: value.into(),
            bottom_right: value.into(),
            bottom_left: value.into(),
        }
    }
}

impl From<i32> for BorderRadius {
    fn from(value: i32) -> Self {
        Self {
            top_left: value.into(),
            top_right: value.into(),
            bottom_right: value.into(),
            bottom_left: value.into(),
        }
    }
}

impl From<(f32, f32, f32, f32)> for BorderRadius {
    fn from((top_left, top_right, bottom_right, bottom_left): (f32, f32, f32, f32)) -> Self {
        Self {
            top_left: top_left.into(),
            top_right: top_right.into(),
            bottom_right: bottom_right.into(),
            bottom_left: bottom_left.into(),
        }
    }
}

impl From<(i32, i32, i32, i32)> for BorderRadius {
    fn from((top_left, top_right, bottom_right, bottom_left): (i32, i32, i32, i32)) -> Self {
        Self {
            top_left: top_left.into(),
            top_right: top_right.into(),
            bottom_right: bottom_right.into(),
            bottom_left: bottom_left.into(),
        }
    }
}
