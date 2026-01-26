import type { DerivedResultValue, UniverRichText } from "./ipc_bridge";

const LOG_TAG = "tv.ui.richtext";

function logDebug(message: string, data?: unknown): void {
  const entry = {
    level: "DEBUG",
    component: LOG_TAG,
    message,
    data,
    timestamp: new Date().toISOString(),
  };
  console.debug(JSON.stringify(entry));
}

/**
 * Univer ICellData structure for rich text cells.
 */
export interface UniverCellData {
  v?: string | number | boolean;
  p?: UniverRichText["p"];
  s?: Record<string, unknown>;
}

/**
 * Converts a DerivedResultValue to Univer ICellData format.
 * Returns the cell data structure ready to be applied to a cell.
 */
export function derivedResultToCellData(result: DerivedResultValue): UniverCellData {
  switch (result.type) {
    case "text":
      return { v: result.value };
    case "number":
      return { v: result.value };
    case "boolean":
      return { v: result.value };
    case "image":
      // Image cells are rendered as floating images by ImageCellRenderer.
      // Set a placeholder value indicating the image path for display.
      return { v: "[Image]" };
    case "richText":
      return richTextToCellData(result.value);
    case "error":
      return errorToCellData(result.value);
  }
}

/**
 * Converts UniverRichText to ICellData with paragraph structure.
 */
function richTextToCellData(richText: UniverRichText): UniverCellData {
  logDebug("Converting rich text to cell data", { paragraphs: richText.p.length });
  return {
    p: richText.p,
  };
}

/**
 * Creates cell data for error display with red styling.
 */
function errorToCellData(message: string): UniverCellData {
  return {
    v: message,
    s: {
      cl: { rgb: "FF0000" },
    },
  };
}

/**
 * Checks if a cell value is a DerivedResultValue object.
 */
export function isDerivedResult(value: unknown): value is DerivedResultValue {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const obj = value as Record<string, unknown>;
  return (
    typeof obj.type === "string" &&
    ["text", "number", "boolean", "image", "richText", "error"].includes(obj.type) &&
    "value" in obj
  );
}

/**
 * Extracts plain text from a DerivedResultValue for display purposes.
 * Used when full rich text rendering is not available.
 */
export function derivedResultToPlainText(result: DerivedResultValue): string {
  switch (result.type) {
    case "text":
      return result.value;
    case "number":
      return String(result.value);
    case "boolean":
      return String(result.value);
    case "image":
      return `[Image: ${result.value}]`;
    case "richText":
      return extractPlainTextFromRichText(result.value);
    case "error":
      return `Error: ${result.value}`;
  }
}

/**
 * Returns true if the derived result represents an image that should be
 * rendered as a floating image rather than as cell text.
 */
export function isImageResult(result: DerivedResultValue): boolean {
  return result.type === "image" && result.value.length > 0;
}

/**
 * Extracts plain text content from rich text structure.
 */
function extractPlainTextFromRichText(richText: UniverRichText): string {
  let text = "";
  for (const paragraph of richText.p) {
    for (const run of paragraph.ts) {
      text += run.t;
    }
  }
  return text;
}
