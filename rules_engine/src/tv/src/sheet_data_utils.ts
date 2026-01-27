import { FWorksheet } from "@univerjs/sheets/facade";

import type { TomlTableData } from "./ipc_bridge";
import { formatHeaderForDisplay } from "./header_utils";

/**
 * Compare two TomlTableData objects for equality.
 */
export function isSheetDataEqual(a: TomlTableData, b: TomlTableData): boolean {
  // Compare headers
  if (a.headers.length !== b.headers.length) return false;
  for (let i = 0; i < a.headers.length; i++) {
    if (a.headers[i] !== b.headers[i]) return false;
  }

  // Compare rows
  if (a.rows.length !== b.rows.length) return false;
  for (let rowIdx = 0; rowIdx < a.rows.length; rowIdx++) {
    const rowA = a.rows[rowIdx];
    const rowB = b.rows[rowIdx];
    if (rowA.length !== rowB.length) return false;
    for (let colIdx = 0; colIdx < rowA.length; colIdx++) {
      if (rowA[colIdx] !== rowB[colIdx]) return false;
    }
  }

  return true;
}

/**
 * Populate a sheet with TomlTableData using batch operations.
 * Uses setValues() to set all cells in a single API call per region.
 */
export function populateSheetDataBatch(
  sheet: FWorksheet,
  data: TomlTableData,
  dataOffset: number = 0,
): void {
  if (!sheet) return;

  const numColumns = data.headers.length;
  if (numColumns === 0) return;

  // Set headers row using batch operation with display-formatted names
  const headerRange = sheet.getRange(0, dataOffset, 1, numColumns);
  if (headerRange) {
    headerRange.setValues([data.headers.map(formatHeaderForDisplay)]);
    headerRange.setFontWeight("bold");
  }

  // Set data rows using a single batch operation
  if (data.rows.length > 0) {
    const dataRange = sheet.getRange(
      1,
      dataOffset,
      data.rows.length,
      numColumns,
    );
    if (dataRange) {
      // Convert null values to empty strings and booleans to 1/0 for display
      const displayRows = data.rows.map((row) =>
        row.map((cellValue) => {
          if (cellValue === null) return "";
          if (typeof cellValue === "boolean") return cellValue ? 1 : 0;
          return cellValue;
        }),
      );
      dataRange.setValues(displayRows);
    }
  }
}
