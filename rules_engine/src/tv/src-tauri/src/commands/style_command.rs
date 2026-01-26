use crate::error::error_types::TvError;
use crate::toml::color_schemes::{self, ColorPalette};
use crate::toml::metadata_parser;
use crate::toml::metadata_types::TableStyle;

/// Response combining the resolved color palette with table style flags.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedTableStyle {
    pub palette: Option<ColorPalette>,
    pub show_row_stripes: bool,
    pub show_column_stripes: bool,
    pub header_bold: bool,
    pub header_background: Option<String>,
}

/// Tauri command to get the resolved table style for a TOML file.
#[tauri::command]
pub fn get_table_style(file_path: String) -> Result<Option<ResolvedTableStyle>, TvError> {
    tracing::debug!(
        component = "tv.commands.style",
        file_path = %file_path,
        "Loading table style"
    );

    let table_style = metadata_parser::parse_table_style_from_file(&file_path)?;

    let Some(style) = table_style else {
        tracing::debug!(
            component = "tv.commands.style",
            file_path = %file_path,
            "No table style found in metadata"
        );
        return Ok(None);
    };

    let palette = resolve_palette(&style);

    tracing::debug!(
        component = "tv.commands.style",
        file_path = %file_path,
        color_scheme = ?style.color_scheme,
        has_palette = palette.is_some(),
        "Table style resolved"
    );

    Ok(Some(ResolvedTableStyle {
        palette,
        show_row_stripes: style.show_row_stripes,
        show_column_stripes: style.show_column_stripes,
        header_bold: style.header_bold,
        header_background: style.header_background,
    }))
}

/// Tauri command to get available color scheme names.
#[tauri::command]
pub fn get_available_color_schemes() -> Vec<String> {
    color_schemes::available_schemes().into_iter().map(String::from).collect()
}

fn resolve_palette(style: &TableStyle) -> Option<ColorPalette> {
    style.color_scheme.as_deref().and_then(color_schemes::resolve_color_scheme)
}
