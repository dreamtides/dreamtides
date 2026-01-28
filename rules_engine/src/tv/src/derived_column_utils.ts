import { FUniver } from "@univerjs/core/facade";
import { FWorksheet } from "@univerjs/sheets/facade";

import type {
  DerivedColumnInfo,
  DerivedResultValue,
  TextStyle,
} from "./ipc_bridge";
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
  univerAPI: FUniver,
  sheet: FWorksheet,
  row: number,
  col: number,
  result: DerivedResultValue,
): void {
  const range = sheet.getRange(row, col, 1, 1);
  if (!range) return;

  if (result.type === "richText") {
    // Build rich text using insertText (unstyled) + setStyle to work around
    // a Univer RichTextBuilder bug where insertText(text, style) calculates
    // textRun st/ed indices using the document-absolute insertion position
    // instead of fragment-relative indices, causing style offsets to shift
    // by the length of previously inserted text.
    const allRuns = result.value.p.flatMap((p) => p.ts);
    const plainText = allRuns.map((r) => r.t).join("");
    const richText = univerAPI.newRichText();
    richText.insertText(plainText);
    let offset = 0;
    for (const run of allRuns) {
      if (run.s) {
        const ts = toUniverTextStyle(run.s);
        if (ts) {
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          richText.setStyle(offset, offset + run.t.length, ts as any);
        }
      }
      offset += run.t.length;
    }
    range.setRichTextValueForCell(richText);
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
    const cellData = derivedResultToCellData(result);
    const value = cellData.v !== undefined ? cellData.v : "";
    range.setValues([[value]]);
    // Reset font color to default for non-error results
    range.setFontColor("#000000");
    logger.debug("Applied derived result", { row, col, type: result.type });
  }
}

function toUniverTextStyle(
  style: TextStyle,
): Record<string, unknown> | undefined {
  if (!style.bl && !style.it && !style.ul && !style.cl) return undefined;
  const result: Record<string, unknown> = {};
  if (style.bl) result.bl = style.bl;
  if (style.it) result.it = style.it;
  if (style.ul) result.ul = { s: style.ul.s };
  if (style.cl) result.cl = { rgb: `#${style.cl.rgb}` };
  return result;
}
