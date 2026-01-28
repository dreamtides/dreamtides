import type { IDocumentData, ITextRun, IParagraph, ITextStyle } from "@univerjs/core";
import { FWorksheet } from "@univerjs/sheets/facade";

import type {
  DerivedColumnInfo,
  DerivedResultValue,
  TextRun,
  TextStyle,
} from "./ipc_bridge";
import { derivedResultToCellData } from "./rich_text_utils";
import { createLogger } from "./logger_frontend";

const logger = createLogger("tv.ui.derived");

export interface ColumnMapping {
  dataToVisual: number[];
  visualToData: Map<number, number>;
  reservedPositions: Set<number>;
  totalVisualColumns: number;
}

export interface ContiguousSegment {
  dataStart: number;
  visualStart: number;
  length: number;
}

/**
 * Builds a mapping between data column indices and visual column indices,
 * placing data columns at the first available positions not reserved by
 * positioned derived columns.
 */
export function buildColumnMapping(
  configs: DerivedColumnInfo[] | undefined,
  dataColumnCount: number,
): ColumnMapping {
  const reservedPositions = new Set<number>();
  if (configs) {
    for (const c of configs) {
      if (c.position !== undefined && c.position !== null) {
        reservedPositions.add(c.position);
      }
    }
  }

  const dataToVisual: number[] = [];
  let visual = 0;
  for (let dataIdx = 0; dataIdx < dataColumnCount; dataIdx++) {
    while (reservedPositions.has(visual)) visual++;
    dataToVisual.push(visual);
    visual++;
  }

  const visualToData = new Map<number, number>();
  for (let i = 0; i < dataToVisual.length; i++) {
    visualToData.set(dataToVisual[i], i);
  }

  let totalVisualColumns =
    dataToVisual.length > 0 ? dataToVisual[dataToVisual.length - 1] + 1 : 0;
  for (const pos of reservedPositions) {
    totalVisualColumns = Math.max(totalVisualColumns, pos + 1);
  }

  return { dataToVisual, visualToData, reservedPositions, totalVisualColumns };
}

/**
 * Splits the data-to-visual mapping into contiguous segments for efficient
 * batch read/write operations on the spreadsheet grid.
 */
export function getContiguousSegments(
  mapping: ColumnMapping,
): ContiguousSegment[] {
  if (mapping.dataToVisual.length === 0) return [];
  const segments: ContiguousSegment[] = [];
  let segStart = 0;
  for (let i = 1; i <= mapping.dataToVisual.length; i++) {
    if (
      i === mapping.dataToVisual.length ||
      mapping.dataToVisual[i] !== mapping.dataToVisual[i - 1] + 1
    ) {
      segments.push({
        dataStart: segStart,
        visualStart: mapping.dataToVisual[segStart],
        length: i - segStart,
      });
      segStart = i;
    }
  }
  return segments;
}

/** Empty mapping constant for fallback when no mapping is available. */
export const EMPTY_MAPPING: ColumnMapping = {
  dataToVisual: [],
  visualToData: new Map(),
  reservedPositions: new Set(),
  totalVisualColumns: 0,
};

/**
 * Calculates the column index for a derived column.
 * If the config has an explicit position, uses that. Otherwise appends
 * after all data columns plus any prior derived columns without positions.
 */
export function getDerivedColumnIndex(
  config: DerivedColumnInfo,
  allConfigs: DerivedColumnInfo[],
  mapping: ColumnMapping,
): number {
  if (config.position !== undefined && config.position !== null) {
    return config.position;
  }

  let offset = 0;
  for (const c of allConfigs) {
    if (c.name === config.name) break;
    if (c.position === undefined || c.position === null) {
      offset++;
    }
  }
  return mapping.totalVisualColumns + offset;
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

  if (result.type === "richText") {
    // Build IDocumentData manually rather than using RichTextBuilder,
    // because the builder's insertParagraph() produces incorrect paragraph
    // metadata when interleaved with insertText() calls.
    // In Univer's document model: \r = paragraph break, \r\n = document end.
    const allRuns = result.value.p.flatMap((p) => p.ts);
    const docData = buildRichTextDocumentData(allRuns);
    range.setRichTextValueForCell(docData);
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

function toUniverTextStyle(style: TextStyle): ITextStyle | undefined {
  if (!style.bl && !style.it && !style.ul && !style.cl) return undefined;
  const result: ITextStyle = {};
  if (style.bl) result.bl = style.bl;
  if (style.it) result.it = style.it;
  if (style.ul) result.ul = { s: style.ul.s };
  if (style.cl) result.cl = { rgb: `#${style.cl.rgb}` };
  return result;
}

/**
 * Builds Univer IDocumentData directly from text runs, handling \n
 * characters by converting them to \r paragraph breaks. In Univer's
 * document model: \r = paragraph break, \n = section end. The document
 * always ends with \r\n. Each \r must have a corresponding entry in the
 * paragraphs array.
 */
function buildRichTextDocumentData(allRuns: TextRun[]): IDocumentData {
  let dataStream = "";
  const textRuns: ITextRun[] = [];
  const paragraphs: IParagraph[] = [];

  for (const run of allRuns) {
    const ts = run.s ? toUniverTextStyle(run.s) : undefined;
    const parts = run.t.split("\n");
    for (let i = 0; i < parts.length; i++) {
      if (i > 0) {
        dataStream += "\r";
        paragraphs.push({ startIndex: dataStream.length - 1 });
      }
      if (parts[i].length > 0) {
        const start = dataStream.length;
        dataStream += parts[i];
        if (ts) {
          textRuns.push({ st: start, ed: dataStream.length, ts });
        }
      }
    }
  }

  // Document must end with \r\n; the final \r is the last paragraph break.
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
