import { FWorksheet } from "@univerjs/sheets/facade";

import type { TomlTableData } from "./ipc_bridge";
import { formatHeaderForDisplay } from "./header_utils";
import {
  getContiguousSegments,
  type ColumnMapping,
} from "./derived_column_utils";

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
 * Writes data in contiguous segments to handle non-contiguous column mappings.
 */
export function populateSheetDataBatch(
  sheet: FWorksheet,
  data: TomlTableData,
  mapping: ColumnMapping,
): void {
  if (!sheet) return;
  if (data.headers.length === 0) return;

  const segments = getContiguousSegments(mapping);

  for (const seg of segments) {
    const segHeaders = data.headers
      .slice(seg.dataStart, seg.dataStart + seg.length)
      .map(formatHeaderForDisplay);
    const headerRange = sheet.getRange(0, seg.visualStart, 1, seg.length);
    if (headerRange) {
      headerRange.setValues([segHeaders]);
      headerRange.setFontWeight("bold");
    }
  }

  if (data.rows.length > 0) {
    for (const seg of segments) {
      const dataRange = sheet.getRange(
        1,
        seg.visualStart,
        data.rows.length,
        seg.length,
      );
      if (dataRange) {
        const displayRows = data.rows.map((row) => {
          const segRow: (string | number | boolean)[] = [];
          for (let c = seg.dataStart; c < seg.dataStart + seg.length; c++) {
            const val = c < row.length ? row[c] : null;
            if (val === null) segRow.push("");
            else if (typeof val === "boolean") segRow.push(val ? 1 : 0);
            else segRow.push(val);
          }
          return segRow;
        });
        dataRange.setValues(displayRows);
      }
    }
  }
}
