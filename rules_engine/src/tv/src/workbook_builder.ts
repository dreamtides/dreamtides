import {
  IWorkbookData,
  CellValueType,
  HorizontalAlign,
  VerticalAlign,
  WrapStrategy,
} from "@univerjs/core";

import type {
  DerivedColumnInfo,
  RowConfig,
  ColumnConfig,
} from "./ipc_bridge";
import type { MultiSheetData } from "./spreadsheet_types";
import { formatHeaderForDisplay } from "./header_utils";
import { computeDataColumnOffset } from "./derived_column_utils";
import { createLogger } from "./logger_frontend";

const logger = createLogger("tv.ui.workbook_builder");

/**
 * Apply sheet ordering: use persisted order if available, falling back to
 * alphabetical sort for any sheets not listed in the persisted order.
 */
function applySheetOrder(
  sheets: MultiSheetData["sheets"],
  persistedOrder?: string[],
): MultiSheetData["sheets"] {
  if (!persistedOrder || persistedOrder.length === 0) {
    // No persisted order: fall back to alphabetical sort
    return [...sheets].sort((a, b) => a.name.localeCompare(b.name));
  }

  // Build a map from filename to sheet for fast lookup
  const sheetByName = new Map(sheets.map((s) => [s.name, s]));
  const ordered: MultiSheetData["sheets"] = [];
  const seen = new Set<string>();

  // First, add sheets in persisted order
  for (const name of persistedOrder) {
    const sheet = sheetByName.get(name);
    if (sheet) {
      ordered.push(sheet);
      seen.add(name);
    }
  }

  // Then, append any new sheets not in persisted order (alphabetically)
  const remaining = sheets
    .filter((s) => !seen.has(s.name))
    .sort((a, b) => a.name.localeCompare(b.name));
  ordered.push(...remaining);

  return ordered;
}

/**
 * Build IWorkbookData from MultiSheetData.
 * Creates a workbook with multiple sheets, ordered by persisted sheet order
 * or alphabetically by name if no persisted order exists.
 */
export function buildMultiSheetWorkbook(
  multiSheetData: MultiSheetData,
  derivedConfigs?: Record<string, DerivedColumnInfo[]>,
  rowConfigs?: Record<string, RowConfig>,
  columnConfigs?: Record<string, ColumnConfig[]>,
  persistedSheetOrder?: string[],
): Partial<IWorkbookData> {
  const sortedSheets = applySheetOrder(
    multiSheetData.sheets,
    persistedSheetOrder,
  );

  const sheets: Record<string, IWorkbookData["sheets"][string]> = {};
  const sheetOrder: string[] = [];

  for (const sheetData of sortedSheets) {
    sheetOrder.push(sheetData.id);

    const configs = derivedConfigs?.[sheetData.id];
    const dataOffset = computeDataColumnOffset(configs);

    // Calculate required dimensions
    const rowCount = sheetData.data.rows.length + 1 + 100; // +1 for header row, +100 blank rows at bottom
    const columnCount = Math.max(
      sheetData.data.headers.length + dataOffset + 1,
      26,
    );

    // Build cell data
    const cellData: Record<
      number,
      Record<number, { v: unknown; t?: CellValueType; s?: { bl?: number } }>
    > = {};

    // Header row (row 0) with display-formatted names and bold styling
    cellData[0] = {};
    sheetData.data.headers.forEach((header, colIndex) => {
      cellData[0][colIndex + dataOffset] = {
        v: formatHeaderForDisplay(header),
        s: { bl: 1 },
      };
    });

    // Set derived column headers at their explicit positions
    if (configs) {
      for (const config of configs) {
        if (config.position !== undefined && config.position !== null) {
          cellData[0][config.position] = {
            v: formatHeaderForDisplay(config.name),
            s: { bl: 1 },
          };
        }
      }
    }

    // Build a set of bold column indices from column configs
    const boldColumnIndices = new Set<number>();
    if (sheetColumnConfigs) {
      for (const colConfig of sheetColumnConfigs) {
        if (colConfig.bold) {
          const headerIndex = sheetData.data.headers.indexOf(colConfig.key);
          if (headerIndex !== -1) {
            boldColumnIndices.add(headerIndex);
          }
        }
      }
    }

    // Data rows (starting at row 1), shifted by data offset
    sheetData.data.rows.forEach((row, rowIndex) => {
      cellData[rowIndex + 1] = {};
      row.forEach((cellValue, colIndex) => {
        if (cellValue !== null) {
          const isBold = boldColumnIndices.has(colIndex);
          if (typeof cellValue === "boolean") {
            const cell: { v: unknown; t?: CellValueType; s?: { bl?: number } } = {
              v: cellValue ? 1 : 0,
              t: CellValueType.BOOLEAN,
            };
            if (isBold) {
              cell.s = { bl: 1 };
            }
            cellData[rowIndex + 1][colIndex + dataOffset] = cell;
          } else {
            const cell: { v: unknown; s?: { bl?: number } } = { v: cellValue };
            if (isBold) {
              cell.s = { bl: 1 };
            }
            cellData[rowIndex + 1][colIndex + dataOffset] = cell;
          }
        }
      });
    });

    // Set column widths for positioned derived columns
    const columnData: Record<number, { w: number }> = {};
    if (configs) {
      for (const config of configs) {
        if (
          config.position !== undefined &&
          config.position !== null &&
          config.width
        ) {
          columnData[config.position] = { w: config.width };
        }
      }
    }

    // Apply persisted column widths from metadata
    const sheetColumnConfigs = columnConfigs?.[sheetData.id];
    if (sheetColumnConfigs) {
      for (const colConfig of sheetColumnConfigs) {
        const headerIndex = sheetData.data.headers.indexOf(colConfig.key);
        if (headerIndex !== -1 && colConfig.width && colConfig.width !== 100) {
          columnData[headerIndex + dataOffset] = { w: colConfig.width };
        }
      }
    }

    // Apply row height configuration from metadata
    const rowConfig = rowConfigs?.[sheetData.id];
    const rowData: Record<number, { h: number; hd?: number }> = {};

    // Header row (row 0) always gets an explicit height, defaulting to 40px
    const headerH = rowConfig?.header_height ?? 40;
    rowData[0] = { h: headerH };

    if (rowConfig) {
      const defaultH = rowConfig.default_height;
      if (defaultH) {
        // Apply default height to data rows only (row 1 onward)
        for (let r = 1; r <= sheetData.data.rows.length; r++) {
          rowData[r] = { h: defaultH };
        }
      }
      // Apply per-row overrides (row indices are 0-indexed data rows, offset +1 for header)
      if (rowConfig.heights) {
        for (const rh of rowConfig.heights) {
          rowData[rh.row + 1] = { h: rh.height };
        }
      }
      // Mark hidden rows
      if (rowConfig.hidden) {
        for (const hiddenRow of rowConfig.hidden) {
          const r = hiddenRow + 1;
          if (rowData[r]) {
            rowData[r].hd = 1;
          } else {
            rowData[r] = { h: rowConfig.default_height ?? 24, hd: 1 };
          }
        }
      }
    }

    const sheetConfig: Record<string, unknown> = {
      id: sheetData.id,
      name: sheetData.name,
      rowCount,
      columnCount,
      cellData,
      defaultStyle: {
        ht: HorizontalAlign.LEFT,
        vt: VerticalAlign.MIDDLE,
        tb: WrapStrategy.WRAP,
      },
    };
    if (rowConfig?.default_height) {
      sheetConfig.defaultRowHeight = rowConfig.default_height;
    }
    if (Object.keys(rowData).length > 0) {
      sheetConfig.rowData = rowData;
    }
    if (Object.keys(columnData).length > 0) {
      sheetConfig.columnData = columnData;
    }
    sheets[sheetData.id] = sheetConfig as IWorkbookData["sheets"][string];

    logger.debug("Sheet created", {
      sheetId: sheetData.id,
      sheetName: sheetData.name,
      rowCount: sheetData.data.rows.length,
      columnCount: sheetData.data.headers.length,
      dataOffset,
    });
  }

  return {
    id: "tv-workbook",
    name: "TV Workbook",
    sheets,
    sheetOrder,
  };
}
