import { FWorksheet } from "@univerjs/sheets/facade";

import type { DerivedColumnInfo, DerivedResultValue } from "./ipc_bridge";
import { derivedResultToCellData } from "./rich_text_utils";
import { createLogger } from "./logger_frontend";

const logger = createLogger("tv.ui.derived");

/**
 * Calculates the column index for a derived column.
 * If the config has an explicit position, uses that. Otherwise appends
 * after all data columns plus any prior derived columns without positions.
 */
export function getDerivedColumnIndex(
  config: DerivedColumnInfo,
  dataColumnCount: number,
  allConfigs: DerivedColumnInfo[],
  dataOffset: number = 0,
): number {
  if (config.position !== undefined && config.position !== null) {
    return config.position;
  }

  // Place after all data columns (including offset) plus prior derived columns without explicit position
  let offset = 0;
  for (const c of allConfigs) {
    if (c.name === config.name) break;
    if (c.position === undefined || c.position === null) {
      offset++;
    }
  }
  return dataOffset + dataColumnCount + offset;
}

/**
 * Computes the number of spreadsheet columns occupied by positioned derived
 * columns. Data columns are shifted right by this amount.
 */
export function computeDataColumnOffset(
  configs: DerivedColumnInfo[] | undefined,
): number {
  if (!configs) return 0;
  return configs.filter((c) => c.position !== undefined && c.position !== null)
    .length;
}

/**
 * Applies a derived result value to a specific cell in the spreadsheet.
 * Handles all DerivedResult variants: text, number, boolean, image,
 * richText, and error. Error results display with red font color.
 */
export function applyDerivedResultToCell(
  sheet: FWorksheet,
  row: number,
  col: number,
  result: DerivedResultValue,
): void {
  const range = sheet.getRange(row, col, 1, 1);
  if (!range) return;

  const cellData = derivedResultToCellData(result);

  if (result.type === "richText" && cellData.p) {
    // Rich text requires setting the paragraph structure
    // Use setValues with the plain text first, then apply formatting
    const plainText = cellData.p.flatMap((p) => p.ts.map((r) => r.t)).join("");
    range.setValues([[plainText]]);
    // Apply rich text styling via individual text runs
    for (const paragraph of cellData.p) {
      for (const run of paragraph.ts) {
        if (run.s) {
          if (run.s.bl) range.setFontWeight("bold");
          if (run.s.it) range.setFontStyle("italic");
          if (run.s.cl) range.setFontColor(run.s.cl.rgb);
        }
      }
    }
    logger.debug("Applied rich text derived result", { row, col });
  } else if (result.type === "error") {
    range.setValues([[`Error: ${result.value}`]]);
    range.setFontColor("#FF0000");
    logger.debug("Applied error derived result", {
      row,
      col,
      error: result.value,
    });
  } else if (result.type === "image") {
    // Image results are handled by ImageCellRenderer via the
    // derived-value-computed event listener, not via cell text.
    return;
  } else {
    const value = cellData.v !== undefined ? cellData.v : "";
    range.setValues([[value]]);
    // Reset font color to default for non-error results
    range.setFontColor("#000000");
    logger.debug("Applied derived result", { row, col, type: result.type });
  }
}
