# Appendix D: Univer Integration Details

## Official Resources

Univer documentation: https://docs.univer.ai
Facade API guide: https://docs.univer.ai/guides/sheets/getting-started/facade
Facade API reference: https://reference.univer.ai/en-US
GitHub repository: https://github.com/dream-num/univer
Floating images guide: https://univer.ai/guides/sheet/features/floating-images

## Plugin Configuration

TV initializes Univer with a comprehensive plugin set. Core plugins are loaded
first, followed by feature plugins. The formula plugin is included to support
any spreadsheet formulas users may want to use.

Required plugins in load order:
- UniverRenderEnginePlugin: Canvas rendering engine
- UniverFormulaEnginePlugin: Formula calculation engine
- UniverUIPlugin: Basic UI framework with container mounting
- UniverDocsPlugin: Text cell content support
- UniverDocsUIPlugin: Text editing UI
- UniverSheetsPlugin: Core spreadsheet functionality
- UniverSheetsUIPlugin: Spreadsheet UI components
- UniverSheetsFormulaPlugin: Formula support for sheets
- UniverSheetsFormulaUIPlugin: Formula bar and editor
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
- UniverSheetsConditionalFormattingUIPlugin: Formatting rules UI
- UniverSheetsDrawingPlugin: Drawing layer for images
- UniverSheetsDrawingUIPlugin: Drawing UI controls

## Image Rendering Support

Univer supports cell images through the @univerjs/sheets-drawing-ui package.
This feature was added via GitHub PR #4036 and refined in PR #4729 which added
the cell image Facade API.

### Required Packages for Images
- @univerjs/drawing: Core drawing functionality
- @univerjs/drawing-ui: Drawing UI layer
- @univerjs/sheets-drawing: Sheet-specific drawing support
- @univerjs/sheets-drawing-ui: Cell image insertion and rendering

### Image Insertion API
The Facade API provides the insertImage method on FWorksheet:

Import the facade extension: import '@univerjs/sheets-drawing-ui/facade'
Get the active sheet: const sheet = univerAPI.getActiveWorkbook().getActiveSheet()
Insert image: sheet.insertImage(urlOrBase64, rowIndex, columnIndex)

The method accepts either a URL string or a base64 data URI. The image renders
within the target cell, scaling to fit the cell dimensions.

### Image Limitations
Cell images no longer support mixed layout with text in the same cell. This
was changed in PR #4729 to simplify the rendering model. A cell contains
either text content or an image, not both.

Performance issues with many cell images on initial render have been addressed
in recent versions. The rendering pipeline now handles large numbers of images
more efficiently through lazy loading.

### Browser Data URL Limits
Base64-encoded images use data URLs which have browser-specific limits:
- Chromium/Firefox: 512MB limit
- Safari (WebKit): 2GB limit
These limits are far beyond typical card artwork sizes and are not a concern.

## Data Binding

Univer receives data through its FUniver facade API. Data is structured as
a workbook containing sheets. Each sheet maps to one TOML file. Rows map to
TOML array entries. Columns map to TOML table keys.

### Workbook Data Structure
The IWorkbookData interface defines the workbook structure:
- id: Unique workbook identifier
- name: Display name for the workbook
- sheets: Object mapping sheet IDs to ISheetData objects
- sheetOrder: Array of sheet IDs defining tab order

### Sheet Data Structure
Each ISheetData object contains:
- id: Unique sheet identifier
- name: Sheet tab label (TOML filename without extension)
- rowCount: Total rows including header
- columnCount: Number of columns from header keys
- cellData: Nested object mapping row -> column -> ICellData
- rowData: Row height configurations
- columnData: Column width configurations
- freeze: Frozen pane configuration

### Cell Data Structure
ICellData contains the cell value and styling:
- v: Cell value (string, number, boolean)
- s: Style ID or inline style object
- t: Cell type (string, number, boolean, formula)

### Data Flow on Load
1. Parse TOML file to extract headers and rows
2. Create IWorkbookData with sheet definitions
3. Populate cellData with header row in row 0
4. Populate cellData with data rows starting at row 1
5. Apply styling to header row (bold, background color)
6. Apply column widths from metadata
7. Call univerAPI.createWorkbook(workbookData)

## Change Detection

Univer emits command execution events for all operations. TV listens for
specific mutation commands indicating data changes.

### Relevant Commands
- sheet.mutation.set-range-values: Direct value edit from user input
- sheet.command.set-range-values: Programmatic value set
- sheet.mutation.delete-range: Cell or range deletion
- sheet.command.copy-paste: Paste operation
- sheet.mutation.insert-row: Row insertion
- sheet.mutation.remove-row: Row deletion

### Event Subscription
Subscribe using the Facade API event system:
univerAPI.addEvent(univerAPI.Event.CommandExecuted, (event) => { ... })

The event object contains:
- id: Command identifier string
- params: Command parameters including affected range

### Extracting Changed Data
When a mutation occurs, extract the affected range from params. The range
includes startRow, endRow, startColumn, endColumn. Iterate through the range
to collect changed cell values. Convert values back to TOML-compatible types.

### Preventing Feedback Loops
Use a loading flag to distinguish programmatic updates from user edits. When
loading data into Univer, set the flag true. Command listeners check the flag
and ignore events that occur during loading. This prevents save-on-load cycles.

## Cell Rendering Customization

### Image Cells
Image cells use the Univer drawing layer via insertImage. The backend sends
image data as base64-encoded strings. The frontend calls insertImage with
the data URI and target cell coordinates. Images automatically scale to fit
cell dimensions while preserving aspect ratio.

For URL-based images, the backend can pass the URL directly if cross-origin
access is permitted. Otherwise, the backend fetches and encodes the image.

### Checkbox Cells
Univer's data validation plugin supports checkbox rendering. Configure a
validation rule with type 'checkbox' on boolean columns. The plugin renders
an interactive checkbox that toggles on click.

Checkbox configuration in cell data:
- Set validation rule type to DataValidationType.CHECKBOX
- Univer renders checkbox UI automatically
- Value changes trigger standard mutation events

### Dropdown Cells
Dropdown cells use data validation with type 'list'. The validation rule
specifies allowed values as an array. Univer renders a dropdown arrow that
opens a selection list on click.

Dropdown configuration:
- Set validation rule type to DataValidationType.LIST
- Provide formula1 with comma-separated allowed values
- Univer renders dropdown UI and enforces selection

### Rich Text Cells
Rich text uses Univer's document model within cells. Text spans carry styling
properties including bold, italic, underline, and color.

Rich text structure in ICellData:
- p: Array of paragraph objects
- Each paragraph contains array of text runs
- Each run has text content and style properties

Style properties for text runs:
- bl: Bold (1 for bold, 0 for normal)
- it: Italic (1 for italic, 0 for normal)
- ul: Underline object with style property
- cl: Font color as RGB object

## Styling Application

### Cell Style Properties
Cell styles are applied through ICellData.s which references a style object.
Key style properties:
- bg: Background color as RGB object {rgb: 'FFFFFF'}
- cl: Font color as RGB object
- bl: Bold flag (1 or 0)
- it: Italic flag (1 or 0)
- ul: Underline object {s: 1} for single underline
- fs: Font size in points
- ff: Font family name
- ht: Horizontal alignment (0=left, 1=center, 2=right)
- vt: Vertical alignment (0=top, 1=middle, 2=bottom)
- tb: Text wrapping (0=clip, 1=overflow, 2=wrap)

### Applying Styles Programmatically
Get range and apply styles via Facade API:
const range = sheet.getRange(row, col, rowCount, colCount)
range.setBackgroundColor('#FFFFFF')
range.setFontWeight('bold')
range.setFontColor('#000000')

### Conditional Formatting
Register rules with UniverSheetsConditionalFormattingPlugin. Rules specify:
- Range of cells to evaluate
- Condition type and parameters
- Style to apply when condition matches

Condition types include:
- Cell value comparisons (greater than, less than, equals)
- Text contains, begins with, ends with
- Blank, not blank
- Custom formula

## Column Width Management

### Setting Column Widths
Apply widths through column data in ISheetData:
columnData: { 0: { w: 150 }, 1: { w: 200 } }

Or via Facade API after creation:
sheet.setColumnWidth(columnIndex, width)

### Detecting Width Changes
Listen for column resize command:
sheet.mutation.set-worksheet-col-width

Extract new width from command parameters and save to metadata.

### Auto-sizing
Univer provides autoFitWidth method on FRange:
range.autoFitWidth()

This calculates optimal width based on cell content. Apply to columns without
metadata-specified widths on initial load.

## Frozen Panes

### Configuration
Set freeze state in ISheetData:
freeze: { startRow: 1, startColumn: 1, xSplit: 1, ySplit: 1 }

This freezes the first row and first column. Values specify:
- startRow/startColumn: First unfrozen row/column
- xSplit/ySplit: Number of frozen columns/rows

### Via Facade API
sheet.setFreeze({ row: 1, column: 1 })

### Detecting Freeze Changes
Listen for freeze mutation commands and update metadata accordingly.

## Sheet Tab Navigation

### Multiple Sheets
Create workbook with multiple sheets in sheetOrder array. Each sheet has
unique ID and name. Tab bar appears automatically when multiple sheets exist.

### Tab Events
Listen for sheet activation:
univerAPI.addEvent(univerAPI.Event.ActiveSheetChanged, (event) => { ... })

### Programmatic Navigation
Activate sheet by ID:
workbook.setActiveSheet(sheetId)

## Scroll and Selection Persistence

### Getting Current State
const selection = sheet.getSelection()
const activeCell = selection.getActiveCell()

### Setting Position on Load
sheet.setActiveCell(row, column)
Use scrollTo if available to center the viewport.

### Viewport Management
Univer virtualizes rendering based on visible viewport. Large sheets remain
performant as only visible cells are rendered. Scroll events update the
viewport and trigger incremental rendering.

## Performance Optimization

### Virtualized Rendering
Univer only renders cells within the visible viewport plus a buffer zone.
Scrolling triggers render updates for newly visible cells. This enables
responsive performance with thousands of rows.

### Batch Updates
When loading data, set all cell values before creating the workbook. This
avoids incremental updates that would trigger re-renders. Use transaction
API if making multiple changes after creation.

### Memory Management
Univer maintains cell data in memory. For very large sheets, consider
pagination or lazy loading of distant rows. The current TOML file sizes
are well within comfortable memory limits.

## Event Cleanup

### React Integration
Store disposable references from event subscriptions:
const disposable = univerAPI.addEvent(...)

In useEffect cleanup:
disposable.dispose()

### Workbook Disposal
When switching files or unmounting:
univer.dispose()

This releases all resources and event listeners associated with the Univer
instance. Create a fresh instance for new files.
