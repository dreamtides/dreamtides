import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Univer, IWorkbookData } from "@univerjs/core";
import { FUniver } from "@univerjs/core/facade";

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
      const workbookData = buildMultiSheetWorkbook(multiSheetData);
      instance.univerAPI.createWorkbook(workbookData);
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

    const sheet = univerAPIRef.current?.getActiveWorkbook()?.getActiveSheet();
    if (!sheet || !data) return;

    isLoadingRef.current = true;
    headersRef.current = data.headers;

    data.headers.forEach((header, colIndex) => {
      const colLetter = getColumnLetter(colIndex);
      const range = sheet.getRange(`${colLetter}1`);
      range?.setValue(header);
      range?.setFontWeight("bold");
    });

    data.rows.forEach((row, rowIndex) => {
      row.forEach((cellValue, colIndex) => {
        const colLetter = getColumnLetter(colIndex);
        const cellAddress = `${colLetter}${rowIndex + 2}`;
        const displayValue = cellValue === null ? "" : cellValue;
        sheet.getRange(cellAddress)?.setValue(displayValue);
      });
    });

    isLoadingRef.current = false;
  }, [data]);

  // Handle multi-sheet data updates
  useEffect(() => {
    if (!multiSheetData || multiSheetData.sheets.length === 0) return;
    if (!univerAPIRef.current) return;

    const workbook = univerAPIRef.current.getActiveWorkbook();
    if (!workbook) return;

    isLoadingRef.current = true;
    isMultiSheetRef.current = true;

    // Update headers map for all sheets
    headersMapRef.current.clear();
    for (const sheetData of multiSheetData.sheets) {
      headersMapRef.current.set(sheetData.id, sheetData.data.headers);
    }

    // If workbook was already created with multi-sheet data, update cell contents
    // Otherwise, this is handled by initial workbook creation
    for (const sheetData of multiSheetData.sheets) {
      const sheet = workbook.getSheetBySheetId(sheetData.id);
      if (!sheet) {
        logDebug("Sheet not found, skipping update", { sheetId: sheetData.id });
        continue;
      }

      populateSheetData(sheet, sheetData.data);
    }

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
  getRange(address: string): {
    setValue(value: unknown): void;
    setFontWeight(weight: string): void;
  } | null;
}

/**
 * Populate a sheet with TomlTableData.
 * Used for updating sheet contents after initial creation.
 */
function populateSheetData(sheet: SheetFacade, data: TomlTableData): void {
  if (!sheet) return;

  // Set headers with bold styling
  data.headers.forEach((header, colIndex) => {
    const colLetter = getColumnLetter(colIndex);
    const range = sheet.getRange(`${colLetter}1`);
    range?.setValue(header);
    range?.setFontWeight("bold");
  });

  // Set data rows
  data.rows.forEach((row, rowIndex) => {
    row.forEach((cellValue, colIndex) => {
      const colLetter = getColumnLetter(colIndex);
      const cellAddress = `${colLetter}${rowIndex + 2}`;
      const displayValue = cellValue === null ? "" : cellValue;
      sheet.getRange(cellAddress)?.setValue(displayValue);
    });
  });
}
