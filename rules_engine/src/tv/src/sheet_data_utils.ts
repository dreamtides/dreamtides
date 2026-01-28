import { FWorksheet } from "@univerjs/sheets/facade";

import type { TomlTableData } from "./ipc_bridge";
import { formatHeaderForDisplay } from "./header_utils";
import {
  getContiguousSegments,
  type ColumnMapping,
} from "./derived_column_utils";

/**
 * Build an IDocumentData object from a plain string, converting \n to
 * paragraph breaks so Univer renders line breaks in the cell.
 */
export function stringToDocumentData(
  text: string,
  bold?: boolean,
): Record<string, unknown> {
  let dataStream = "";
  const textRuns: { st: number; ed: number; ts: Record<string, unknown> }[] =
    [];
  const paragraphs: { startIndex: number }[] = [];

  const parts = text.split("\n");
  for (let i = 0; i < parts.length; i++) {
    if (i > 0) {
      dataStream += "\r";
      paragraphs.push({ startIndex: dataStream.length - 1 });
    }
    if (parts[i].length > 0) {
      const start = dataStream.length;
      dataStream += parts[i];
      if (bold) {
        textRuns.push({ st: start, ed: dataStream.length, ts: { bl: 1 } });
      }
    }
  }

  dataStream += "\r\n";
  paragraphs.push({ startIndex: dataStream.length - 2 });

  return {
    id: "",
    body: {
      dataStream,
      paragraphs,
      textRuns,
      sectionBreaks: [],
      customBlocks: [],
      customRanges: [],
      tables: [],
    },
    documentStyle: {},
  };
}

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

    for (let rowIndex = 0; rowIndex < data.rows.length; rowIndex++) {
      const row = data.rows[rowIndex];
      for (let colIndex = 0; colIndex < row.length; colIndex++) {
        const cellValue = row[colIndex];
        if (typeof cellValue === "string" && cellValue.includes("\n")) {
          const visualCol = mapping.dataToVisual[colIndex];
          const range = sheet.getRange(rowIndex + 1, visualCol, 1, 1);
          if (range) {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            range.setRichTextValueForCell(stringToDocumentData(cellValue) as any);
          }
        }
      }
    }
  }
}
