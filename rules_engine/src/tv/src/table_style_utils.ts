import { FWorksheet } from "@univerjs/sheets/facade";

import type {
  TomlTableData,
  ResolvedTableStyle,
  CellFormatResult,
} from "./ipc_bridge";
import type { ColumnMapping } from "./derived_column_utils";
import { hasUnderlineMethod } from "./spreadsheet_types";

/**
 * Applies a resolved table style (color scheme, row stripes, header styling)
 * to a sheet. Colors follow visual (display) row order, not data order,
 * so alternating stripes remain consistent after sorting.
 */
export function applyTableStyle(
  sheet: FWorksheet,
  data: TomlTableData,
  style: ResolvedTableStyle,
  mapping: ColumnMapping,
): void {
  const numColumns = mapping.totalVisualColumns;
  if (numColumns === 0) return;

  const headerBg = style.header_background ?? style.palette?.header_background;
  const headerFontColor = style.palette?.header_font_color;

  // Apply header row styling
  const headerRange = sheet.getRange(0, 0, 1, numColumns);
  if (headerRange) {
    if (style.header_bold) {
      headerRange.setFontWeight("bold");
    }
    if (headerBg) {
      headerRange.setBackgroundColor(headerBg);
    }
    if (headerFontColor) {
      headerRange.setFontColor(headerFontColor);
    }
  }

  // Apply alternating row stripe colors and row font color
  if (style.show_row_stripes && style.palette && data.rows.length > 0) {
    const evenBg = style.palette.row_even_background;
    const oddBg = style.palette.row_odd_background;
    const rowFontColor = style.palette.row_font_color;

    for (let displayRow = 0; displayRow < data.rows.length; displayRow++) {
      const cellRow = displayRow + 1;
      const rowRange = sheet.getRange(cellRow, 0, 1, numColumns);
      if (rowRange) {
        rowRange.setBackgroundColor(displayRow % 2 === 0 ? evenBg : oddBg);
        rowRange.setFontColor(rowFontColor);
      }
    }
  }

  // Apply alternating column stripe colors
  if (style.show_column_stripes && style.palette && data.rows.length > 0) {
    const accentBg = style.palette.accent_color;

    for (let colIdx = 0; colIdx < numColumns; colIdx++) {
      if (colIdx % 2 === 1) {
        const colRange = sheet.getRange(1, colIdx, data.rows.length, 1);
        if (colRange) {
          colRange.setBackgroundColor(accentBg + "33");
        }
      }
    }
  }
}

/**
 * Applies conditional formatting results to cells in a sheet.
 * Each result specifies a row, column index, and style to apply.
 * Conditional formatting runs after table color scheme application
 * so that conditional styles override base table styling.
 */
export function applyConditionalFormatting(
  sheet: FWorksheet,
  results: CellFormatResult[],
  mapping: ColumnMapping,
): void {
  for (const result of results) {
    const cellRow = result.row + 1; // +1 for header row
    const visualCol =
      result.col_index < mapping.dataToVisual.length
        ? mapping.dataToVisual[result.col_index]
        : result.col_index;
    const range = sheet.getRange(cellRow, visualCol, 1, 1);
    if (!range) continue;

    if (result.style.background_color) {
      range.setBackgroundColor(result.style.background_color);
    }
    if (result.style.font_color) {
      range.setFontColor(result.style.font_color);
    }
    if (result.style.bold === true) {
      range.setFontWeight("bold");
    }
    if (result.style.italic === true) {
      range.setFontStyle("italic");
    }
    if (result.style.underline === true && hasUnderlineMethod(range)) {
      range.setUnderline(true);
    }
  }
}
