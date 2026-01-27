use serde::{Deserialize, Serialize};

/// Resolved color palette for a table color scheme.
///
/// Contains all concrete hex colors needed to render a styled table,
/// resolved from a named color scheme preset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorPalette {
    pub header_background: String,
    pub header_font_color: String,
    pub row_even_background: String,
    pub row_odd_background: String,
    pub row_font_color: String,
    pub accent_color: String,
}

/// Returns the [ColorPalette] for a named color scheme, or None if
/// the scheme name is not recognized.
pub fn resolve_color_scheme(name: &str) -> Option<ColorPalette> {
    match name {
        "blue_light" => Some(ColorPalette {
            header_background: "#4472C4".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#D6E4F0".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#4472C4".to_string(),
        }),
        "blue_medium" => Some(ColorPalette {
            header_background: "#4472C4".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#B4C6E7".to_string(),
            row_odd_background: "#D6E4F0".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#4472C4".to_string(),
        }),
        "blue_dark" => Some(ColorPalette {
            header_background: "#2F5597".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#4472C4".to_string(),
            row_odd_background: "#2F5597".to_string(),
            row_font_color: "#FFFFFF".to_string(),
            accent_color: "#2F5597".to_string(),
        }),
        "green_light" => Some(ColorPalette {
            header_background: "#70AD47".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#E2EFDA".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#70AD47".to_string(),
        }),
        "green_medium" => Some(ColorPalette {
            header_background: "#70AD47".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#C6E0B4".to_string(),
            row_odd_background: "#E2EFDA".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#70AD47".to_string(),
        }),
        "green_dark" => Some(ColorPalette {
            header_background: "#548235".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#70AD47".to_string(),
            row_odd_background: "#548235".to_string(),
            row_font_color: "#FFFFFF".to_string(),
            accent_color: "#548235".to_string(),
        }),
        "orange_light" => Some(ColorPalette {
            header_background: "#ED7D31".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#FCE4D6".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#ED7D31".to_string(),
        }),
        "orange_medium" => Some(ColorPalette {
            header_background: "#ED7D31".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#F8CBAD".to_string(),
            row_odd_background: "#FCE4D6".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#ED7D31".to_string(),
        }),
        "gray_classic" => Some(ColorPalette {
            header_background: "#A5A5A5".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#EDEDED".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#A5A5A5".to_string(),
        }),
        "gray_dark" => Some(ColorPalette {
            header_background: "#595959".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#A5A5A5".to_string(),
            row_odd_background: "#808080".to_string(),
            row_font_color: "#FFFFFF".to_string(),
            accent_color: "#595959".to_string(),
        }),
        "gold_light" => Some(ColorPalette {
            header_background: "#FFC000".to_string(),
            header_font_color: "#000000".to_string(),
            row_even_background: "#FFF2CC".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#FFC000".to_string(),
        }),
        "purple_light" => Some(ColorPalette {
            header_background: "#7030A0".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#E2D0F0".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#7030A0".to_string(),
        }),
        "red_light" => Some(ColorPalette {
            header_background: "#FF0000".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#FFC7CE".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#FF0000".to_string(),
        }),
        "black" => Some(ColorPalette {
            header_background: "#333333".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#E0E0E0".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#333333".to_string(),
        }),
        "orange" => Some(ColorPalette {
            header_background: "#ED7D31".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#FCE4D6".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#ED7D31".to_string(),
        }),
        "gray" => Some(ColorPalette {
            header_background: "#A5A5A5".to_string(),
            header_font_color: "#FFFFFF".to_string(),
            row_even_background: "#EDEDED".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#A5A5A5".to_string(),
        }),
        "yellow" => Some(ColorPalette {
            header_background: "#FFC000".to_string(),
            header_font_color: "#000000".to_string(),
            row_even_background: "#FFF2CC".to_string(),
            row_odd_background: "#FFFFFF".to_string(),
            row_font_color: "#000000".to_string(),
            accent_color: "#FFC000".to_string(),
        }),
        _ => None,
    }
}

/// Returns all available color scheme names.
pub fn available_schemes() -> Vec<&'static str> {
    vec![
        "blue_light",
        "blue_medium",
        "blue_dark",
        "green_light",
        "green_medium",
        "green_dark",
        "orange_light",
        "orange_medium",
        "gray_classic",
        "gray_dark",
        "gold_light",
        "purple_light",
        "red_light",
        "black",
        "orange",
        "gray",
        "yellow",
    ]
}
