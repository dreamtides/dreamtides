use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a color with the given RGBA values represented as floats in the
/// 0-1 range.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl DisplayColor {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self { red, green, blue, alpha }
    }
}

const fn color(red: f32, green: f32, blue: f32, alpha: f32) -> DisplayColor {
    DisplayColor { red, green, blue, alpha }
}

// Converted from hex using https://www.tydac.ch/color/
pub const TRANSPARENT: DisplayColor = color(1.0, 1.0, 1.0, 0.0);
pub const WHITE: DisplayColor = color(1.0, 1.0, 1.0, 1.0);
pub const WHITE_ALPHA_25: DisplayColor = color(1.0, 1.0, 1.0, 0.25);
pub const WHITE_ALPHA_75: DisplayColor = color(1.0, 1.0, 1.0, 0.75);
pub const BLACK: DisplayColor = color(0.0, 0.0, 0.0, 1.0);
pub const BLACK_ALPHA_25: DisplayColor = color(0.0, 0.0, 0.0, 0.25);
pub const BLACK_ALPHA_50: DisplayColor = color(0.0, 0.0, 0.0, 0.5);
pub const BLACK_ALPHA_75: DisplayColor = color(0.0, 0.0, 0.0, 0.75);
pub const RED_100: DisplayColor = color(1.0, 0.8, 0.82, 1.0);
pub const RED_500: DisplayColor = color(0.96, 0.26, 0.21, 1.0);
pub const RED_600: DisplayColor = color(0.9, 0.22, 0.21, 1.0);
pub const RED_700: DisplayColor = color(0.83, 0.18, 0.18, 1.0);
pub const RED_800: DisplayColor = color(0.78, 0.16, 0.16, 1.0);
pub const RED_900: DisplayColor = color(0.72, 0.11, 0.11, 1.0);
pub const RED_900_ALPHA_75: DisplayColor = color(0.72, 0.11, 0.11, 0.75);
pub const BLUE_500: DisplayColor = color(0.13, 0.59, 0.95, 1.0);
pub const BLUE_700: DisplayColor = color(0.1, 0.46, 0.82, 1.0);
pub const BLUE_900: DisplayColor = color(0.05, 0.28, 0.63, 1.0);
pub const GREEN_500: DisplayColor = color(0.3, 0.69, 0.31, 1.0);
pub const GREEN_700: DisplayColor = color(0.22, 0.56, 0.24, 1.0);
pub const GREEN_900: DisplayColor = color(0.11, 0.37, 0.13, 1.0);
pub const GREEN_900_ALPHA_75: DisplayColor = color(0.11, 0.37, 0.13, 0.75);
pub const GREEN: DisplayColor = color(0.0, 1.0, 0.0, 1.0);
pub const YELLOW_500: DisplayColor = color(1.0, 0.92, 0.23, 1.0);
pub const YELLOW_700: DisplayColor = color(0.98, 0.75, 0.18, 1.0);
pub const YELLOW_900: DisplayColor = color(0.96, 0.5, 0.09, 1.0);
pub const PINK_500: DisplayColor = color(0.91, 0.12, 0.39, 1.0);
pub const PINK_700: DisplayColor = color(0.76, 0.09, 0.36, 1.0);
pub const PINK_900: DisplayColor = color(0.53, 0.05, 0.31, 1.0);
pub const ORANGE_500: DisplayColor = color(1.0, 0.6, 0.0, 1.0);
pub const ORANGE_700: DisplayColor = color(0.96, 0.49, 0.0, 1.0);
pub const ORANGE_900: DisplayColor = color(0.9, 0.32, 0.0, 1.0);
pub const GRAY_500: DisplayColor = color(0.62, 0.62, 0.62, 1.0);
pub const GRAY_700: DisplayColor = color(0.38, 0.38, 0.38, 1.0);
pub const GRAY_900: DisplayColor = color(0.13, 0.13, 0.13, 1.0);
pub const PURPLE_500: DisplayColor = color(0.40, 0.23, 0.72, 1.0);
pub const PURPLE_700: DisplayColor = color(0.32, 0.18, 0.66, 1.0);
pub const PURPLE_900: DisplayColor = color(0.19, 0.11, 0.57, 1.0);
