import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Univer, IWorkbookData } from "@univerjs/core";
import { FUniver, FWorksheet } from "@univerjs/core/facade";

import {
  createUniverInstance,
  disposeUniverInstance,
  UniverInstance,
} from "./univer_config";
import type { TomlTableData } from "./ipc_bridge";

// Component logging tag: tv.ui.sheets
const LOG_TAG = "tv.ui.sheets";

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

function logInfo(message: string, data?: unknown): void {
  const entry = {
    level: "INFO",
    component: LOG_TAG,
    message,
    data,
    timestamp: new Date().toISOString(),
  };
  console.info(JSON.stringify(entry));
}

/**
 * Represents a single sheet within a multi-sheet workbook.
 * Each sheet corresponds to one TOML file.
 */
export interface SheetData {
  /** Unique sheet ID */
  id: string;
  /** Display name (filename without .toml extension) */
  name: string;
  /** Full file path for reference */
  path: string;
  /** The actual TOML table data */
  data: TomlTableData;
}

/**
 * Multi-sheet workbook data structure.
 * Contains multiple sheets, each representing a TOML file.
 */
export interface MultiSheetData {
  sheets: SheetData[];
}

export interface UniverSpreadsheetHandle {
  /** Get data from the currently active sheet */
  getData: () => TomlTableData | null;
  /** Get the ID of the currently active sheet */
  getActiveSheetId: () => string | null;
  /** Get data from a specific sheet by ID */
  getSheetData: (sheetId: string) => TomlTableData | null;
}

interface UniverSpreadsheetProps {
  width?: string | number;
  height?: string | number;
  /** Single sheet data (legacy support) */
  data?: TomlTableData;
  /** Multi-sheet data - takes precedence over single data prop */
  multiSheetData?: MultiSheetData;
  /** Called when cell data changes in any sheet */
  onChange?: (data: TomlTableData, sheetId: string) => void;
  /** Called when the active sheet changes */
  onActiveSheetChanged?: (sheetId: string) => void;
}

export const UniverSpreadsheet = forwardRef<
  UniverSpreadsheetHandle,
  UniverSpreadsheetProps
>(function UniverSpreadsheet(
  {
    width = "100%",
    height = "600px",
    data,
    multiSheetData,
    onChange,
    onActiveSheetChanged,
  },
  ref
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const univerRef = useRef<Univer | null>(null);
  const univerAPIRef = useRef<FUniver | null>(null);
  // Map of sheetId -> headers for that sheet
  const headersMapRef = useRef<Map<string, string[]>>(new Map());
  // Legacy single-sheet headers (for backward compatibility)
  const headersRef = useRef<string[]>([]);
  const onChangeRef = useRef(onChange);
  const onActiveSheetChangedRef = useRef(onActiveSheetChanged);
  const isLoadingRef = useRef(false);
  // Track if we've initialized with multi-sheet data
  const isMultiSheetRef = useRef(false);
  // Track whether initial workbook creation has completed (data already in cellData)
  const initialLoadCompleteRef = useRef(false);
  // Track last known data for each sheet to detect actual changes
  const lastMultiSheetDataRef = useRef<MultiSheetData | null>(null);

  onChangeRef.current = onChange;
  onActiveSheetChangedRef.current = onActiveSheetChanged;

  /**
   * Extract data from a specific sheet or the active sheet.
   * @param sheetId Optional sheet ID. If not provided, uses active sheet.
   */
  const extractDataFromSheet = (sheetId?: string): TomlTableData | null => {
    const workbook = univerAPIRef.current?.getActiveWorkbook();
    if (!workbook) return null;

    const sheet = sheetId
      ? workbook.getSheetBySheetId(sheetId)
      : workbook.getActiveSheet();
    if (!sheet) return null;

    const currentSheetId = sheet.getSheetId();
    const headers = isMultiSheetRef.current
      ? headersMapRef.current.get(currentSheetId) ?? []
      : headersRef.current;

    if (headers.length === 0) return null;

    const rows: (string | number | boolean | null)[][] = [];
    let rowIndex = 2;
    let hasData = true;

    while (hasData) {
      const row: (string | number | boolean | null)[] = [];
      let rowHasContent = false;

      for (let colIndex = 0; colIndex < headers.length; colIndex++) {
        const colLetter = getColumnLetter(colIndex);
        const cellAddress = `${colLetter}${rowIndex}`;
        const cellValue = sheet.getRange(cellAddress)?.getValue();

        if (cellValue !== undefined && cellValue !== null && cellValue !== "") {
          rowHasContent = true;
          row.push(cellValue as string | number | boolean);
        } else {
          row.push(null);
        }
      }

      if (rowHasContent) {
        rows.push(row);
        rowIndex++;
      } else {
        hasData = false;
      }
    }

    return { headers, rows };
  };

  /** Legacy extractData for backward compatibility */
  const extractData = (): TomlTableData | null => {
    return extractDataFromSheet();
  };

  useImperativeHandle(ref, () => ({
    getData: extractData,
    getActiveSheetId: () => {
      const sheet = univerAPIRef.current?.getActiveWorkbook()?.getActiveSheet();
      return sheet?.getSheetId() ?? null;
    },
    getSheetData: (sheetId: string) => extractDataFromSheet(sheetId),
  }));

  useEffect(() => {
    if (!containerRef.current || univerRef.current) return;

    const instance: UniverInstance = createUniverInstance({
      container: containerRef.current,
    });
    univerRef.current = instance.univer;
    univerAPIRef.current = instance.univerAPI;

    // Check if we have multi-sheet data to initialize with
    if (multiSheetData && multiSheetData.sheets.length > 0) {
      isMultiSheetRef.current = true;
      // Store headers for all sheets
      headersMapRef.current.clear();
      for (const sheetData of multiSheetData.sheets) {
        headersMapRef.current.set(sheetData.id, sheetData.data.headers);
      }
      const workbookData = buildMultiSheetWorkbook(multiSheetData);
      instance.univerAPI.createWorkbook(workbookData);

      // Apply checkbox validation to boolean columns in all sheets
      const workbook = instance.univerAPI.getActiveWorkbook();
      if (workbook) {
        for (const sheetData of multiSheetData.sheets) {
          const sheet = workbook.getSheetBySheetId(sheetData.id);
          if (sheet) {
            const booleanColumns = detectBooleanColumns(sheetData.data);
            applyCheckboxValidation(
              instance.univerAPI,
              sheet,
              sheetData.data,
              booleanColumns
            );
          }
        }
      }

      // Mark initial load as complete - data is already in cellData, no need to repopulate
      initialLoadCompleteRef.current = true;
      lastMultiSheetDataRef.current = multiSheetData;
      logInfo("Workbook initialized with multiple sheets", {
        sheetCount: multiSheetData.sheets.length,
        sheetNames: multiSheetData.sheets.map((s) => s.name),
      });
    } else {
      // Legacy single-sheet mode: create empty workbook
      instance.univerAPI.createWorkbook({});
      logDebug("Workbook initialized in single-sheet mode");
    }

    // Listen for cell value changes
    instance.univerAPI.onCommandExecuted((command) => {
      if (isLoadingRef.current) return;
      if (
        command.id === "sheet.mutation.set-range-values" ||
        command.id === "sheet.command.set-range-values"
      ) {
        const activeSheet = instance.univerAPI
          .getActiveWorkbook()
          ?.getActiveSheet();
        const sheetId = activeSheet?.getSheetId() ?? "";
        const newData = extractDataFromSheet(sheetId);
        if (newData && onChangeRef.current) {
          onChangeRef.current(newData, sheetId);
        }
      }
    });

    // Listen for active sheet changes
    const activeSheetDisposable = instance.univerAPI.addEvent(
      instance.univerAPI.Event.ActiveSheetChanged,
      (event) => {
        const { activeSheet } = event;
        const sheetId = activeSheet.getSheetId();
        logDebug("Active sheet changed", { sheetId, sheetName: activeSheet.getSheetName() });
        if (onActiveSheetChangedRef.current) {
          onActiveSheetChangedRef.current(sheetId);
        }
      }
    );

    return () => {
      activeSheetDisposable.dispose();
      disposeUniverInstance(instance);
      univerRef.current = null;
      univerAPIRef.current = null;
    };
  }, []);

  // Handle single-sheet data updates (legacy mode)
  useEffect(() => {
    // Skip if we're in multi-sheet mode
    if (isMultiSheetRef.current) return;

    const univerAPI = univerAPIRef.current;
    const sheet = univerAPI?.getActiveWorkbook()?.getActiveSheet();
    if (!univerAPI || !sheet || !data) return;

    isLoadingRef.current = true;
    headersRef.current = data.headers;

    const numColumns = data.headers.length;
    if (numColumns > 0) {
      // Set headers row using batch operation
      const headerRange = sheet.getRange(0, 0, 1, numColumns);
      if (headerRange) {
        headerRange.setValues([data.headers]);
        headerRange.setFontWeight("bold");
      }

      // Set data rows using a single batch operation
      if (data.rows.length > 0) {
        const dataRange = sheet.getRange(1, 0, data.rows.length, numColumns);
        if (dataRange) {
          // Convert null values to empty strings for display
          const displayRows = data.rows.map((row) =>
            row.map((cellValue) => (cellValue === null ? "" : cellValue))
          );
          dataRange.setValues(displayRows);
        }
      }

      // Apply checkbox validation to boolean columns
      const booleanColumns = detectBooleanColumns(data);
      applyCheckboxValidation(univerAPI, sheet, data, booleanColumns);
    }

    isLoadingRef.current = false;
  }, [data]);

  // Handle multi-sheet data updates
  useEffect(() => {
    if (!multiSheetData || multiSheetData.sheets.length === 0) return;
    if (!univerAPIRef.current) return;

    const workbook = univerAPIRef.current.getActiveWorkbook();
    if (!workbook) return;

    // Skip if this is the initial load - data was already set via cellData in buildMultiSheetWorkbook
    if (initialLoadCompleteRef.current && lastMultiSheetDataRef.current === multiSheetData) {
      logDebug("Skipping multi-sheet update - same data reference as initial load");
      return;
    }

    isLoadingRef.current = true;
    isMultiSheetRef.current = true;

    // Update headers map for all sheets
    headersMapRef.current.clear();
    for (const sheetData of multiSheetData.sheets) {
      headersMapRef.current.set(sheetData.id, sheetData.data.headers);
    }

    // Find sheets that actually changed (for reload optimization)
    const previousData = lastMultiSheetDataRef.current;
    const changedSheetIds = new Set<string>();

    if (previousData) {
      for (const sheetData of multiSheetData.sheets) {
        const prevSheet = previousData.sheets.find(s => s.id === sheetData.id);
        if (!prevSheet || !isSheetDataEqual(prevSheet.data, sheetData.data)) {
          changedSheetIds.add(sheetData.id);
        }
      }
    } else {
      // No previous data, all sheets need updating
      for (const sheetData of multiSheetData.sheets) {
        changedSheetIds.add(sheetData.id);
      }
    }

    // Only update sheets that actually changed, using batch operations
    for (const sheetData of multiSheetData.sheets) {
      if (!changedSheetIds.has(sheetData.id)) {
        logDebug("Skipping unchanged sheet", { sheetId: sheetData.id, sheetName: sheetData.name });
        continue;
      }

      const sheet = workbook.getSheetBySheetId(sheetData.id);
      if (!sheet) {
        logDebug("Sheet not found, skipping update", { sheetId: sheetData.id });
        continue;
      }

      logDebug("Updating changed sheet with batch operation", {
        sheetId: sheetData.id,
        sheetName: sheetData.name,
        rowCount: sheetData.data.rows.length,
        columnCount: sheetData.data.headers.length,
      });
      populateSheetDataBatch(sheet, sheetData.data);

      // Apply checkbox validation to boolean columns after updating data
      const booleanColumns = detectBooleanColumns(sheetData.data);
      applyCheckboxValidation(
        univerAPIRef.current!,
        sheet,
        sheetData.data,
        booleanColumns
      );
    }

    lastMultiSheetDataRef.current = multiSheetData;
    isLoadingRef.current = false;
  }, [multiSheetData]);

  return (
    <div
      ref={containerRef}
      style={{
        width: typeof width === "number" ? `${width}px` : width,
        height: typeof height === "number" ? `${height}px` : height,
      }}
    />
  );
});

function getColumnLetter(index: number): string {
  let result = "";
  let n = index;
  while (n >= 0) {
    result = String.fromCharCode((n % 26) + 65) + result;
    n = Math.floor(n / 26) - 1;
  }
  return result;
}

/**
 * Build IWorkbookData from MultiSheetData.
 * Creates a workbook with multiple sheets, sorted alphabetically by name.
 */
function buildMultiSheetWorkbook(multiSheetData: MultiSheetData): IWorkbookData {
  // Sort sheets alphabetically by name for consistent tab order
  const sortedSheets = [...multiSheetData.sheets].sort((a, b) =>
    a.name.localeCompare(b.name)
  );

  const sheets: Record<string, IWorkbookData["sheets"][string]> = {};
  const sheetOrder: string[] = [];

  for (const sheetData of sortedSheets) {
    sheetOrder.push(sheetData.id);

    // Calculate required dimensions
    const rowCount = Math.max(sheetData.data.rows.length + 2, 100); // +1 for header, +1 for buffer
    const columnCount = Math.max(sheetData.data.headers.length + 1, 26);

    // Build cell data
    const cellData: Record<number, Record<number, { v: unknown }>> = {};

    // Header row (row 0)
    cellData[0] = {};
    sheetData.data.headers.forEach((header, colIndex) => {
      cellData[0][colIndex] = { v: header };
    });

    // Data rows (starting at row 1)
    sheetData.data.rows.forEach((row, rowIndex) => {
      cellData[rowIndex + 1] = {};
      row.forEach((cellValue, colIndex) => {
        if (cellValue !== null) {
          cellData[rowIndex + 1][colIndex] = { v: cellValue };
        }
      });
    });

    sheets[sheetData.id] = {
      id: sheetData.id,
      name: sheetData.name,
      rowCount,
      columnCount,
      cellData,
    };

    logDebug("Sheet created", {
      sheetId: sheetData.id,
      sheetName: sheetData.name,
      rowCount: sheetData.data.rows.length,
      columnCount: sheetData.data.headers.length,
    });
  }

  return {
    id: "tv-workbook",
    name: "TV Workbook",
    sheets,
    sheetOrder,
  };
}

/** Sheet interface matching FWorksheet facade methods we use */
interface SheetFacade {
  getRange(
    startRow: number,
    startColumn: number,
    numRows: number,
    numColumns: number
  ): {
    setValues(values: unknown[][]): void;
    setFontWeight(weight: string): void;
  } | null;
}

/**
 * Compare two TomlTableData objects for equality.
 */
function isSheetDataEqual(a: TomlTableData, b: TomlTableData): boolean {
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
function populateSheetDataBatch(sheet: SheetFacade, data: TomlTableData): void {
  if (!sheet) return;

  const numColumns = data.headers.length;
  if (numColumns === 0) return;

  // Set headers row using batch operation
  const headerRange = sheet.getRange(0, 0, 1, numColumns);
  if (headerRange) {
    headerRange.setValues([data.headers]);
    headerRange.setFontWeight("bold");
  }

  // Set data rows using a single batch operation
  if (data.rows.length > 0) {
    const dataRange = sheet.getRange(1, 0, data.rows.length, numColumns);
    if (dataRange) {
      // Convert null values to empty strings for display
      const displayRows = data.rows.map((row) =>
        row.map((cellValue) => (cellValue === null ? "" : cellValue))
      );
      dataRange.setValues(displayRows);
    }
  }
}

/**
 * Detect columns that contain only boolean values (or nulls).
 * Returns an array of column indices that should be rendered as checkboxes.
 */
function detectBooleanColumns(data: TomlTableData): number[] {
  const booleanColumns: number[] = [];

  for (let colIdx = 0; colIdx < data.headers.length; colIdx++) {
    let hasNonNullValue = false;
    let allBoolean = true;

    for (const row of data.rows) {
      const value = row[colIdx];
      if (value === null || value === undefined) {
        continue;
      }
      hasNonNullValue = true;
      if (typeof value !== "boolean") {
        allBoolean = false;
        break;
      }
    }

    if (hasNonNullValue && allBoolean) {
      booleanColumns.push(colIdx);
    }
  }

  return booleanColumns;
}

/**
 * Apply checkbox data validation to boolean columns.
 * Uses Univer's data validation API to render checkboxes for boolean values.
 */
function applyCheckboxValidation(
  univerAPI: FUniver,
  sheet: FWorksheet,
  data: TomlTableData,
  booleanColumns: number[]
): void {
  if (booleanColumns.length === 0 || data.rows.length === 0) {
    return;
  }

  for (const colIdx of booleanColumns) {
    const colLetter = getColumnLetter(colIdx);
    const startRow = 2;
    const endRow = data.rows.length + 1;
    const rangeAddress = `${colLetter}${startRow}:${colLetter}${endRow}`;
    const range = sheet.getRange(rangeAddress);

    if (range) {
      const rule = univerAPI
        .newDataValidation()
        .requireCheckbox()
        .setOptions({ allowBlank: true, showErrorMessage: false })
        .build();
      range.setDataValidation(rule);

      logDebug("Applied checkbox validation to column", {
        column: data.headers[colIdx],
        range: rangeAddress,
      });
    }
  }
}
