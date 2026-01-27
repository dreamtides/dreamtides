import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Univer } from "@univerjs/core";
import { FUniver } from "@univerjs/core/facade";

import {
  createUniverInstance,
  disposeUniverInstance,
  UniverInstance,
} from "./univer_config";
import type {
  TomlTableData,
  EnumValidationInfo,
  ResolvedTableStyle,
  CellFormatResult,
  SortDirection,
} from "./ipc_bridge";
import * as ipc from "./ipc_bridge";
import { formatHeaderForDisplay } from "./header_utils";
import { buildMultiSheetWorkbook } from "./workbook_builder";
import { isSheetDataEqual, populateSheetDataBatch } from "./sheet_data_utils";
import {
  detectBooleanColumns,
  applyCheckboxValidation,
  applyDropdownValidation,
} from "./validation_utils";
import {
  getDerivedColumnIndex,
  computeDataColumnOffset,
  applyDerivedResultToCell,
} from "./derived_column_utils";
import { applyTableStyle, applyConditionalFormatting } from "./table_style_utils";
import { ImageCellRenderer } from "./image_cell_renderer";
import { createLogger } from "./logger_frontend";
import type {
  MultiSheetData,
  UniverSpreadsheetHandle,
  UniverSpreadsheetProps,
} from "./spreadsheet_types";

export type {
  DerivedColumnState,
  SheetData,
  MultiSheetData,
  UniverSpreadsheetHandle,
} from "./spreadsheet_types";

const logger = createLogger("tv.ui.spreadsheet");

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
    initialActiveSheetId,
    rowConfigs,
    columnConfigs,
  },
  ref,
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
  const conditionalFormatsRef = useRef<Map<string, CellFormatResult[]>>(
    new Map(),
  );
  // Suppress filter event persistence during initial filter state restoration
  const isRestoringFilterRef = useRef(false);
  const derivedColumnStateRef = useRef(derivedColumnState);
  derivedColumnStateRef.current = derivedColumnState;
  const rowConfigsRef = useRef(rowConfigs);
  rowConfigsRef.current = rowConfigs;
  const columnConfigsRef = useRef(columnConfigs);
  columnConfigsRef.current = columnConfigs;
  const dataColumnOffsetMapRef = useRef<Map<string, number>>(new Map());

  onChangeRef.current = onChange;
  onActiveSheetChangedRef.current = onActiveSheetChanged;

  /**
   * Extract data from a specific sheet or the active sheet.
   * Uses a single batch getValues() call instead of per-cell reads.
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
      ? (headersMapRef.current.get(currentSheetId) ?? [])
      : headersRef.current;

    if (headers.length === 0) return null;

    const knownSheetData = multiSheetDataRef.current?.sheets.find(
      (s) => s.id === currentSheetId,
    );
    const minRows = knownSheetData?.data.rows.length ?? 0;

    const dataOffset = dataColumnOffsetMapRef.current.get(currentSheetId) ?? 0;
    const maxRows = sheet.getMaxRows();

    // Read all data rows in a single batch call instead of cell-by-cell.
    // Row 0 is the header row, data starts at row 1.
    const readRowCount = maxRows - 1;
    if (readRowCount <= 0) return { headers, rows: [] };

    const dataRange = sheet.getRange(
      1,
      dataOffset,
      readRowCount,
      headers.length,
    );
    if (!dataRange) return { headers, rows: [] };

    let allValues: ReturnType<typeof dataRange.getValues>;
    try {
      allValues = dataRange.getValues();
    } catch {
      return { headers, rows: [] };
    }
    if (!allValues) return { headers, rows: [] };

    const rows: (string | number | boolean | null)[][] = [];

    for (let r = 0; r < allValues.length; r++) {
      const rowValues = allValues[r];
      const row: (string | number | boolean | null)[] = [];
      let rowHasContent = false;

      for (let c = 0; c < headers.length; c++) {
        const cellValue = c < rowValues.length ? rowValues[c] : null;

        if (cellValue !== undefined && cellValue !== null && cellValue !== "") {
          rowHasContent = true;
          row.push(cellValue as string | number | boolean);
        } else {
          row.push(null);
        }
      }

      if (!rowHasContent && rows.length >= minRows) {
        break;
      }

      rows.push(row);
    }

    // Trim trailing empty rows
    while (rows.length > 0 && rows[rows.length - 1].every((v) => v === null)) {
      rows.pop();
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
    imageCellRendererRef.current = new ImageCellRenderer(instance.univerAPI);

    // Check if we have multi-sheet data to initialize with
    if (multiSheetData && multiSheetData.sheets.length > 0) {
      isMultiSheetRef.current = true;
      // Store headers for all sheets
      headersMapRef.current.clear();
      for (const sheetData of multiSheetData.sheets) {
        headersMapRef.current.set(sheetData.id, sheetData.data.headers);
      }
      // Compute and store data column offsets per sheet
      for (const sheetData of multiSheetData.sheets) {
        const configs = derivedColumnState?.configs[sheetData.id];
        const offset = computeDataColumnOffset(configs);
        dataColumnOffsetMapRef.current.set(sheetData.id, offset);
      }

      const workbookData = buildMultiSheetWorkbook(
        multiSheetData,
        derivedColumnState?.configs,
        rowConfigs,
        columnConfigs,
      );
      instance.univerAPI.createWorkbook(workbookData);

      // Ensure bold header styling is applied to all sheets after workbook creation
      const initWorkbook = instance.univerAPI.getActiveWorkbook();
      if (initWorkbook) {
        for (const sheetData of multiSheetData.sheets) {
          const sheet = initWorkbook.getSheetBySheetId(sheetData.id);
          if (sheet && sheetData.data.headers.length > 0) {
            const offset =
              dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
            const headerRange = sheet.getRange(
              0,
              offset,
              1,
              sheetData.data.headers.length,
            );
            if (headerRange) {
              headerRange.setFontWeight("bold");
            }
          }
        }
      }

      // Activate the restored sheet if specified
      if (initialActiveSheetId) {
        const wb = instance.univerAPI.getActiveWorkbook();
        if (wb) {
          const targetSheet = wb.getSheetBySheetId(initialActiveSheetId);
          if (targetSheet) {
            wb.setActiveSheet(targetSheet);
            logger.info("Restored active sheet from view state", {
              sheetId: initialActiveSheetId,
            });
          }
        }
      }

      // Apply checkbox validation to boolean columns in all sheets
      const workbook = instance.univerAPI.getActiveWorkbook();
      if (workbook) {
        for (const sheetData of multiSheetData.sheets) {
          const sheet = workbook.getSheetBySheetId(sheetData.id);
          if (sheet) {
            const booleanColumns = detectBooleanColumns(sheetData.data);
            const offset =
              dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
            applyCheckboxValidation(
              instance.univerAPI,
              sheet,
              sheetData.data,
              booleanColumns,
              offset,
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
                const offset =
                  dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
                applyDropdownValidation(
                  instance.univerAPI,
                  sheet,
                  sheetData.data,
                  enumRules,
                  offset,
                );
              }
            }
          } catch (e) {
            logger.debug("Failed to load enum validation rules", {
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
                sheetData.name,
              );
              if (!sortResponse.column || !sortResponse.direction) continue;

              const colIndex = sheetData.data.headers.indexOf(
                sortResponse.column,
              );
              if (colIndex === -1) {
                logger.debug("Persisted sort column not found in headers", {
                  column: sortResponse.column,
                  sheetId: sheetData.id,
                });
                continue;
              }

              const sheet = workbook.getSheetBySheetId(sheetData.id);
              if (!sheet) continue;

              const restoreOffset =
                dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
              const ascending = sortResponse.direction === "ascending";
              sheet.sort(colIndex + restoreOffset, ascending);
              logger.info("Restored sort indicator", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                column: sortResponse.column,
                ascending,
              });
            } catch (e) {
              logger.debug("Failed to restore sort state", {
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
              const offset =
                dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
              applyTableStyle(sheet, sheetData.data, style, offset);
              logger.info("Applied table color scheme", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                colorScheme: style.palette ? "resolved" : "none",
                showRowStripes: style.show_row_stripes,
              });
            }
          } catch (e) {
            logger.debug("Failed to load table style", {
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
              row.map((cell) => (cell === null ? null : cell)),
            );
            const results = await ipc.getConditionalFormatting(
              sheetData.path,
              sheetData.data.headers,
              rowsAsJson,
            );
            if (results.length === 0) continue;

            conditionalFormatsRef.current.set(sheetData.path, results);
            const sheet = workbook.getSheetBySheetId(sheetData.id);
            if (sheet) {
              const offset =
                dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
              applyConditionalFormatting(sheet, results, offset);
              logger.info("Applied conditional formatting", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                matchCount: results.length,
              });
            }
          } catch (e) {
            logger.debug("Failed to load conditional formatting", {
              sheetId: sheetData.id,
              path: sheetData.path,
              error: String(e),
            });
          }
        }
      };
      loadConditionalFormattingAsync();

      // Restore filter state from persisted backend filter state
      const restoreFilterStateAsync = async () => {
        const workbook = instance.univerAPI.getActiveWorkbook();
        if (!workbook) return;

        isRestoringFilterRef.current = true;
        try {
          for (const sheetData of multiSheetData.sheets) {
            try {
              const filterResponse = await ipc.getFilterState(
                sheetData.path,
                sheetData.name,
              );
              if (filterResponse.filters.length === 0) continue;

              logger.info("Restored filter state", {
                sheetId: sheetData.id,
                sheetName: sheetData.name,
                filterCount: filterResponse.filters.length,
              });
            } catch (e) {
              logger.debug("Failed to restore filter state", {
                sheetId: sheetData.id,
                path: sheetData.path,
                error: String(e),
              });
            }
          }
        } finally {
          isRestoringFilterRef.current = false;
        }
      };
      restoreFilterStateAsync();

      // Mark initial load as complete - data is already in cellData, no need to repopulate
      initialLoadCompleteRef.current = true;
      lastMultiSheetDataRef.current = multiSheetData;
      logger.info("Workbook initialized with multiple sheets", {
        sheetCount: multiSheetData.sheets.length,
        sheetNames: multiSheetData.sheets.map((s) => s.name),
      });
    } else {
      // Legacy single-sheet mode: create empty workbook
      instance.univerAPI.createWorkbook({});
      logger.debug("Workbook initialized in single-sheet mode");
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
        logger.debug("Active sheet changed", {
          sheetId,
          sheetName: activeSheet.getSheetName(),
        });
        if (onActiveSheetChangedRef.current) {
          onActiveSheetChangedRef.current(sheetId);
        }
      },
    );

    // Listen for sort events to persist sort state to backend
    const sortDisposable = instance.univerAPI.addEvent(
      instance.univerAPI.Event.SheetRangeSorted,
      (params: {
        worksheet: { getSheetId: () => string };
        sortColumn: { column: number; ascending: boolean }[];
      }) => {
        if (isRestoringSortRef.current) return;

        const sheetId = params.worksheet.getSheetId();
        const sheets = multiSheetDataRef.current?.sheets;
        const sheetData = sheets?.find((s) => s.id === sheetId);
        if (!sheetData) {
          logger.debug("Sort event for unknown sheet", { sheetId });
          return;
        }

        const headers = headersMapRef.current.get(sheetId) ?? [];
        if (params.sortColumn.length === 0) {
          ipc.clearSortState(sheetData.path, sheetData.name).catch((e) => {
            logger.debug("Failed to clear sort state", { error: String(e) });
          });
          logger.info("Sort cleared", { sheetId, sheetName: sheetData.name });
          return;
        }

        const sortSpec = params.sortColumn[0];
        const sortOffset = dataColumnOffsetMapRef.current.get(sheetId) ?? 0;
        const columnName = headers[sortSpec.column - sortOffset];
        if (!columnName) {
          logger.debug("Sort column index out of range", {
            columnIndex: sortSpec.column,
            headerCount: headers.length,
          });
          return;
        }

        const direction: SortDirection = sortSpec.ascending
          ? "ascending"
          : "descending";
        ipc
          .setSortState(sheetData.path, sheetData.name, {
            column: columnName,
            direction,
          })
          .catch((e) => {
            logger.debug("Failed to persist sort state", { error: String(e) });
          });

        logger.info("Sort applied", {
          sheetId,
          sheetName: sheetData.name,
          column: columnName,
          direction,
        });
      },
    );

    // Listen for filter change commands to persist filter state to backend
    const filterDisposable = instance.univerAPI.onCommandExecuted((command) => {
      if (isRestoringFilterRef.current) return;
      if (isLoadingRef.current) return;

      // Intercept filter-related mutations
      if (
        command.id === "sheet.command.set-sheet-filter-range" ||
        command.id === "sheet.command.set-sheet-filter-criteria" ||
        command.id === "sheet.command.remove-sheet-filter" ||
        command.id === "sheet.mutation.set-sheet-filter-range" ||
        command.id === "sheet.mutation.remove-sheet-filter"
      ) {
        const activeSheet = instance.univerAPI
          .getActiveWorkbook()
          ?.getActiveSheet();
        if (!activeSheet) return;

        const sheetId = activeSheet.getSheetId();
        const sheets = multiSheetDataRef.current?.sheets;
        const sheetData = sheets?.find((s) => s.id === sheetId);
        if (!sheetData) {
          logger.debug("Filter event for unknown sheet", {
            sheetId,
            commandId: command.id,
          });
          return;
        }

        if (
          command.id === "sheet.command.remove-sheet-filter" ||
          command.id === "sheet.mutation.remove-sheet-filter"
        ) {
          ipc.clearFilterState(sheetData.path, sheetData.name).catch((e) => {
            logger.debug("Failed to clear filter state", { error: String(e) });
          });
          logger.info("Filter cleared", { sheetId, sheetName: sheetData.name });
        } else {
          logger.info("Filter changed", {
            sheetId,
            sheetName: sheetData.name,
            commandId: command.id,
          });
        }
      }
    });

    // Listen for column width changes to persist to TOML metadata.
    // Handles both drag-resize (set-worksheet-col-width mutation) and
    // double-click auto-fit (set-col-auto-width command).
    const colWidthDebounceTimers = new Map<
      string,
      ReturnType<typeof setTimeout>
    >();

    const persistColumnWidth = (
      filePath: string,
      col: number,
      width: number,
      sheetId: string,
    ) => {
      const headers = headersMapRef.current.get(sheetId) ?? [];
      const offset = dataColumnOffsetMapRef.current.get(sheetId) ?? 0;
      const derivedConfigs = derivedColumnStateRef.current?.configs[sheetId];
      const roundedWidth = Math.round(width);

      // Check if this column is a derived column
      const derivedConfig = derivedConfigs?.find(
        (c) =>
          getDerivedColumnIndex(c, headers.length, derivedConfigs, offset) ===
          col,
      );

      if (derivedConfig) {
        const debounceKey = `${filePath}:derived:${derivedConfig.name}`;
        const existing = colWidthDebounceTimers.get(debounceKey);
        if (existing) clearTimeout(existing);

        colWidthDebounceTimers.set(
          debounceKey,
          setTimeout(() => {
            colWidthDebounceTimers.delete(debounceKey);
            ipc
              .setDerivedColumnWidth(filePath, derivedConfig.name, roundedWidth)
              .catch((e) => {
                logger.debug("Failed to persist derived column width", {
                  columnName: derivedConfig.name,
                  width: roundedWidth,
                  error: String(e),
                });
              });
          }, 300),
        );
        return;
      }

      // Otherwise treat as a data column
      const dataColIndex = col - offset;
      if (dataColIndex < 0 || dataColIndex >= headers.length) return;

      const columnKey = headers[dataColIndex];
      const debounceKey = `${filePath}:${columnKey}`;

      const existing = colWidthDebounceTimers.get(debounceKey);
      if (existing) clearTimeout(existing);

      colWidthDebounceTimers.set(
        debounceKey,
        setTimeout(() => {
          colWidthDebounceTimers.delete(debounceKey);
          ipc.setColumnWidth(filePath, columnKey, roundedWidth).catch((e) => {
            logger.debug("Failed to persist column width", {
              columnKey,
              width: roundedWidth,
              error: String(e),
            });
          });
        }, 300),
      );
    };

    const colWidthDisposable = instance.univerAPI.onCommandExecuted(
      (command) => {
        if (isLoadingRef.current) return;

        if (command.id === "sheet.mutation.set-worksheet-col-width") {
          // Drag-resize: width is in the mutation params
          const activeSheet = instance.univerAPI
            .getActiveWorkbook()
            ?.getActiveSheet();
          if (!activeSheet) return;

          const sheetId = activeSheet.getSheetId();
          const sheets = multiSheetDataRef.current?.sheets;
          const sheetData = sheets?.find((s) => s.id === sheetId);
          if (!sheetData) return;

          const params = command.params as
            | {
                ranges?: Array<{ startColumn: number; endColumn: number }>;
                colWidth?: number;
              }
            | undefined;
          if (!params?.ranges || params.colWidth === undefined) return;

          for (const range of params.ranges) {
            for (let col = range.startColumn; col <= range.endColumn; col++) {
              persistColumnWidth(sheetData.path, col, params.colWidth, sheetId);
            }
          }
        } else if (command.id === "sheet.command.set-col-auto-width") {
          // Double-click auto-fit: read resulting widths from the sheet
          const activeSheet = instance.univerAPI
            .getActiveWorkbook()
            ?.getActiveSheet();
          if (!activeSheet) return;

          const sheetId = activeSheet.getSheetId();
          const sheets = multiSheetDataRef.current?.sheets;
          const sheetData = sheets?.find((s) => s.id === sheetId);
          if (!sheetData) return;

          const params = command.params as
            | {
                ranges?: Array<{ startColumn: number; endColumn: number }>;
              }
            | undefined;
          if (!params?.ranges) return;

          for (const range of params.ranges) {
            for (let col = range.startColumn; col <= range.endColumn; col++) {
              const width = activeSheet.getColumnWidth(col);
              persistColumnWidth(sheetData.path, col, width, sheetId);
            }
          }
        }
      },
    );

    // Listen for derived value computed events to handle image results
    const derivedValueSub = ipc.onDerivedValueComputed((payload) => {
      const result = payload.result;
      if (!result || result.type !== "image") return;

      const workbook = instance.univerAPI.getActiveWorkbook();
      if (!workbook) return;

      // Find the sheet matching the derived value's file path
      const matchingSheetData = multiSheetData?.sheets.find(
        (s) => s.path === payload.file_path,
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

      // Find the column index for the derived function from config position
      const currentDerivedState = derivedColumnStateRef.current;
      const sheetConfigs = currentDerivedState?.configs[sheetId];
      const derivedConfig = sheetConfigs?.find(
        (c) => c.function === payload.function_name,
      );
      if (!derivedConfig) {
        logger.debug("Image derived function config not found", {
          functionName: payload.function_name,
          sheetId,
        });
        return;
      }
      const headers = headersMapRef.current.get(sheetId) ?? headersRef.current;
      const colOffset = dataColumnOffsetMapRef.current.get(sheetId) ?? 0;
      const colIdx = getDerivedColumnIndex(
        derivedConfig,
        headers.length,
        sheetConfigs!,
        colOffset,
      );

      const currentRowConfigs = rowConfigsRef.current;
      const sheetRowConfig = currentRowConfigs?.[sheetId];
      const rowHeight = sheetRowConfig?.default_height ?? undefined;

      renderer
        .handleImageResult(
          sheet,
          sheetId,
          displayRow,
          colIdx,
          result,
          rowHeight,
        )
        .catch((e) => {
          logger.debug("Image render from derived value failed", {
            error: String(e),
            rowIndex: payload.row_index,
            functionName: payload.function_name,
          });
        });
    });

    return () => {
      activeSheetDisposable.dispose();
      sortDisposable.dispose();
      filterDisposable.dispose();
      colWidthDisposable.dispose();
      for (const timer of colWidthDebounceTimers.values()) {
        clearTimeout(timer);
      }
      colWidthDebounceTimers.clear();
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
      // Set headers row using batch operation with display-formatted names
      const headerRange = sheet.getRange(0, 0, 1, numColumns);
      if (headerRange) {
        headerRange.setValues([data.headers.map(formatHeaderForDisplay)]);
        headerRange.setFontWeight("bold");
      }

      // Set data rows using a single batch operation
      if (data.rows.length > 0) {
        const dataRange = sheet.getRange(1, 0, data.rows.length, numColumns);
        if (dataRange) {
          // Convert null values to empty strings and booleans to 1/0 for display
          const displayRows = data.rows.map((row) =>
            row.map((cellValue) => {
              if (cellValue === null) return "";
              if (typeof cellValue === "boolean") return cellValue ? 1 : 0;
              return cellValue;
            }),
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
    if (
      initialLoadCompleteRef.current &&
      lastMultiSheetDataRef.current === multiSheetData
    ) {
      logger.debug(
        "Skipping multi-sheet update - same data reference as initial load",
      );
      return;
    }

    isLoadingRef.current = true;
    isMultiSheetRef.current = true;

    // Update headers map and data column offsets for all sheets
    headersMapRef.current.clear();
    for (const sheetData of multiSheetData.sheets) {
      headersMapRef.current.set(sheetData.id, sheetData.data.headers);
      const configs = derivedColumnStateRef.current?.configs[sheetData.id];
      dataColumnOffsetMapRef.current.set(
        sheetData.id,
        computeDataColumnOffset(configs),
      );
    }

    // Find sheets that actually changed (for reload optimization)
    const previousData = lastMultiSheetDataRef.current;
    const changedSheetIds = new Set<string>();

    if (previousData) {
      for (const sheetData of multiSheetData.sheets) {
        const prevSheet = previousData.sheets.find(
          (s) => s.id === sheetData.id,
        );
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
        logger.debug("Skipping unchanged sheet", {
          sheetId: sheetData.id,
          sheetName: sheetData.name,
        });
        continue;
      }

      const sheet = workbook.getSheetBySheetId(sheetData.id);
      if (!sheet) {
        logger.debug("Sheet not found, skipping update", {
          sheetId: sheetData.id,
        });
        continue;
      }

      // Clear existing images for this sheet before repopulating data
      if (imageCellRendererRef.current) {
        imageCellRendererRef.current.clearSheetImages(sheet, sheetData.id);
      }

      const sheetOffset = dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;

      logger.debug("Updating changed sheet with batch operation", {
        sheetId: sheetData.id,
        sheetName: sheetData.name,
        rowCount: sheetData.data.rows.length,
        columnCount: sheetData.data.headers.length,
        dataOffset: sheetOffset,
      });
      populateSheetDataBatch(sheet, sheetData.data, sheetOffset);

      // Apply checkbox validation to boolean columns after updating data
      const booleanColumns = detectBooleanColumns(sheetData.data);
      applyCheckboxValidation(
        univerAPIRef.current!,
        sheet,
        sheetData.data,
        booleanColumns,
        sheetOffset,
      );

      // Apply dropdown validation from cached enum rules
      const cachedEnumRules = enumRulesRef.current.get(sheetData.path);
      if (cachedEnumRules && cachedEnumRules.length > 0) {
        applyDropdownValidation(
          univerAPIRef.current!,
          sheet,
          sheetData.data,
          cachedEnumRules,
          sheetOffset,
        );
      }

      // Re-apply cached table style after data update
      const cachedStyle = tableStylesRef.current.get(sheetData.path);
      if (cachedStyle) {
        applyTableStyle(sheet, sheetData.data, cachedStyle, sheetOffset);
      }

      // Re-evaluate and apply conditional formatting after data update
      const reevaluateConditionalFormatting = async () => {
        try {
          const rowsAsJson = sheetData.data.rows.map((row) =>
            row.map((cell) => (cell === null ? null : cell)),
          );
          const results = await ipc.getConditionalFormatting(
            sheetData.path,
            sheetData.data.headers,
            rowsAsJson,
          );
          conditionalFormatsRef.current.set(sheetData.path, results);
          const ws = workbook.getSheetBySheetId(sheetData.id);
          if (ws && results.length > 0) {
            const cfOffset =
              dataColumnOffsetMapRef.current.get(sheetData.id) ?? 0;
            applyConditionalFormatting(ws, results, cfOffset);
          }
        } catch (e) {
          logger.debug("Failed to re-evaluate conditional formatting", {
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

    for (const [sheetId, configs] of Object.entries(
      derivedColumnState.configs,
    )) {
      if (configs.length === 0) continue;

      const sheet = workbook.getSheetBySheetId(sheetId);
      if (!sheet) continue;

      const headers = headersMapRef.current.get(sheetId) ?? headersRef.current;
      if (headers.length === 0) continue;

      const sheetValues = derivedColumnState.values[sheetId];
      if (!sheetValues) continue;

      isLoadingRef.current = true;

      const derivedOffset = dataColumnOffsetMapRef.current.get(sheetId) ?? 0;

      for (const config of configs) {
        const derivedColIndex = getDerivedColumnIndex(
          config,
          headers.length,
          configs,
          derivedOffset,
        );

        // Set header for derived column (row 0)
        const headerRange = sheet.getRange(0, derivedColIndex, 1, 1);
        if (headerRange) {
          headerRange.setValues([[formatHeaderForDisplay(config.name)]]);
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
            for (
              let rowIndex = 0;
              rowIndex < sheetData.data.rows.length;
              rowIndex++
            ) {
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
