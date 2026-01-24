# Appendix D: Univers Integration Details

## Plugin Configuration

TV initializes Univers with a specific plugin set optimized for TOML viewing.
Core plugins are loaded first, followed by feature plugins. Unnecessary plugins
like UniverSheetsFormulaPlugin are omitted since formulas are Rust-computed.

Required plugins in load order:
- UniverRenderEnginePlugin: Canvas rendering engine
- UniverUIPlugin: Basic UI framework with container mounting
- UniverDocsPlugin: Text cell content support
- UniverDocsUIPlugin: Text editing UI
- UniverSheetsPlugin: Core spreadsheet functionality
- UniverSheetsUIPlugin: Spreadsheet UI components
- UniverSheetsNumfmtPlugin: Number formatting support
- UniverSheetsNumfmtUIPlugin: Format picker UI
- UniverSheetsFilterPlugin: Filtering infrastructure
- UniverSheetsFilterUIPlugin: Filter panel UI
- UniverSheetsSortPlugin: Sorting infrastructure
- UniverSheetsSortUIPlugin: Sort controls
- UniverDataValidationPlugin: Validation rules engine
- UniverSheetsDataValidationPlugin: Sheet-specific validation
- UniverSheetsDataValidationUIPlugin: Validation error display
- UniverSheetsConditionalFormattingPlugin: Conditional styling

## Data Binding

Univers receives data through its FUniver facade API. Data is structured as
a workbook containing sheets. Each sheet maps to one TOML file. Rows map to
TOML array entries. Columns map to TOML table keys.

Data flow on load:
1. Create IWorkbookData object with sheet definitions
2. Each sheet defines column count from header length
3. Each sheet defines row count from data row count plus header
4. Row 0 contains headers with bold styling
5. Rows 1+ contain data values
6. Call univerAPI.createWorkbook(workbookData)

## Change Detection

Univers emits command execution events for all operations. TV listens for
specific mutation commands indicating data changes:
- sheet.mutation.set-range-values: Direct value edit
- sheet.command.set-range-values: Command-triggered value set
- sheet.mutation.delete-range: Cell or range deletion
- sheet.command.copy-paste: Paste operation

The event payload includes the target range and new values. TV extracts the
affected cells and forwards to the backend for saving.

## Cell Rendering Customization

Custom cell renderers handle special content types:

Image cells: Use Univers drawing layer to render images from base64 data.
Images scale to fit cell bounds with aspect ratio preservation. Placeholder
shown while image loads.

Checkbox cells: Render as centered checkbox graphics. Click handler toggles
value and triggers save. Checked state reflects boolean TOML value.

Dropdown cells: Standard Univers data validation dropdown. Options come from
enum validation rule in metadata. Selection updates cell value and saves.

Rich text cells: Use Univers rich text model with styled spans. Spans carry
bold, italic, underline, and color properties. HTML-like tags from Fluent
output are parsed into span definitions.

## Styling Application

Cell styles are applied through Univers ICellData style properties. Background
color uses bgColor property with hex string. Font styling uses bl for bold,
it for italic, ul for underline. Font color uses cl property.

Conditional formatting applies styles dynamically based on cell values. Rules
are registered with Univers conditional formatting plugin. The plugin
evaluates rules on render and merges matching styles.

Table color schemes define style templates applied globally. The header row
receives distinct styling. Alternating row colors apply via row index modulo.

## Column Width Management

Column widths are stored in metadata and applied on load. Width changes from
user drag operations are detected via Univers column resize events. Changed
widths are immediately saved to metadata.

Initial widths use auto-sizing based on content if no metadata width exists.
Auto-size calculates maximum content width across visible rows. A minimum
width prevents columns from becoming too narrow.

## Frozen Panes

Frozen columns and rows are configured through Univers freeze API. Freeze
state from metadata is applied on load. User changes to freeze state trigger
metadata updates.

Common configuration freezes the ID column and header row so they remain
visible while scrolling through large tables.

## Sheet Tab Navigation

Multiple sheets display tabs at the bottom of the spreadsheet. Tab labels
show the TOML file name without extension. Clicking a tab activates that
sheet, loading its content if not already loaded.

Active sheet tracks which file receives edits. Sheet activation triggers
file watcher attachment if not already watching.

## Scroll and Selection Persistence

Scroll position and selected cell are saved to metadata on change. On reload,
these positions are restored so the user returns to their previous location.
Selection persistence prevents disorientation after external file changes.

## Performance Optimization

Univers uses virtualized rendering, only drawing visible cells. Large TOML
files with thousands of rows remain responsive. Scroll events trigger
incremental rendering without full redraw.

Data binding uses reference semantics where possible to avoid copying large
datasets. Cell value access uses the facade API which provides efficient
indexed access.

## Event Cleanup

React effect cleanup detaches all Univers event listeners on component
unmount. Listener references are stored for cleanup. Cleanup prevents memory
leaks and stale event handlers after navigation.
