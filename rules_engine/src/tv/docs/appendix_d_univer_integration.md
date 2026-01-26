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

Univer supports two types of images in spreadsheets:
- **Floating Images (Over-Grid Images)**: Positioned over cells with pixel-level
  control, can span multiple cells
- **Cell Images**: Embedded directly within individual cells

TV uses floating images positioned at cell locations for card artwork display.

### Required Packages for Images
- @univerjs/drawing: Core drawing functionality
- @univerjs/drawing-ui: Drawing UI layer
- @univerjs/sheets-drawing: Sheet-specific drawing support
- @univerjs/sheets-drawing-ui: Image insertion and rendering facade

Or use the preset: @univerjs/preset-sheets-drawing

### Source Code References
Key facade files in @univerjs/sheets-drawing-ui/src/facade/:
- f-worksheet.ts: FWorksheet image methods (insertImage, getImages, etc.)
- f-over-grid-image.ts: FOverGridImage class and FOverGridImageBuilder
- f-range.ts: FRange.insertCellImageAsync for cell images
- f-event.ts: Image-related events (insert, update, delete)

GitHub: https://github.com/dream-num/univer/tree/dev/packages/sheets-drawing-ui/src/facade

### ImageSourceType Enum
Defined in @univerjs/core (packages/core/src/services/image-io/image-io.service.ts):

```typescript
enum ImageSourceType {
    URL = 'URL',      // HTTP/HTTPS URL string
    UUID = 'UUID',    // Univer image hosting service ID
    BASE64 = 'BASE64' // Data URI: data:image/png;base64,...
}
```

Access via: univerAPI.Enum.ImageSourceType.URL

### Floating Image API (FWorksheet)

**Simple Insertion:**
```typescript
// Import facade extension
import '@univerjs/sheets-drawing-ui/facade';

// Insert at specific cell (column 5, row 5 = F6)
const result = await sheet.insertImage(url, 5, 5);

// Insert with pixel offset from cell origin
const result = await sheet.insertImage(url, 5, 5, 10, 10);

// Insert from IFBlobSource (File/Blob converted to base64 internally)
const result = await sheet.insertImage(blobSource, 5, 5);
```

**Builder Pattern (Recommended for Control):**
```typescript
const image = await sheet.newOverGridImage()
    .setSource(url, univerAPI.Enum.ImageSourceType.URL)
    .setColumn(5)
    .setRow(5)
    .setWidth(200)    // pixels
    .setHeight(150)   // pixels
    .setColumnOffset(10)  // pixel offset from cell
    .setRowOffset(10)
    .setRotate(0)     // rotation in degrees
    .setAnchorType(anchorType)  // position behavior
    .buildAsync();

sheet.insertImages([image]);
```

**FOverGridImageBuilder Methods:**
- setSource(source: string, sourceType?: ImageSourceType)
- setColumn(column: number) / setRow(row: number)
- setColumnOffset(offset: number) / setRowOffset(offset: number)
- setWidth(width: number) / setHeight(height: number)
- setRotate(angle: number)
- setAnchorType(anchorType: SheetDrawingAnchorType)
- setCropTop/Left/Right/Bottom(percent: number)
- buildAsync(): Promise<ISheetImage>

**Managing Images:**
```typescript
// Get all images
const images: FOverGridImage[] = sheet.getImages();

// Get image by ID
const image = sheet.getImageById('drawingId');

// Update images
const updatedImage = await image.toBuilder()
    .setWidth(100)
    .setHeight(50)
    .buildAsync();
sheet.updateImages([updatedImage]);

// Delete images
sheet.deleteImages([image]);

// Get currently selected images
const activeImages = sheet.getActiveImages();
```

### Cell Image API (FRange)

For images embedded directly in cells:

```typescript
const range = sheet.getRange('A10');

// Insert from URL
const result = await range.insertCellImageAsync(url);

// Insert from File object
const result = await range.insertCellImageAsync(file);
```

Internally uses:
- controller.insertCellImageByUrl(url, location) for URL strings
- controller.insertCellImageByFile(file, location) for File objects

### Image Events

Subscribe to image lifecycle events:

```typescript
// Before insertion (can cancel)
univerAPI.addEvent(univerAPI.Event.BeforeOverGridImageInsert, (params) => {
    const { workbook, insertImageParams } = params;
    params.cancel = true; // Cancel insertion
});

// After insertion
univerAPI.addEvent(univerAPI.Event.OverGridImageInserted, (params) => {
    const { workbook, images } = params;
});

// Before/after update
univerAPI.addEvent(univerAPI.Event.BeforeOverGridImageChange, callback);
univerAPI.addEvent(univerAPI.Event.OverGridImageChanged, callback);

// Before/after deletion
univerAPI.addEvent(univerAPI.Event.BeforeOverGridImageRemove, callback);
univerAPI.addEvent(univerAPI.Event.OverGridImageRemoved, callback);
```

Deprecated callback methods (use events instead):
- sheet.onImageInserted(callback)
- sheet.onImageDeleted(callback)
- sheet.onImageChanged(callback)

### Supported Image Sources

1. **URL Strings (ImageSourceType.URL)**
   - HTTP/HTTPS URLs: `https://example.com/image.png`
   - Loaded asynchronously by browser
   - Subject to CORS restrictions for cross-origin URLs

2. **Base64 Data URIs (ImageSourceType.BASE64)**
   - Format: `data:image/png;base64,iVBORw0KGgo...`
   - No CORS restrictions
   - ~33% larger than binary

3. **IFBlobSource Objects**
   - Wrapper for File/Blob types
   - Converted to base64 internally by Univer
   - Use for file input handling

4. **UUID (ImageSourceType.UUID)**
   - For Univer's cloud image hosting service
   - Not applicable to TV

**NOT Supported:**
- `file://` URLs are not directly supported
- Use Tauri's asset protocol (convertFileSrc) to convert local paths

### Tauri Asset Protocol Integration

For local filesystem images in a Tauri app:

```typescript
import { convertFileSrc } from '@tauri-apps/api/core';

// Convert local path to asset URL
const localPath = '/path/to/cached/image.png';
const assetUrl = convertFileSrc(localPath);
// Returns: http://asset.localhost/path/to/cached/image.png

// Pass to Univer (recognized as regular URL)
await sheet.insertImage(assetUrl, column, row);
```

Required tauri.conf.json configuration:
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self' ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost",
      "assetProtocol": {
        "enable": true,
        "scope": {
          "requireLiteralLeadingDot": false,
          "allow": ["$APPDATA/**/*", "$CACHE/**/*"]
        }
      }
    }
  }
}
```

### Image Limitations

**Cell Images:**
- No mixed layout with text in same cell (removed in PR #4729)
- A cell contains either text content OR an image, not both

**Floating Images:**
- Can overlap cells freely
- Support rotation, cropping, and transforms
- Performance optimized for many images via lazy loading

### Browser Data URL Limits
Base64-encoded images use data URLs with browser-specific limits:
- Chromium/Firefox: 512MB limit
- Safari (WebKit): 2GB limit
These limits are far beyond typical card artwork sizes.

### Batch Save Images

Save multiple images to filesystem:

```typescript
// Save all images from ranges
await sheet.saveCellImagesAsync({
    useCellAddress: true,      // Use A1, B2 in filename
    useColumnIndex: 2          // Use column C values in filename
}, [range1, range2]);

// Single range
await range.saveCellImagesAsync(options);
```

## Vite Pre-Bundling Compatibility

**CRITICAL**: Univer's architecture is fragile under Vite's dependency
pre-bundling. Two issues have caused significant debugging time and must be
guarded against.

### Version Alignment

All `@univerjs/*` packages MUST be pinned to the exact same version in
package.json. Even a minor patch mismatch (e.g. core at 0.15.2 while drawing
packages are at 0.15.3) causes the dependency injection (DI) system
(`@wendellhu/redi`) to fail at runtime with opaque errors like:

    [redi]: Expect 1 dependency item(s) for id "z" but get 0.

This happens because a newer plugin may depend on services or DI tokens
introduced in the newer release that do not exist in the older core. The
errors are difficult to diagnose because redi uses minified identifiers in
its error messages, giving no indication of which service is missing.

When upgrading, always update every `@univerjs/*` dependency together to the
same version. Pin exact versions (e.g. `"0.15.3"`) rather than using caret
ranges (`"^0.15.3"`) to prevent the package manager from resolving different
patch versions for different packages.

### Facade Mixin Duplication

Univer's Facade API uses a class-extension mixin pattern:

```typescript
// Inside @univerjs/sheets-drawing-ui/facade:
FWorksheet.extend(DrawingMixin);
```

This call adds methods like `newOverGridImage()` and `insertImages()` onto the
`FWorksheet` prototype. However, Vite's pre-bundling can create separate
JavaScript module copies of the same class. When this happens, the `.extend()`
call modifies one copy of `FWorksheet` while the runtime uses a different copy,
resulting in `newOverGridImage is not a function` errors.

To mitigate this, `vite.config.ts` lists all drawing-related Univer packages in
`optimizeDeps.include`, which encourages Vite to share code across pre-bundled
chunks. However, this is not a complete fix — Vite may still create separate
chunks that duplicate class prototypes.

### Command Bypass Workaround

Because of the facade mixin issue, `ImageCellRenderer` bypasses the FWorksheet
facade entirely and uses Univer's command system directly:

1. It imports `InsertSheetDrawingCommand` from `@univerjs/sheets-drawing-ui`
2. It gets `ICommandService` from the injector
3. If the drawing plugin's `onRendered()` lifecycle has not yet registered the
   command, it manually registers `InsertSheetDrawingCommand`
4. It calls `univerAPI.executeCommand("sheet.command.insert-sheet-image", ...)`
   with manually constructed image data

This approach also requires computing image positions via internal render
services (`IRenderManagerService`, `ISheetSelectionRenderService`,
`SheetSkeletonManagerService`) and the `convertPositionCellToSheetOverGrid()`
utility from `@univerjs/sheets-ui`.

### Drawing Command Timing

The `UniverSheetsDrawingUIPlugin` registers its commands (including
`sheet.command.insert-sheet-image`) during the `onRendered()` lifecycle phase,
which fires only after the first render frame completes. Events from the Tauri
backend (e.g. derived value results containing image paths) can arrive before
this phase. The `ImageCellRenderer.ensureCommandsRegistered()` method handles
this by checking `hasCommand()` and manually registering the command if needed.

### Summary of Pitfalls

| Problem | Symptom | Fix |
|---------|---------|-----|
| Version mismatch across `@univerjs/*` packages | `[redi]: Expect N dependency item(s) for id "x" but get 0` | Pin all packages to the same exact version |
| Vite duplicates FWorksheet prototype | `newOverGridImage is not a function` | Bypass facade; use `executeCommand()` directly |
| Drawing commands not registered yet | `[CommandService]: command "..." is not registered` | Call `ensureCommandsRegistered()` before use |

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
See the detailed "Image Rendering Support" section above for comprehensive
documentation. Key points for TV:

- Use floating images (FWorksheet.insertImage) positioned at cell locations
- For cached local images, use Tauri's convertFileSrc() to create asset URLs
- For remote images, cache on backend then serve via asset protocol
- Use the builder pattern (newOverGridImage) for precise size/position control

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
