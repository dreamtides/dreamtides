# Post-Mortem: Show Delete Button Feature (Failed)

## Goal

Add a "View > Show Delete Button" menu toggle to TV. When enabled, a red
delete button ("✕") should appear as the leftmost column in each row,
before frozen columns, allowing one-click row deletion.

## Approach Taken

Used Univer's column system: reserve visual column 0 for a narrow red "✕"
marker. Clicking a cell in column 0 would trigger the existing `onDeleteRow`
flow (sort-index translation -> TOML delete -> sheet reload).

### Changes Made (all reverted)

**Rust backend:**
- Added `delete_button_visible` to `ViewState` struct
- Added `show_delete_button` `CheckMenuItem` to View menu
- Added menu event handler emitting `"delete-button-visibility-changed"`

**TypeScript frontend:**
- Added `onDeleteButtonVisibilityChanged` event listener in `ipc_bridge.ts`
- Added `deleteButtonVisible` state in `app_root.tsx` with ViewState persistence
- Added `showDeleteButton` prop threaded through `SpreadsheetView` to `UniverSpreadsheet`
- Modified `buildColumnMapping` in `derived_column_utils.ts` to reserve position 0
- Modified `buildMultiSheetWorkbook` in `workbook_builder.ts` to populate column 0 cells
- Added selection detection in `UniverSpreadsheet.tsx` for column 0 clicks
- Added `key` prop to force remount on toggle

## Failures

### Failure 1: Menu event never reached frontend

The `show_delete_button` Rust handler used the `disable_autosave` pattern:
```rust
if let Some(window) = app_handle.get_webview_window("main") {
    if let Some(menu) = window.menu() {
        if let Some(MenuItemKind::Check(check_item)) = menu.get("show_delete_button") {
            // emit event with checked state
        }
    }
}
```

**Root cause:** `menu.get("show_delete_button")` may not recursively search
into submenus. The menu item was inside the "View" submenu, so the lookup
returned `None` and the emit was silently skipped. The checkmark toggled
(handled by Tauri automatically) but no event was emitted.

**Attempted fix:** Switched to the `show_statistics` pattern (emit a void
event directly on `app_handle` without looking up menu state). This made the
event reach the frontend, but introduced Failure 2.

### Failure 2: Column 0 conflict with derived image column

`art-assigned.toml` has a derived image column configured at position 0.
When `showDeleteButton` reserved position 0 for the delete column, both
competed for the same visual column. The image cell renderer tried to
insert images at column 0 (now the delete button column), causing hundreds
of errors and app crashes.

**Attempted fix:** Added a `deleteColumnOffset` system that shifts all
derived column positions by +1 when `showDeleteButton` is true. This
required changes to:
- `buildColumnMapping` (shift reserved positions)
- `getDerivedColumnIndex` (shift return values)
- `buildMultiSheetWorkbook` (shift header placement, width placement, frozen column calc)
- All 4 `getDerivedColumnIndex` call sites in `UniverSpreadsheet.tsx`

This fix was never tested because the user halted the attempt.

### Failure 3: Workbook not rebuilding on toggle

Changing `showDeleteButton` prop didn't trigger a workbook rebuild because
the Univer workbook is created once in an initialization effect and only
updates when `multiSheetData` changes.

**Attempted fix:** Added a `key` prop to `<UniverSpreadsheet>` that changes
with `showDeleteButton`, forcing a full React unmount/remount. This is a
heavy-handed approach that destroys and recreates the entire Univer instance.

## Why This Approach Was Fundamentally Flawed

1. **Column mapping is deeply wired:** The column mapping system
   (`buildColumnMapping`, `getDerivedColumnIndex`, `extractDataFromSheet`,
   filter resources, column width persistence, sort index translation) is
   designed for a static layout determined at load time. Inserting a dynamic
   column 0 that shifts everything requires touching every layer.

2. **Derived columns use absolute positions:** Derived column configs
   specify absolute visual positions (e.g., `position: 0`). There's no
   indirection — the position IS the column index. Adding an offset breaks
   the fundamental contract and requires updating every consumer.

3. **No dynamic column insertion in Univer:** Univer workbooks don't
   support dynamically adding/removing columns without a full rebuild.
   The `key` remount approach is expensive and loses all transient state
   (scroll position, selection, unsaved edits, image cache).

4. **Too many integration points:** The feature touched 10 files across
   Rust backend and TypeScript frontend. Each integration point (menu event,
   view state, column mapping, workbook builder, click detection, edit
   prevention, image renderer) had its own failure mode.

## Recommendations for Retry

### Option A: Column-based with embedded offset (recommended)

Retry the column-based approach with these fixes for each failure:

**Failure 1 fix:** Use the `show_statistics` void event pattern instead
of the `disable_autosave` pattern. Emit directly on `app_handle` without
looking up the menu item via `menu.get()` (which doesn't search submenus).

**Failure 2 fix:** Embed the delete column offset inside the `ColumnMapping`
struct itself, rather than passing it as a separate parameter to every
consumer. This way `getDerivedColumnIndex` reads the offset from the
mapping automatically, and no call sites need to change:

```typescript
interface ColumnMapping {
  dataToVisual: number[];
  visualToData: Map<number, number>;
  reservedPositions: Set<number>;
  totalVisualColumns: number;
  deleteColumnOffset: number;  // 0 or 1
}
```

Then `getDerivedColumnIndex` returns `config.position + mapping.deleteColumnOffset`
for positioned columns. The offset also applies to derived column header
placement, width placement, and frozen column calculations in
`buildMultiSheetWorkbook` — all of which can read `mapping.deleteColumnOffset`
from the mapping they already have.

**Failure 3 fix:** Key prop on `<UniverSpreadsheet>` to force remount.
Heavy but functional; no alternative since Univer doesn't support dynamic
column insertion.

### Option B: React overlay (NOT recommended)

Render delete buttons as a React component overlaid on the spreadsheet,
outside Univer entirely.

**Why not:** Univer uses canvas-based rendering with virtual scrolling.
There is no DOM scroll container to hook into. Syncing an overlay would
require listening to Univer's internal scroll events, which are not
exposed through the Facade API. This approach is harder than the column
approach, not easier.

### Option C: Context menu enhancement

Instead of a visible column, enhance the existing right-click delete with
a more prominent affordance (e.g., row highlight on hover, or a toolbar
delete button that acts on the selected row).

**Pros:** Zero column layout changes.
**Cons:** Doesn't meet the "visible button per row" requirement.
