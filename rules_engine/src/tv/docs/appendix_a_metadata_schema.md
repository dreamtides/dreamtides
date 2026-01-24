# Appendix A: Metadata Schema Specification

## Overview

The [metadata] table appears as the final entry in each TOML file and stores
all spreadsheet configuration. This section never appears in the spreadsheet
grid and is preserved across all edit operations.

## Schema Version

The schema includes a version field for forward compatibility. Current version
is 1. TV rejects metadata with versions higher than supported, preserving the
file without applying unknown configurations.

```
[metadata]
schema_version = 1
```

## Column Configuration

Column settings are stored in a columns array of tables. Each entry identifies
a column by its TOML key name and specifies display properties.

Fields per column entry:
- key: String matching the TOML field name, required
- width: Integer pixel width, defaults to 100
- alignment: Enum of "left", "center", "right", defaults to "left"
- wrap: Boolean for text wrapping, defaults to false
- frozen: Boolean for freeze pane, defaults to false
- hidden: Boolean for column visibility, defaults to false
- format: String number format pattern like "#,##0.00" or "yyyy-mm-dd"

## Derived Column Definitions

Derived columns are defined separately from regular columns since they have no
corresponding TOML field. Each derived column specifies a function name that
maps to a registered Rust function.

Fields per derived column entry:
- name: String display header name, required
- function: String matching a registered function name, required
- position: Integer column index for insertion, defaults to end
- width: Integer pixel width, same as regular columns
- inputs: Array of input field names passed to the function

## Data Validation Rules

Validation rules specify constraints per column. Multiple rules can apply to
the same column and all must pass for a value to be accepted.

Rule types:
- type: Enum of "string", "integer", "float", "boolean"
- enum: Array of allowed string values, renders as dropdown
- min: Numeric minimum bound inclusive
- max: Numeric maximum bound inclusive
- pattern: Regex pattern string for text validation
- required: Boolean preventing empty cells

## Conditional Formatting

Formatting rules apply styles when conditions match. Rules are evaluated in
order with later rules overriding earlier ones for the same style property.

Condition types:
- equals: Exact value match
- contains: Substring presence
- greater_than, less_than: Numeric comparison
- is_empty, not_empty: Presence check
- matches: Regex pattern match

Style properties:
- background_color: Hex color string like "#FF0000"
- font_color: Hex color string
- bold: Boolean
- italic: Boolean
- underline: Boolean

## Table Styling

Global table appearance settings control the overall visual theme.

Fields:
- color_scheme: String name from predefined schemes like "blue_light",
  "green_medium", "gray_classic", matching Excel table styles
- show_row_stripes: Boolean for alternating row colors, defaults to true
- show_column_stripes: Boolean for alternating column colors, defaults to false
- header_bold: Boolean for bold header text, defaults to true
- header_background: Override hex color for header row

## Sort and Filter State

Persistent sort and filter configuration for restoring view state.

Sort fields:
- sort_column: String key of sorted column, empty for no sort
- sort_ascending: Boolean sort direction, defaults to true

Filter fields:
- filters: Array of filter objects with column key and filter value/condition
- active: Boolean indicating if filters are currently applied

## Row Configuration

Per-row settings stored by row index.

Fields:
- height: Integer pixel height override for specific rows
- hidden: Array of integer indices for hidden rows

## Application Settings

General application preferences stored with the file.

Fields:
- last_selected_cell: String in A1 notation for cursor restoration
- scroll_position: Object with row and column integers for scroll restoration
- zoom_level: Float zoom percentage, defaults to 1.0

## Example Complete Metadata

```
[metadata]
schema_version = 1
color_scheme = "blue_light"
show_row_stripes = true
sort_column = "name"
sort_ascending = true

[[metadata.columns]]
key = "id"
width = 300
frozen = true

[[metadata.columns]]
key = "name"
width = 200
alignment = "left"

[[metadata.derived_columns]]
name = "Preview"
function = "rules_preview"
position = 5
width = 400
inputs = ["rules_text", "variables"]

[[metadata.validation_rules]]
column = "card_type"
type = "enum"
enum = ["Character", "Event"]

[[metadata.conditional_formatting]]
column = "rarity"
condition = { equals = "Rare" }
style = { background_color = "#FFD700", bold = true }
```
