---
name: univer
description: Build spreadsheet applications using Univer, a TypeScript spreadsheet library. Use for rendering spreadsheets, handling cell data, managing images/drawings, data validation, conditional formatting, and spreadsheet events.
allowed-tools: Read, Write, Edit, Bash, Glob, Grep
---

# Univer Spreadsheet Library

Build web-based spreadsheet applications using Univer's Facade API.

## Official Resources

- Documentation: https://docs.univer.ai
- Facade API Guide: https://docs.univer.ai/guides/sheets/getting-started/facade
- API Reference: https://reference.univer.ai/en-US
- GitHub: https://github.com/dream-num/univer

## Quick Start

```typescript
import { createUniver, defaultTheme, LocaleType } from '@univerjs/presets';
import { UniverSheetsCorePreset } from '@univerjs/presets/preset-sheets-core';
import '@univerjs/presets/lib/styles/preset-sheets-core.css';

const { univer, univerAPI } = createUniver({
  locale: LocaleType.EN_US,
  theme: defaultTheme,
  presets: [UniverSheetsCorePreset()],
});

// Create workbook
const workbook = univerAPI.createWorkbook({ name: 'My Workbook' });
```

## Core Concepts

### Getting References

```typescript
// Get active workbook and sheet
const fWorkbook = univerAPI.getActiveWorkbook();
const fWorksheet = fWorkbook.getActiveSheet();

// Get range by A1 notation or coordinates
const fRange = fWorksheet.getRange('A1:B10');
const fRange2 = fWorksheet.getRange(0, 0, 10, 2); // row, col, numRows, numCols
```

### Cell Values

```typescript
// Get/set single value
const value = fRange.getValue();
fRange.setValueForCell(42);

// Get/set multiple values (2D array)
const values = fRange.getValues();
fRange.setValues([
  ['A1', 'B1'],
  ['A2', 'B2'],
]);
```

### Cell Styling

```typescript
// Apply styles
fRange.setBackgroundColor('#FFFF00');
fRange.setFontWeight('bold');
fRange.setFontColor('#FF0000');
fRange.setFontSize(14);
fRange.setHorizontalAlignment('center');
fRange.setVerticalAlignment('middle');
fRange.setTextWrap(true);

// Get style
const style = fRange.getCellStyleData();
```

### Row and Column Operations

```typescript
// Insert/delete rows
fWorksheet.insertRowBefore(5);
fWorksheet.insertRowAfter(5);
fWorksheet.deleteRow(5);

// Row height / column width
fWorksheet.setRowHeight(5, 100); // row 5, height 100px
fWorksheet.setColumnWidth(0, 150); // column A, width 150px

// Hide/show
fWorksheet.hideRow(5);
fWorksheet.showRow(5);
```

### Sheet Operations

```typescript
// Create/delete sheets
const newSheet = fWorkbook.insertSheet();
fWorkbook.deleteSheet(sheet.getSheetId());

// Rename/activate
fWorksheet.setName('New Name');
fWorkbook.setActiveSheet(fWorksheet);

// Get sheet info
const sheetId = fWorksheet.getSheetId();
const sheetName = fWorksheet.getSheetName();
```

## Events

```typescript
// Value changed
univerAPI.addEvent(univerAPI.Event.SheetValueChanged, (params) => {
  const { effectedRanges, payload } = params;
  console.log('Values changed:', effectedRanges);
});

// Active sheet changed
univerAPI.addEvent(univerAPI.Event.ActiveSheetChanged, (params) => {
  const { workbook, activeSheet } = params;
});

// Before sheet create (cancellable)
univerAPI.addEvent(univerAPI.Event.BeforeSheetCreate, (params) => {
  params.cancel = true; // Cancel creation
});

// Cleanup listener
const disposable = univerAPI.addEvent(...);
disposable.dispose();
```

### Available Events
- `SheetValueChanged` - Cell values changed
- `SheetCreated` / `BeforeSheetCreate` - Sheet creation
- `SheetDeleted` / `BeforeSheetDelete` - Sheet deletion
- `ActiveSheetChanged` / `BeforeActiveSheetChange` - Sheet activation
- `SheetMoved` / `BeforeSheetMove` - Sheet reordering
- `SheetNameChanged` / `BeforeSheetNameChange` - Sheet rename
- `WorkbookCreated` / `WorkbookDisposed` - Workbook lifecycle

## Images / Drawings

### Required Packages
```bash
pnpm add @univerjs/drawing @univerjs/drawing-ui @univerjs/sheets-drawing @univerjs/sheets-drawing-ui
# Or use preset: @univerjs/preset-sheets-drawing
```

### Import Facade Extension
```typescript
import '@univerjs/sheets-drawing-ui/facade';
```

### Insert Floating Images

```typescript
// Simple insertion at cell (column, row)
await fWorksheet.insertImage(url, 5, 5);

// With offset
await fWorksheet.insertImage(url, 5, 5, 10, 10); // offsetX, offsetY

// Builder pattern (recommended for control)
const image = await fWorksheet.newOverGridImage()
  .setSource(url, univerAPI.Enum.ImageSourceType.URL)
  .setColumn(5)
  .setRow(5)
  .setWidth(200)
  .setHeight(150)
  .setColumnOffset(10)
  .setRowOffset(10)
  .setRotate(0)
  .setAnchorType(univerAPI.Enum.SheetDrawingAnchorType.Position)
  .buildAsync();

fWorksheet.insertImages([image]);
```

### Image Sources
```typescript
// URL (HTTP/HTTPS)
.setSource('https://example.com/image.png', univerAPI.Enum.ImageSourceType.URL)

// Base64 data URI
.setSource('data:image/png;base64,iVBORw0...', univerAPI.Enum.ImageSourceType.BASE64)
```

### Tauri Asset Protocol (for local files)
```typescript
import { convertFileSrc } from '@tauri-apps/api/core';

const localPath = '/path/to/cached/image.png';
const assetUrl = convertFileSrc(localPath);
// Returns: http://asset.localhost/path/to/cached/image.png

await fWorksheet.insertImage(assetUrl, column, row);
```

Required tauri.conf.json:
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

### Manage Images

```typescript
// Get all images
const images = fWorksheet.getImages();

// Get by ID
const image = fWorksheet.getImageById('drawingId');

// Update image
const updatedImage = await image.toBuilder()
  .setWidth(100)
  .setHeight(50)
  .buildAsync();
fWorksheet.updateImages([updatedImage]);

// Delete
image.remove();
// Or: fWorksheet.deleteImages([image]);

// Layer ordering
image.setFront();
image.setBack();
image.setForward();
image.setBackward();
```

### Image Events
```typescript
univerAPI.addEvent(univerAPI.Event.OverGridImageInserted, (params) => {
  const { workbook, images } = params;
});

univerAPI.addEvent(univerAPI.Event.BeforeOverGridImageInsert, (params) => {
  params.cancel = true; // Cancel insertion
});
```

## Data Validation

### Create Validation Rules

```typescript
// Dropdown list from values
const rule = univerAPI.newDataValidation()
  .requireValueInList(['Yes', 'No', 'Maybe'])
  .setOptions({
    allowBlank: true,
    showErrorMessage: true,
    error: 'Please select a valid option'
  })
  .build();

fRange.setDataValidation(rule);

// Dropdown from cell range
const sourceRange = fWorksheet.getRange('Z1:Z10');
const rule = univerAPI.newDataValidation()
  .requireValueInRange(sourceRange)
  .build();

// Checkbox
const rule = univerAPI.newDataValidation()
  .requireCheckbox('Yes', 'No')
  .build();

// Number validation
const rule = univerAPI.newDataValidation()
  .requireNumberBetween(1, 100)
  .build();

// Date validation
const rule = univerAPI.newDataValidation()
  .requireDateBetween(new Date('2024-01-01'), new Date('2024-12-31'))
  .build();

// Custom formula
const rule = univerAPI.newDataValidation()
  .requireFormulaSatisfied('=A1>B1')
  .build();
```

### Validation Methods
- `requireValueInList(values[], multiple?, showDropdown?)` - Dropdown
- `requireValueInRange(fRange, multiple?, showDropdown?)` - Range reference
- `requireCheckbox(checked?, unchecked?)` - Checkbox
- `requireNumberBetween(start, end, isInteger?)` - Number range
- `requireNumberGreaterThan(num, isInteger?)` - Minimum
- `requireNumberLessThan(num, isInteger?)` - Maximum
- `requireNumberEqualTo(num, isInteger?)` - Exact match
- `requireDateBetween(start, end)` - Date range
- `requireDateAfter(date)` / `requireDateBefore(date)`
- `requireFormulaSatisfied(formula)` - Custom formula

### Get Validation Status

```typescript
const status = await fRange.getValidatorStatus();
// Returns 2D array: [['valid', 'invalid'], ['valid', 'valid']]
```

## Conditional Formatting

```typescript
// Create rule builder
const rule = fWorksheet.newConditionalFormattingRule()
  .whenCellEmpty()
  .setBackground('#FF0000')
  .setRanges([fRange.getRange()])
  .build();

fWorksheet.addConditionalFormattingRule(rule);

// Available conditions
.whenCellEmpty()
.whenCellNotEmpty()
.whenTextContains('text')
.whenTextStartsWith('prefix')
.whenNumberGreaterThan(10)
.whenNumberBetween(1, 100)
.whenDateBefore(new Date())
.setAverage(operator)  // Above/below average

// Style methods
.setBackground('#FF0000')
.setFontColor('#FFFFFF')
.setBold(true)
.setItalic(true)
```

## Freezing Panes

```typescript
// Freeze first row and column
fWorksheet.setFreeze({ row: 1, column: 1 });

// Cancel freeze
fWorksheet.cancelFreeze();
```

## Merge Cells

```typescript
// Merge range
fRange.merge();

// Unmerge
fRange.unmerge();

// Check if merged
const isMerged = fRange.isMerged();
```

## Selection

```typescript
// Get current selection
const selection = fWorksheet.getSelection();
const activeCell = selection.getActiveCell();
const activeRange = selection.getActiveRange();

// Set selection programmatically
fWorksheet.setActiveRange(fRange);
```

## Workbook Data Structure

```typescript
// IWorkbookData structure
const workbookData = {
  id: 'workbook-id',
  name: 'My Workbook',
  sheets: {
    'sheet-id': {
      id: 'sheet-id',
      name: 'Sheet1',
      rowCount: 1000,
      columnCount: 26,
      cellData: {
        0: { // row 0
          0: { v: 'A1 value', s: 'style-id' }, // column 0
          1: { v: 'B1 value' }
        }
      }
    }
  },
  sheetOrder: ['sheet-id']
};

// ICellData structure
const cellData = {
  v: 'cell value',     // string | number | boolean
  s: 'style-id',       // style reference or inline IStyleData
  t: 1,                // cell type (string=1, number=2, boolean=3)
  p: { /* rich text */ }
};

// Create workbook with data
univerAPI.createWorkbook(workbookData);
```

## Enums

```typescript
// Access via univerAPI.Enum
univerAPI.Enum.ImageSourceType.URL
univerAPI.Enum.ImageSourceType.BASE64
univerAPI.Enum.SheetDrawingAnchorType.Position  // Move with cells
univerAPI.Enum.SheetDrawingAnchorType.Both      // Move and resize
univerAPI.Enum.SheetDrawingAnchorType.None      // Fixed position
univerAPI.Enum.DataValidationType.LIST
univerAPI.Enum.DataValidationType.CHECKBOX
univerAPI.Enum.DataValidationType.DECIMAL
```

## React Integration

```typescript
import { useEffect, useRef } from 'react';

function Spreadsheet() {
  const containerRef = useRef<HTMLDivElement>(null);
  const univerRef = useRef<{ univer: any; univerAPI: any } | null>(null);

  useEffect(() => {
    if (containerRef.current && !univerRef.current) {
      const { univer, univerAPI } = createUniver({
        locale: LocaleType.EN_US,
        theme: defaultTheme,
        presets: [UniverSheetsCorePreset({ container: containerRef.current })],
      });
      univerRef.current = { univer, univerAPI };
      univerAPI.createWorkbook({});
    }

    return () => {
      univerRef.current?.univer.dispose();
      univerRef.current = null;
    };
  }, []);

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
}
```

## Cleanup

```typescript
// Dispose event listener
const disposable = univerAPI.addEvent(...);
disposable.dispose();

// Dispose entire Univer instance
univer.dispose();
```

## Plugin Configuration

Required plugins loaded in order:
1. UniverRenderEnginePlugin - Canvas rendering
2. UniverFormulaEnginePlugin - Formula engine
3. UniverUIPlugin - UI framework
4. UniverDocsPlugin - Text content
5. UniverSheetsPlugin - Core spreadsheet
6. UniverSheetsUIPlugin - Spreadsheet UI
7. UniverSheetsFormulaPlugin - Formula support
8. UniverSheetsFilterPlugin - Filtering
9. UniverSheetsSortPlugin - Sorting
10. UniverDataValidationPlugin - Validation
11. UniverSheetsConditionalFormattingPlugin - Conditional formatting
12. UniverSheetsDrawingPlugin - Images/drawings

Or use presets for simplified setup.

## Documentation Source

Univer source code available at: `~/Desktop/tmp/univer/`
- packages/core/src/facade/ - Core Facade API
- packages/sheets/src/facade/ - Sheets Facade API
- packages/sheets-drawing-ui/src/facade/ - Image/drawing APIs
- packages/sheets-data-validation/src/facade/ - Data validation
- packages/sheets-conditional-formatting/src/facade/ - Conditional formatting
