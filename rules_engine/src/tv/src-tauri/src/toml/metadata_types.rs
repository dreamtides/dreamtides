// Metadata schema types for TV TOML files.
//
// The [metadata] section stores spreadsheet configuration including column
// widths, alignment, validation rules, derived column definitions, styling,
// sort/filter state, and application settings.
//
// Schema version 1 is the current version. TV rejects metadata with higher
// versions to maintain forward compatibility.

use serde::{Deserialize, Serialize};

use crate::validation::validation_rules::ValidationRule;

/// Current metadata schema version.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Root metadata structure for a TOML file.
///
/// Contains all spreadsheet configuration that persists across sessions.
/// The metadata section is never displayed in the spreadsheet grid.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct Metadata {
    /// Schema version for forward compatibility. Defaults to 1.
    pub schema_version: u32,

    /// Column configurations keyed by TOML field name.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<ColumnConfig>,

    /// Derived column definitions for computed values.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_columns: Vec<DerivedColumnConfig>,

    /// Validation rules for data entry constraints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_rules: Vec<ValidationRule>,

    /// Conditional formatting rules.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditional_formatting: Vec<ConditionalFormatRule>,

    /// Table styling configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub table_style: Option<TableStyle>,

    /// Sort state for visual ordering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<SortConfig>,

    /// Filter configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<FilterConfig>,

    /// Row-specific configurations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<RowConfig>,

    /// Application settings stored with the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_settings: Option<AppSettings>,
}

impl Metadata {
    /// Creates a new metadata with default values.
    pub fn new() -> Self {
        Self { schema_version: CURRENT_SCHEMA_VERSION, ..Default::default() }
    }

    /// Returns true if this metadata has a compatible schema version.
    pub fn is_version_compatible(&self) -> bool {
        self.schema_version <= CURRENT_SCHEMA_VERSION
    }

    /// Gets the column configuration for a specific column, if defined.
    pub fn get_column_config(&self, key: &str) -> Option<&ColumnConfig> {
        self.columns.iter().find(|c| c.key == key)
    }

    /// Gets the derived column configuration for a specific column name.
    pub fn get_derived_column_config(&self, name: &str) -> Option<&DerivedColumnConfig> {
        self.derived_columns.iter().find(|c| c.name == name)
    }

    /// Gets all validation rules for a specific column.
    pub fn get_validation_rules_for_column(&self, column: &str) -> Vec<&ValidationRule> {
        self.validation_rules.iter().filter(|r| r.column() == column).collect()
    }
}

/// Configuration for a single data column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnConfig {
    /// TOML field name this configuration applies to. Required.
    pub key: String,

    /// Column width in pixels. Defaults to 100.
    #[serde(default = "default_column_width")]
    pub width: u32,

    /// Text alignment. Defaults to Left.
    #[serde(default)]
    pub alignment: Alignment,

    /// Enable text wrapping. Defaults to false.
    #[serde(default)]
    pub wrap: bool,

    /// Freeze this column in place. Defaults to false.
    #[serde(default)]
    pub frozen: bool,

    /// Hide this column from view. Defaults to false.
    #[serde(default)]
    pub hidden: bool,

    /// Number or date format pattern (e.g., "#,##0.00", "yyyy-mm-dd").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

fn default_column_width() -> u32 {
    100
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            key: String::new(),
            width: default_column_width(),
            alignment: Alignment::default(),
            wrap: false,
            frozen: false,
            hidden: false,
            format: None,
        }
    }
}

impl ColumnConfig {
    /// Creates a new column configuration with the given key.
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), ..Default::default() }
    }

    /// Builder method to set width.
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Builder method to set alignment.
    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Builder method to set frozen state.
    pub fn with_frozen(mut self, frozen: bool) -> Self {
        self.frozen = frozen;
        self
    }
}

/// Text alignment options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Configuration for a derived (computed) column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DerivedColumnConfig {
    /// Display name for the column header. Required.
    pub name: String,

    /// Registered function name to compute values. Required.
    pub function: String,

    /// Column position (0-indexed). Defaults to end of columns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<usize>,

    /// Column width in pixels. Defaults to 100.
    #[serde(default = "default_column_width")]
    pub width: u32,

    /// Input field names passed to the function.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,

    /// URL template for image-related derived functions.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_template: Option<String>,
}

impl DerivedColumnConfig {
    /// Creates a new derived column configuration.
    pub fn new(name: impl Into<String>, function: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            function: function.into(),
            position: None,
            width: default_column_width(),
            inputs: Vec::new(),
            url_template: None,
        }
    }

    /// Builder method to set inputs.
    pub fn with_inputs(mut self, inputs: Vec<String>) -> Self {
        self.inputs = inputs;
        self
    }

    /// Builder method to set width.
    pub fn with_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Builder method to set position.
    pub fn with_position(mut self, position: usize) -> Self {
        self.position = Some(position);
        self
    }
}

/// Conditional formatting rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionalFormatRule {
    /// Column this rule applies to.
    pub column: String,

    /// Condition to evaluate.
    pub condition: FormatCondition,

    /// Style to apply when condition is met.
    pub style: FormatStyle,
}

impl ConditionalFormatRule {
    /// Creates a new conditional format rule.
    pub fn new(column: impl Into<String>, condition: FormatCondition, style: FormatStyle) -> Self {
        Self { column: column.into(), condition, style }
    }
}

/// Condition for conditional formatting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FormatCondition {
    /// Exact value match.
    Equals(serde_json::Value),
    /// Substring presence.
    Contains(String),
    /// Numeric greater than.
    GreaterThan(f64),
    /// Numeric less than.
    LessThan(f64),
    /// Cell is empty.
    IsEmpty,
    /// Cell is not empty.
    NotEmpty,
    /// Regex pattern match.
    Matches(String),
}

/// Style for conditional formatting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct FormatStyle {
    /// Background color as hex string (e.g., "#FF0000").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,

    /// Font color as hex string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_color: Option<String>,

    /// Bold text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,

    /// Italic text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,

    /// Underlined text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underline: Option<bool>,
}

impl FormatStyle {
    /// Creates a new empty format style.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method to set background color.
    pub fn with_background_color(mut self, color: impl Into<String>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    /// Builder method to set font color.
    pub fn with_font_color(mut self, color: impl Into<String>) -> Self {
        self.font_color = Some(color.into());
        self
    }

    /// Builder method to set bold.
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }
}

/// Table styling configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct TableStyle {
    /// Color scheme name (e.g., "blue_light", "green_medium", "gray_classic").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme: Option<String>,

    /// Show alternating row colors. Defaults to true.
    pub show_row_stripes: bool,

    /// Show alternating column colors. Defaults to false.
    pub show_column_stripes: bool,

    /// Bold header text. Defaults to true.
    pub header_bold: bool,

    /// Override hex color for header row.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_background: Option<String>,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            color_scheme: None,
            show_row_stripes: true,
            show_column_stripes: false,
            header_bold: true,
            header_background: None,
        }
    }
}

impl TableStyle {
    /// Creates a new table style with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method to set color scheme.
    pub fn with_color_scheme(mut self, scheme: impl Into<String>) -> Self {
        self.color_scheme = Some(scheme.into());
        self
    }
}

/// Sort configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SortConfig {
    /// Column key to sort by.
    pub column: String,

    /// Sort direction. Defaults to ascending.
    #[serde(default)]
    pub ascending: bool,
}

impl SortConfig {
    /// Creates a new ascending sort configuration.
    pub fn ascending(column: impl Into<String>) -> Self {
        Self { column: column.into(), ascending: true }
    }

    /// Creates a new descending sort configuration.
    pub fn descending(column: impl Into<String>) -> Self {
        Self { column: column.into(), ascending: false }
    }
}

impl Default for SortConfig {
    fn default() -> Self {
        Self { column: String::new(), ascending: true }
    }
}

/// Filter configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct FilterConfig {
    /// Individual column filters.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<ColumnFilter>,

    /// Whether filters are currently active.
    pub active: bool,
}

impl FilterConfig {
    /// Creates a new empty filter configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a filter to the configuration.
    pub fn add_filter(&mut self, filter: ColumnFilter) {
        self.filters.push(filter);
    }
}

/// Filter for a single column.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColumnFilter {
    /// Column key this filter applies to.
    pub column: String,

    /// Filter condition.
    pub condition: FilterCondition,
}

impl ColumnFilter {
    /// Creates a new column filter.
    pub fn new(column: impl Into<String>, condition: FilterCondition) -> Self {
        Self { column: column.into(), condition }
    }
}

/// Filter condition types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterCondition {
    /// Text contains substring.
    Contains(String),
    /// Exact value match.
    Equals(serde_json::Value),
    /// Numeric range.
    Range { min: Option<f64>, max: Option<f64> },
    /// Boolean value.
    Boolean(bool),
}

/// Row-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct RowConfig {
    /// Height overrides by row index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heights: Vec<RowHeight>,

    /// Hidden row indices.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden: Vec<usize>,
}

/// Height configuration for a specific row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RowHeight {
    /// Row index (0-indexed, excluding header).
    pub row: usize,
    /// Height in pixels.
    pub height: u32,
}

impl RowHeight {
    /// Creates a new row height configuration.
    pub fn new(row: usize, height: u32) -> Self {
        Self { row, height }
    }
}

/// Application settings stored with the file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct AppSettings {
    /// Last selected cell in A1 notation (e.g., "B5").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_selected_cell: Option<String>,

    /// Scroll position for view restoration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_position: Option<ScrollPosition>,

    /// Zoom level as a multiplier. Defaults to 1.0.
    #[serde(default = "default_zoom_level")]
    pub zoom_level: f64,
}

fn default_zoom_level() -> f64 {
    1.0
}

impl AppSettings {
    /// Creates new app settings with defaults.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Scroll position for view restoration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScrollPosition {
    /// First visible row index.
    pub row: usize,
    /// First visible column index.
    pub column: usize,
}

impl ScrollPosition {
    /// Creates a new scroll position.
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }
}
