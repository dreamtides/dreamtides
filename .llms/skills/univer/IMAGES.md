# Univer Image API Reference

Comprehensive reference for working with images in Univer spreadsheets.

## Setup

### Required Packages
```bash
pnpm add @univerjs/drawing @univerjs/drawing-ui @univerjs/sheets-drawing @univerjs/sheets-drawing-ui
```

Or use the preset:
```bash
pnpm add @univerjs/preset-sheets-drawing
```

### Import Facade Extension
```typescript
// IMPORTANT: Must import to extend FWorksheet with image methods
import '@univerjs/sheets-drawing-ui/facade';
```

## Image Types

### Floating Images (Over-Grid Images)
- Positioned over cells with pixel-level control
- Can span multiple cells
- Support rotation, cropping, and transforms
- **Recommended for card artwork and dynamic images**

### Cell Images
- Embedded directly within individual cells
- No mixed text/image layout (cell is either text OR image)
- Simpler but less flexible

## FWorksheet Image Methods

### insertImage()
```typescript
// Insert from URL at cell position
const result = await sheet.insertImage(url, column, row);

// With pixel offset from cell origin
const result = await sheet.insertImage(url, column, row, offsetX, offsetY);

// Insert from IFBlobSource (File/Blob converted to base64 internally)
const result = await sheet.insertImage(blobSource, column, row);
```

### newOverGridImage() - Builder Pattern
```typescript
const image = await sheet.newOverGridImage()
  .setSource(url, univerAPI.Enum.ImageSourceType.URL)
  .setColumn(5)       // 0-indexed column
  .setRow(5)          // 0-indexed row
  .setWidth(200)      // pixels
  .setHeight(150)     // pixels
  .setColumnOffset(10) // pixel offset from cell
  .setRowOffset(10)
  .setRotate(90)      // degrees
  .setAnchorType(univerAPI.Enum.SheetDrawingAnchorType.Position)
  .setCropTop(10)     // pixels to crop
  .setCropLeft(10)
  .setCropBottom(10)
  .setCropRight(10)
  .buildAsync();

sheet.insertImages([image]);
```

### Managing Images
```typescript
// Get all images on sheet
const images: FOverGridImage[] = sheet.getImages();

// Get specific image by ID
const image = sheet.getImageById('drawingId');

// Get currently selected images
const activeImages = sheet.getActiveImages();

// Update images
sheet.updateImages([updatedImage]);

// Delete images
sheet.deleteImages([image]);
```

## FOverGridImageBuilder Methods

| Method | Description |
|--------|-------------|
| `setSource(source, sourceType?)` | Set image source URL or base64 |
| `setColumn(column)` | Set horizontal position (0-indexed) |
| `setRow(row)` | Set vertical position (0-indexed) |
| `setColumnOffset(offset)` | Pixel offset from cell left edge |
| `setRowOffset(offset)` | Pixel offset from cell top edge |
| `setWidth(width)` | Image width in pixels |
| `setHeight(height)` | Image height in pixels |
| `setRotate(angle)` | Rotation in degrees |
| `setAnchorType(type)` | Position behavior with cell changes |
| `setCropTop/Left/Right/Bottom(px)` | Crop edges |
| `buildAsync()` | Build the ISheetImage object |

## FOverGridImage Methods

| Method | Description |
|--------|-------------|
| `getId()` | Get image drawing ID |
| `getType()` | Get drawing type enum |
| `remove()` | Delete this image |
| `toBuilder()` | Get builder for modifications |
| `setSource(source, type?)` | Change image source |
| `setPositionAsync(row, col, rowOffset?, colOffset?)` | Move image |
| `setSizeAsync(width, height)` | Resize image |
| `setCrop(top?, left?, bottom?, right?)` | Set crop region |
| `setRotate(angle)` | Set rotation |
| `setForward()` | Move layer up one |
| `setBackward()` | Move layer down one |
| `setFront()` | Move to top layer |
| `setBack()` | Move to bottom layer |

## Image Source Types

### ImageSourceType Enum
```typescript
// Access via univerAPI.Enum.ImageSourceType
enum ImageSourceType {
  URL = 'URL',      // HTTP/HTTPS URL
  UUID = 'UUID',    // Univer cloud service (not for local use)
  BASE64 = 'BASE64' // Data URI: data:image/png;base64,...
}
```

### Supported Sources

**URL Strings (ImageSourceType.URL)**
```typescript
.setSource('https://example.com/image.png', univerAPI.Enum.ImageSourceType.URL)
```
- HTTP/HTTPS URLs
- Subject to CORS restrictions
- Loaded asynchronously by browser

**Base64 Data URIs (ImageSourceType.BASE64)**
```typescript
.setSource('data:image/png;base64,iVBORw0KGgo...', univerAPI.Enum.ImageSourceType.BASE64)
```
- No CORS restrictions
- ~33% larger than binary
- Good for small images or when CORS is problematic

**IFBlobSource Objects**
```typescript
// From file input
const file = event.target.files[0];
await sheet.insertImage(file, column, row);
```
- Wrapper for File/Blob types
- Converted to base64 internally by Univer

### NOT Supported
- `file://` URLs are NOT directly supported
- Use Tauri's asset protocol for local files

## Tauri Asset Protocol Integration

For local filesystem images in Tauri apps:

### tauri.conf.json Configuration
```json
{
  "app": {
    "security": {
      "csp": "default-src 'self' ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost",
      "assetProtocol": {
        "enable": true,
        "scope": {
          "requireLiteralLeadingDot": false,
          "allow": ["$APPDATA/**/*", "$CACHE/**/*", "/path/to/images/**/*"]
        }
      }
    }
  }
}
```

### Usage
```typescript
import { convertFileSrc } from '@tauri-apps/api/core';

// Convert local path to asset URL
const localPath = '/Users/name/cache/image.png';
const assetUrl = convertFileSrc(localPath);
// Returns: http://asset.localhost/Users/name/cache/image.png

// Pass to Univer (recognized as regular URL)
await sheet.insertImage(assetUrl, column, row);
```

## Anchor Types

Control how images behave when rows/columns change:

```typescript
// univerAPI.Enum.SheetDrawingAnchorType
enum SheetDrawingAnchorType {
  Position,  // Move with cells, size unchanged
  Both,      // Move AND resize with cells
  None       // Fixed position, ignores cell changes
}
```

### Example
```typescript
// Image moves but doesn't resize when rows inserted
const image = await sheet.newOverGridImage()
  .setSource(url)
  .setColumn(0)
  .setRow(5)
  .setAnchorType(univerAPI.Enum.SheetDrawingAnchorType.Position)
  .buildAsync();
```

## Image Events

### Subscribe to Events
```typescript
// After insertion
univerAPI.addEvent(univerAPI.Event.OverGridImageInserted, (params) => {
  const { workbook, images } = params;
  console.log('Images inserted:', images);
});

// Before insertion (cancellable)
univerAPI.addEvent(univerAPI.Event.BeforeOverGridImageInsert, (params) => {
  const { workbook, insertImageParams } = params;
  params.cancel = true; // Cancel insertion
});

// After update
univerAPI.addEvent(univerAPI.Event.OverGridImageChanged, (params) => {
  const { workbook, images } = params;
});

// After deletion
univerAPI.addEvent(univerAPI.Event.OverGridImageRemoved, (params) => {
  const { workbook, images } = params;
});
```

### Available Image Events
| Event | Description |
|-------|-------------|
| `BeforeOverGridImageInsert` | Before image insertion (cancellable) |
| `OverGridImageInserted` | After image inserted |
| `BeforeOverGridImageChange` | Before image update (cancellable) |
| `OverGridImageChanged` | After image updated |
| `BeforeOverGridImageRemove` | Before image deletion (cancellable) |
| `OverGridImageRemoved` | After image deleted |

## Cell Images (FRange)

For embedding images directly in cells:

```typescript
const range = sheet.getRange('A10');

// Insert from URL
const result = await range.insertCellImageAsync(url);

// Insert from File object
const result = await range.insertCellImageAsync(file);

// Save cell images to filesystem
await range.saveCellImagesAsync({
  useCellAddress: true,     // Use A1, B2 in filename
  useColumnIndex: 2         // Use column C values in filename
});
```

**Limitations:**
- No mixed text/image in same cell
- Cell contains either text OR image, not both

## Performance Considerations

### Use Asset Protocol Over Base64
- Base64 has ~33% size overhead
- Asset protocol loads directly from filesystem
- More efficient for large images

### Lazy Loading
- Univer virtualizes rendering
- Only visible images are loaded
- Scroll triggers loading of new images

### Caching Strategy
For remote images:
1. Fetch and cache locally
2. Serve via Tauri asset protocol
3. Use cache hash as filename for deduplication

## Error Handling

```typescript
try {
  const image = await sheet.newOverGridImage()
    .setSource(url)
    .setColumn(5)
    .setRow(5)
    .buildAsync();
  sheet.insertImages([image]);
} catch (error) {
  // Handle errors:
  // - Network timeout
  // - Invalid image format
  // - Render unit not found
  console.error('Image insertion failed:', error);
}
```
