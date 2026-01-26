import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Univer, IWorkbookData } from "@univerjs/core";
import { FUniver, FWorksheet } from "@univerjs/core/facade";

import {
  createUniverInstance,
  disposeUniverInstance,
  UniverInstance,
} from "./univer_config";
import type { TomlTableData, EnumValidationInfo, DerivedColumnInfo, DerivedResultValue, ResolvedTableStyle, CellFormatResult } from "./ipc_bridge";
import * as ipc from "./ipc_bridge";
import { derivedResultToCellData } from "./rich_text_utils";
import { ImageCellRenderer } from "./image_cell_renderer";

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
 * Tracks derived column configurations and computed values per sheet.
 *
 * configs: Maps sheet ID to its derived column configurations.
 * values: Maps sheet ID -> row index -> function name -> computed result.
 */
export interface DerivedColumnState {
  configs: Record<string, DerivedColumnInfo[]>;
  values: Record<string, Record<number, Record<string, DerivedResultValue>>>;
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
  /** Derived column configurations and computed values */
  derivedColumnState?: DerivedColumnState;
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
    derivedColumnState,
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
  // Track enum validation rules per sheet (by file path)
  const enumRulesRef = useRef<Map<string, EnumValidationInfo[]>>(new Map());
  // Image cell renderer for floating image support
  const imageCellRendererRef = useRef<ImageCellRenderer | null>(null);
  // Ref to track current multiSheetData for use in event callbacks
  const multiSheetDataRef = useRef<MultiSheetData | undefined>(multiSheetData);
  multiSheetDataRef.current = multiSheetData;
  // Suppress sort event persistence during initial sort state restoration
  const isRestoringSortRef = useRef(false);
  // Track resolved table styles per sheet path
  const tableStylesRef = useRef<Map<string, ResolvedTableStyle>>(new Map());
  // Track conditional formatting results per sheet path
  const conditionalFormatsRef = useRef<Map<string, CellFormatResult[]>>(new Map());

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
    imageCellRendererRef.current = new ImageCellRenderer();

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

      // Load enum validation rules and apply dropdown validation for all sheets
      const loadEnumRulesAsync = async () => {
        const workbook = instance.univerAPI.getActiveWorkbook();
        if (!workbook) return;

        for (const sheetData of multiSheetData.sheets) {
          try {
            const enumRules = await ipc.getEnumValidationRules(sheetData.path);
            enumRulesRef.current.set(sheetData.path, enumRules);

            if (enumRules.length > 0) {
              const sheet = workbook.getSheetBySheetId(sheetData.id);
              if (sheet) {
                applyDropdownValidation(
                  instance.univerAPI,
                  sheet,
                  sheetData.data,
                  enumRules
                );
              }
            }
          } catch (e) {
            logDebug("Failed to load enum validation rules", {
              sheetId: sheetData.id,
              path: sheetData.path,
              error: String(e),
            });
          }
        }
      };
      loadEnumRulesAsync();

      // Restore sort indicators from persisted backend sort state
      const restoreSortStateAsync = async () => {
        const workbook = instance.univerAPI.getActiveWorkbook();
        if (!workbook) return;

        isRestoringSortRef.current = true;
        try {
          for (const sheetData of multiSheetData.sheets) {
            try {
              const sortResponse = await ipc.getSortState(
                sheetData.path,
                sheetData.name
              );
              if (!sortResponse.column || !sortResponse.direction) continue;

              const colIndex = sheetData.data.headers.indexOf(
                sortResponse.column
              );
              if (colIndex === -1) {
                logDebug("Persisted sort column not found in headers", {
                  column: sortResponse.column,
                  sheetId: sheetData.id,
                });
                continue;
              }

              const sheet = workbook.getSheetBySheetId(sheetData.id);
              if (!sheet) continue;

              const ascending = sortResponse.direction === "ascending";
              sheet.sort(colIndex, ascending);
              logInfo("Restored sort indicator", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                column: sortResponse.column,
                ascending,
              });
            } catch (e) {
              logDebug("Failed to restore sort state", {
                sheetId: sheetData.id,
                path: sheetData.path,
                error: String(e),
              });
            }
          }
        } finally {
          isRestoringSortRef.current = false;
        }
      };
      restoreSortStateAsync();

      // Load and apply table color schemes for all sheets
      const loadTableStylesAsync = async () => {
        const workbook = instance.univerAPI.getActiveWorkbook();
        if (!workbook) return;

        for (const sheetData of multiSheetData.sheets) {
          try {
            const style = await ipc.getTableStyle(sheetData.path);
            if (!style) continue;

            tableStylesRef.current.set(sheetData.path, style);
            const sheet = workbook.getSheetBySheetId(sheetData.id);
            if (sheet) {
              applyTableStyle(sheet, sheetData.data, style);
              logInfo("Applied table color scheme", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                colorScheme: style.palette ? "resolved" : "none",
                showRowStripes: style.show_row_stripes,
              });
            }
          } catch (e) {
            logDebug("Failed to load table style", {
              sheetId: sheetData.id,
              path: sheetData.path,
              error: String(e),
            });
          }
        }
      };
      loadTableStylesAsync();

      // Load and apply conditional formatting for all sheets
      const loadConditionalFormattingAsync = async () => {
        const workbook = instance.univerAPI.getActiveWorkbook();
        if (!workbook) return;

        for (const sheetData of multiSheetData.sheets) {
          try {
            const rowsAsJson = sheetData.data.rows.map((row) =>
              row.map((cell) => (cell === null ? null : cell))
            );
            const results = await ipc.getConditionalFormatting(
              sheetData.path,
              sheetData.data.headers,
              rowsAsJson
            );
            if (results.length === 0) continue;

            conditionalFormatsRef.current.set(sheetData.path, results);
            const sheet = workbook.getSheetBySheetId(sheetData.id);
            if (sheet) {
              applyConditionalFormatting(sheet, results);
              logInfo("Applied conditional formatting", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                matchCount: results.length,
              });
            }
          } catch (e) {
            logDebug("Failed to load conditional formatting", {
              sheetId: sheetData.id,
              path: sheetData.path,
              error: String(e),
            });
          }
        }
      };
      loadConditionalFormattingAsync();

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

    // Listen for sort events to persist sort state to backend
    const sortDisposable = instance.univerAPI.addEvent(
      instance.univerAPI.Event.SheetRangeSorted,
      (params: { worksheet: { getSheetId: () => string }; sortColumn: { column: number; ascending: boolean }[] }) => {
        if (isRestoringSortRef.current) return;

        const sheetId = params.worksheet.getSheetId();
        const sheets = multiSheetDataRef.current?.sheets;
        const sheetData = sheets?.find((s) => s.id === sheetId);
        if (!sheetData) {
          logDebug("Sort event for unknown sheet", { sheetId });
          return;
        }

        const headers = headersMapRef.current.get(sheetId) ?? [];
        if (params.sortColumn.length === 0) {
          ipc.clearSortState(sheetData.path, sheetData.name).catch((e) => {
            logDebug("Failed to clear sort state", { error: String(e) });
          });
          logInfo("Sort cleared", { sheetId, sheetName: sheetData.name });
          return;
        }

        const sortSpec = params.sortColumn[0];
        const columnName = headers[sortSpec.column];
        if (!columnName) {
          logDebug("Sort column index out of range", {
            columnIndex: sortSpec.column,
            headerCount: headers.length,
          });
          return;
        }

        const direction: ipc.SortDirection = sortSpec.ascending
          ? "ascending"
          : "descending";
        ipc.setSortState(sheetData.path, sheetData.name, {
          column: columnName,
          direction,
        }).catch((e) => {
          logDebug("Failed to persist sort state", { error: String(e) });
        });

        logInfo("Sort applied", {
          sheetId,
          sheetName: sheetData.name,
          column: columnName,
          direction,
        });
      }
    );

    // Listen for derived value computed events to handle image results
    const derivedValueSub = ipc.onDerivedValueComputed((payload) => {
      const result = payload.result;
      if (!result || result.type !== "image") return;

      const workbook = instance.univerAPI.getActiveWorkbook();
      if (!workbook) return;

      // Find the sheet matching the derived value's file path
      const matchingSheetData = multiSheetData?.sheets.find(
        (s) => s.path === payload.file_path
      );

      const sheet = matchingSheetData
        ? workbook.getSheetBySheetId(matchingSheetData.id)
        : workbook.getActiveSheet();
      if (!sheet) return;

      const sheetId = sheet.getSheetId();
      const renderer = imageCellRendererRef.current;
      if (!renderer) return;

      // Row index from derived value is 0-based data row, add 1 for header row offset
      const displayRow = payload.row_index + 1;

      // Find the column index for the derived function name in headers
      const headers = headersMapRef.current.get(sheetId) ?? headersRef.current;
      const colIdx = headers.indexOf(payload.function_name);
      if (colIdx === -1) {
        logDebug("Image derived function column not found in headers", {
          functionName: payload.function_name,
          headers,
        });
        return;
      }

      renderer
        .handleImageResult(sheet, sheetId, displayRow, colIdx, result)
        .catch((e) => {
          logDebug("Image render from derived value failed", {
            error: String(e),
            rowIndex: payload.row_index,
            functionName: payload.function_name,
          });
        });
    });

    return () => {
      activeSheetDisposable.dispose();
      sortDisposable.dispose();
      derivedValueSub.dispose();
      if (imageCellRendererRef.current) {
        imageCellRendererRef.current.clearAll();
        imageCellRendererRef.current = null;
      }
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

      // Clear existing images for this sheet before repopulating data
      if (imageCellRendererRef.current) {
        imageCellRendererRef.current.clearSheetImages(sheet, sheetData.id);
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

      // Apply dropdown validation from cached enum rules
      const cachedEnumRules = enumRulesRef.current.get(sheetData.path);
      if (cachedEnumRules && cachedEnumRules.length > 0) {
        applyDropdownValidation(
          univerAPIRef.current!,
          sheet,
          sheetData.data,
          cachedEnumRules
        );
      }

      // Re-apply cached table style after data update
      const cachedStyle = tableStylesRef.current.get(sheetData.path);
      if (cachedStyle) {
        applyTableStyle(sheet, sheetData.data, cachedStyle);
      }

      // Re-evaluate and apply conditional formatting after data update
      const reevaluateConditionalFormatting = async () => {
        try {
          const rowsAsJson = sheetData.data.rows.map((row) =>
            row.map((cell) => (cell === null ? null : cell))
          );
          const results = await ipc.getConditionalFormatting(
            sheetData.path,
            sheetData.data.headers,
            rowsAsJson
          );
          conditionalFormatsRef.current.set(sheetData.path, results);
          const ws = workbook.getSheetBySheetId(sheetData.id);
          if (ws && results.length > 0) {
            applyConditionalFormatting(ws, results);
          }
        } catch (e) {
          logDebug("Failed to re-evaluate conditional formatting", {
            sheetId: sheetData.id,
            error: String(e),
          });
        }
      };
      reevaluateConditionalFormatting();
    }

    lastMultiSheetDataRef.current = multiSheetData;
    isLoadingRef.current = false;
  }, [multiSheetData]);

  // Handle derived column value updates
  useEffect(() => {
    if (!derivedColumnState) return;
    if (!univerAPIRef.current) return;

    const workbook = univerAPIRef.current.getActiveWorkbook();
    if (!workbook) return;

    for (const [sheetId, configs] of Object.entries(derivedColumnState.configs)) {
      if (configs.length === 0) continue;

      const sheet = workbook.getSheetBySheetId(sheetId);
      if (!sheet) continue;

      const headers = headersMapRef.current.get(sheetId) ?? headersRef.current;
      if (headers.length === 0) continue;

      const sheetValues = derivedColumnState.values[sheetId];
      if (!sheetValues) continue;

      isLoadingRef.current = true;

      for (const config of configs) {
        const derivedColIndex = getDerivedColumnIndex(config, headers.length, configs);

        // Set header for derived column (row 0)
        const headerRange = sheet.getRange(0, derivedColIndex, 1, 1);
        if (headerRange) {
          headerRange.setValues([[config.name]]);
          headerRange.setFontWeight("bold");
        }

        // Update each row with computed values
        for (const [rowIndexStr, rowValues] of Object.entries(sheetValues)) {
          const rowIndex = parseInt(rowIndexStr, 10);
          const result = rowValues[config.function];
          if (!result) continue;

          const cellRow = rowIndex + 1; // +1 for header row
          applyDerivedResultToCell(sheet, cellRow, derivedColIndex, result);
        }

        // Show loading indicator for rows without results
        if (multiSheetData) {
          const sheetData = multiSheetData.sheets.find((s) => s.id === sheetId);
          if (sheetData) {
            for (let rowIndex = 0; rowIndex < sheetData.data.rows.length; rowIndex++) {
              const rowValues = sheetValues[rowIndex];
              if (!rowValues || !rowValues[config.function]) {
                const cellRow = rowIndex + 1;
                const range = sheet.getRange(cellRow, derivedColIndex, 1, 1);
                if (range) {
                  range.setValues([[""]]);
                  range.setFontColor("#999999");
                }
              }
            }
          }
        }
      }

      isLoadingRef.current = false;
    }
  }, [derivedColumnState, multiSheetData]);

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

/**
 * Apply dropdown (list) data validation to enum columns.
 * Uses Univer's data validation API to render dropdowns for enum values with type-ahead filtering.
 */
function applyDropdownValidation(
  univerAPI: FUniver,
  sheet: FWorksheet,
  data: TomlTableData,
  enumRules: EnumValidationInfo[]
): void {
  if (enumRules.length === 0 || data.rows.length === 0) {
    return;
  }

  for (const rule of enumRules) {
    const colIdx = data.headers.indexOf(rule.column);
    if (colIdx === -1) {
      logDebug("Enum column not found in headers, skipping dropdown validation", {
        column: rule.column,
      });
      continue;
    }

    const colLetter = getColumnLetter(colIdx);
    const startRow = 2;
    const endRow = data.rows.length + 1;
    const rangeAddress = `${colLetter}${startRow}:${colLetter}${endRow}`;
    const range = sheet.getRange(rangeAddress);

    if (range) {
      const validationRule = univerAPI
        .newDataValidation()
        .requireValueInList(rule.allowed_values, true)
        .setOptions({
          allowBlank: true,
          showErrorMessage: true,
          error: `Value must be one of: ${rule.allowed_values.join(", ")}`,
        })
        .build();
      range.setDataValidation(validationRule);

      logDebug("Applied dropdown validation to column", {
        column: rule.column,
        range: rangeAddress,
        allowedValues: rule.allowed_values,
      });
    }
  }
}

/**
 * Calculates the column index for a derived column.
 * If the config has an explicit position, uses that. Otherwise appends
 * after all data columns plus any prior derived columns without positions.
 */
function getDerivedColumnIndex(
  config: DerivedColumnInfo,
  dataColumnCount: number,
  allConfigs: DerivedColumnInfo[]
): number {
  if (config.position !== undefined && config.position !== null) {
    return config.position;
  }

  // Place after all data columns plus prior derived columns without explicit position
  let offset = 0;
  for (const c of allConfigs) {
    if (c.name === config.name) break;
    if (c.position === undefined || c.position === null) {
      offset++;
    }
  }
  return dataColumnCount + offset;
}

/**
 * Applies a derived result value to a specific cell in the spreadsheet.
 * Handles all DerivedResult variants: text, number, boolean, image,
 * richText, and error. Error results display with red font color.
 */
function applyDerivedResultToCell(
  sheet: FWorksheet,
  row: number,
  col: number,
  result: DerivedResultValue
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
    logDebug("Applied rich text derived result", { row, col });
  } else if (result.type === "error") {
    range.setValues([[`Error: ${result.value}`]]);
    range.setFontColor("#FF0000");
    logDebug("Applied error derived result", { row, col, error: result.value });
  } else if (result.type === "image") {
    range.setValues([[`[Image: ${result.value}]`]]);
    range.setFontColor("#0066CC");
    logDebug("Applied image derived result", { row, col, url: result.value });
  } else {
    const value = cellData.v !== undefined ? cellData.v : "";
    range.setValues([[value]]);
    // Reset font color to default for non-error results
    range.setFontColor("#000000");
    logDebug("Applied derived result", { row, col, type: result.type });
  }
}

/**
 * Applies a resolved table style (color scheme, row stripes, header styling)
 * to a sheet. Colors follow visual (display) row order, not data order,
 * so alternating stripes remain consistent after sorting.
 */
function applyTableStyle(
  sheet: FWorksheet,
  data: TomlTableData,
  style: ResolvedTableStyle
): void {
  const numColumns = data.headers.length;
  if (numColumns === 0) return;

  const headerBg = style.header_background ?? style.palette?.header_background;
  const headerFontColor = style.palette?.header_font_color;

  // Apply header row styling
  const headerRange = sheet.getRange(0, 0, 1, numColumns);
  if (headerRange) {
    if (style.header_bold) {
      headerRange.setFontWeight("bold");
    }
    if (headerBg) {
      headerRange.setBackgroundColor(headerBg);
    }
    if (headerFontColor) {
      headerRange.setFontColor(headerFontColor);
    }
  }

  // Apply alternating row stripe colors
  if (style.show_row_stripes && style.palette && data.rows.length > 0) {
    const evenBg = style.palette.row_even_background;
    const oddBg = style.palette.row_odd_background;

    for (let displayRow = 0; displayRow < data.rows.length; displayRow++) {
      const cellRow = displayRow + 1;
      const rowRange = sheet.getRange(cellRow, 0, 1, numColumns);
      if (rowRange) {
        rowRange.setBackgroundColor(displayRow % 2 === 0 ? evenBg : oddBg);
      }
    }
  }

  // Apply alternating column stripe colors
  if (style.show_column_stripes && style.palette && data.rows.length > 0) {
    const evenBg = style.palette.row_even_background;
    const accentBg = style.palette.accent_color;

    for (let colIdx = 0; colIdx < numColumns; colIdx++) {
      if (colIdx % 2 === 1) {
        const colRange = sheet.getRange(1, colIdx, data.rows.length, 1);
        if (colRange) {
          colRange.setBackgroundColor(accentBg + "33");
        }
      }
    }
  }
}

/**
 * Applies conditional formatting results to cells in a sheet.
 * Each result specifies a row, column index, and style to apply.
 * Conditional formatting runs after table color scheme application
 * so that conditional styles override base table styling.
 */
function applyConditionalFormatting(
  sheet: FWorksheet,
  results: CellFormatResult[]
): void {
  for (const result of results) {
    const cellRow = result.row + 1; // +1 for header row
    const range = sheet.getRange(cellRow, result.col_index, 1, 1);
    if (!range) continue;

    if (result.style.background_color) {
      range.setBackgroundColor(result.style.background_color);
    }
    if (result.style.font_color) {
      range.setFontColor(result.style.font_color);
    }
    if (result.style.bold === true) {
      range.setFontWeight("bold");
    }
    if (result.style.italic === true) {
      range.setFontStyle("italic");
    }
    if (result.style.underline === true) {
      // Underline via setUnderline if available on FRange; cast to access optional API
      const rangeAny = range as unknown as Record<string, unknown>;
      if (typeof rangeAny.setUnderline === "function") {
        (rangeAny.setUnderline as (v: boolean) => void)(true);
      }
    }
  }
}
