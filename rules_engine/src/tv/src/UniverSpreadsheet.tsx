import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Univer, RichTextValue } from "@univerjs/core";
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
import {
  isSheetDataEqual,
  populateSheetDataBatch,
} from "./sheet_data_utils";
import {
  detectBooleanColumns,
  applyCheckboxValidation,
  applyDropdownValidation,
} from "./validation_utils";
import {
  getDerivedColumnIndex,
  buildColumnMapping,
  applyDerivedResultToCell,
  EMPTY_MAPPING,
  type ColumnMapping,
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
    onSheetOrderChanged,
    derivedColumnState,
    initialActiveSheetId,
    rowConfigs,
    columnConfigs,
    persistedSheetOrder,
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
  const onSheetOrderChangedRef = useRef(onSheetOrderChanged);
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
  const columnMappingRef = useRef<Map<string, ColumnMapping>>(new Map());
  const originalToDisplayMapRef = useRef<Map<string, Map<number, number>>>(
    new Map(),
  );
  const sortPendingRef = useRef(false);

  onChangeRef.current = onChange;
  onActiveSheetChangedRef.current = onActiveSheetChanged;
  onSheetOrderChangedRef.current = onSheetOrderChanged;

  /**
   * Extract data from a specific sheet or the active sheet.
   * Reads a range covering all data columns and extracts values
   * using the column mapping to handle non-contiguous layouts.
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

    const mapping =
      columnMappingRef.current.get(currentSheetId) ?? EMPTY_MAPPING;
    if (mapping.dataToVisual.length === 0) return { headers, rows: [] };

    const maxRows = sheet.getMaxRows();

    // Read all data rows in a single batch call instead of cell-by-cell.
    // Row 0 is the header row, data starts at row 1.
    const readRowCount = maxRows - 1;
    if (readRowCount <= 0) return { headers, rows: [] };

    const firstDataCol = mapping.dataToVisual[0];
    const lastDataCol = mapping.dataToVisual[mapping.dataToVisual.length - 1];
    const rangeWidth = lastDataCol - firstDataCol + 1;

    const dataRange = sheet.getRange(
      1,
      firstDataCol,
      readRowCount,
      rangeWidth,
    );
    if (!dataRange) return { headers, rows: [] };

    // Use includeRichText=true so cells with document data (p) return
    // RichTextValue objects instead of null.
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let allValues: any[][];
    try {
      allValues = dataRange.getValues(true);
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
        const rangeRelCol = mapping.dataToVisual[c] - firstDataCol;
        const rawCellValue =
          rangeRelCol >= 0 && rangeRelCol < rowValues.length
            ? rowValues[rangeRelCol]
            : null;

        let cellValue: string | number | boolean | null = null;
        if (rawCellValue instanceof RichTextValue) {
          const text = rawCellValue.toPlainText().replace(/\r\n/g, "\n").replace(/\r/g, "\n");
          if (text) {
            cellValue = text;
          }
        } else if (
          rawCellValue !== undefined &&
          rawCellValue !== null &&
          rawCellValue !== ""
        ) {
          cellValue = rawCellValue as string | number | boolean;
        }

        if (cellValue !== null) {
          rowHasContent = true;
          row.push(cellValue);
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

    const buildMappingFromGrid = (
      sheetId: string,
    ): Map<number, number> | null => {
      const wb = instance.univerAPI.getActiveWorkbook();
      if (!wb) return null;
      const ws = wb.getSheetBySheetId(sheetId);
      if (!ws) return null;

      const sheets = multiSheetDataRef.current?.sheets;
      const sheetData = sheets?.find((s) => s.id === sheetId);
      if (!sheetData) return null;

      const headers = headersMapRef.current.get(sheetId) ?? [];
      const mapping =
        columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;
      const originalRows = sheetData.data.rows;
      if (
        originalRows.length === 0 ||
        headers.length === 0 ||
        mapping.dataToVisual.length === 0
      )
        return null;

      const normalize = (val: unknown): string => {
        if (val === null || val === undefined || val === "") return "";
        if (typeof val === "boolean") return val ? "1" : "0";
        return String(val);
      };

      const firstDataCol = mapping.dataToVisual[0];
      const lastDataCol =
        mapping.dataToVisual[mapping.dataToVisual.length - 1];
      const rangeWidth = lastDataCol - firstDataCol + 1;

      const dataRange = ws.getRange(
        1,
        firstDataCol,
        originalRows.length,
        rangeWidth,
      );
      if (!dataRange) return null;

      let gridValues: ReturnType<typeof dataRange.getValues>;
      try {
        gridValues = dataRange.getValues();
      } catch {
        return null;
      }
      if (!gridValues) return null;

      const displayFingerprints: string[] = [];
      for (let d = 0; d < gridValues.length; d++) {
        const row = gridValues[d];
        let fp = "";
        for (let c = 0; c < headers.length; c++) {
          const rangeRelCol = mapping.dataToVisual[c] - firstDataCol;
          fp += `|${normalize(rangeRelCol >= 0 && rangeRelCol < row.length ? row[rangeRelCol] : null)}`;
        }
        displayFingerprints.push(fp);
      }

      const originalFingerprints: string[] = originalRows.map((row) => {
        let fp = "";
        for (let c = 0; c < headers.length; c++) {
          fp += `|${normalize(c < row.length ? row[c] : null)}`;
        }
        return fp;
      });

      const originalToDisplay = new Map<number, number>();
      const usedOriginal = new Set<number>();

      for (let d = 0; d < displayFingerprints.length; d++) {
        for (let o = 0; o < originalFingerprints.length; o++) {
          if (usedOriginal.has(o)) continue;
          if (displayFingerprints[d] === originalFingerprints[o]) {
            originalToDisplay.set(o, d);
            usedOriginal.add(o);
            break;
          }
        }
      }

      return originalToDisplay.size > 0 ? originalToDisplay : null;
    };

    const reinsertImagesForSheet = async (sheetId: string) => {
      const renderer = imageCellRendererRef.current;
      if (!renderer) return;

      const wb = instance.univerAPI.getActiveWorkbook();
      if (!wb) return;
      const ws = wb.getSheetBySheetId(sheetId);
      if (!ws) return;

      await renderer.clearSheetImages(ws, sheetId);

      const inverseMap = originalToDisplayMapRef.current.get(sheetId);
      const currentDerivedState = derivedColumnStateRef.current;
      const sheetValues = currentDerivedState?.values[sheetId];
      const sheetConfigs = currentDerivedState?.configs[sheetId];
      if (!sheetValues || !sheetConfigs) return;

      const mapping =
        columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;
      const sheetRowConfig = rowConfigsRef.current?.[sheetId];
      const rowHeight = sheetRowConfig?.default_height ?? undefined;

      for (const [rowIndexStr, rowValues] of Object.entries(sheetValues)) {
        const originalRowIndex = parseInt(rowIndexStr, 10);
        const displayIndex =
          inverseMap?.get(originalRowIndex) ?? originalRowIndex;
        const displayRow = displayIndex + 1;

        for (const config of sheetConfigs) {
          const result = rowValues[config.function];
          if (!result || result.type !== "image") continue;

          const colIdx = getDerivedColumnIndex(
            config,
            sheetConfigs,
            mapping,
          );
          renderer
            .handleImageResult(
              ws,
              sheetId,
              displayRow,
              colIdx,
              result,
              rowHeight,
            )
            .catch((e) => {
              logger.debug("Image re-insert after sort failed", {
                error: String(e),
              });
            });
        }
      }
    };

    // Check if we have multi-sheet data to initialize with
    if (multiSheetData && multiSheetData.sheets.length > 0) {
      isMultiSheetRef.current = true;
      // Store headers for all sheets
      headersMapRef.current.clear();
      for (const sheetData of multiSheetData.sheets) {
        headersMapRef.current.set(sheetData.id, sheetData.data.headers);
      }
      // Compute and store column mappings per sheet
      for (const sheetData of multiSheetData.sheets) {
        const configs = derivedColumnState?.configs[sheetData.id];
        columnMappingRef.current.set(
          sheetData.id,
          buildColumnMapping(configs, sheetData.data.headers.length),
        );
      }

      const workbookData = buildMultiSheetWorkbook(
        multiSheetData,
        derivedColumnState?.configs,
        rowConfigs,
        columnConfigs,
        persistedSheetOrder,
      );
      instance.univerAPI.createWorkbook(workbookData);

      // Ensure bold header styling is applied to all sheets after workbook creation
      const initWorkbook = instance.univerAPI.getActiveWorkbook();
      if (initWorkbook) {
        for (const sheetData of multiSheetData.sheets) {
          const sheet = initWorkbook.getSheetBySheetId(sheetData.id);
          if (sheet && sheetData.data.headers.length > 0) {
            const mapping =
              columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
            const firstCol =
              mapping.dataToVisual.length > 0 ? mapping.dataToVisual[0] : 0;
            const lastCol =
              mapping.dataToVisual.length > 0
                ? mapping.dataToVisual[mapping.dataToVisual.length - 1]
                : 0;
            const headerRange = sheet.getRange(
              0,
              firstCol,
              1,
              lastCol - firstCol + 1,
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
            const mapping =
              columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
            applyCheckboxValidation(
              instance.univerAPI,
              sheet,
              sheetData.data,
              booleanColumns,
              mapping,
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
                const mapping =
                  columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
                applyDropdownValidation(
                  instance.univerAPI,
                  sheet,
                  sheetData.data,
                  enumRules,
                  mapping,
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

              const mapping =
                columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
              const visualCol = mapping.dataToVisual[colIndex];
              if (visualCol === undefined) continue;
              const ascending = sortResponse.direction === "ascending";
              sheet.sort(visualCol, ascending);
              const restoreMapping = buildMappingFromGrid(sheetData.id);
              if (restoreMapping) {
                originalToDisplayMapRef.current.set(
                  sheetData.id,
                  restoreMapping,
                );
              }
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
              const mapping =
                columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
              applyTableStyle(sheet, sheetData.data, style, mapping);
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
              const mapping =
                columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
              applyConditionalFormatting(sheet, results, mapping);
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

      // Restore persisted filter criteria for each sheet.
      // The filter range itself is set via workbook resources in
      // buildMultiSheetWorkbook, so dropdown arrows are already present.
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
              if (
                !filterResponse.active ||
                filterResponse.filters.length === 0
              )
                continue;

              const sheet = workbook.getSheetBySheetId(sheetData.id);
              if (!sheet) continue;

              const filter = sheet.getFilter();
              if (!filter) continue;

              const headers = sheetData.data.headers;
              const mapping =
                columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;

              for (const savedFilter of filterResponse.filters) {
                const colIndex = headers.indexOf(savedFilter.column);
                if (colIndex === -1) continue;

                const visualCol = mapping.dataToVisual[colIndex];
                if (visualCol === undefined) continue;

                if ("values" in savedFilter.condition) {
                  const values = savedFilter.condition.values as unknown[];
                  filter.setColumnFilterCriteria(visualCol, {
                    colId: colIndex,
                    filters: {
                      filters: values.map((v) => String(v)),
                    },
                  });
                }
              }

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

    // Listen for cell value changes and row removal
    instance.univerAPI.onCommandExecuted((command) => {
      if (isLoadingRef.current) return;
      if (
        command.id === "sheet.mutation.set-range-values" ||
        command.id === "sheet.command.set-range-values" ||
        command.id === "sheet.mutation.remove-rows" ||
        command.id === "sheet.command.remove-row"
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

    // Listen for sheet tab reorder (drag) events to persist sheet order
    const sheetMovedDisposable = instance.univerAPI.addEvent(
      instance.univerAPI.Event.SheetMoved,
      () => {
        // Read the current sheet order from the workbook after the move
        const workbook = instance.univerAPI.getActiveWorkbook();
        if (!workbook) return;

        const sheets = multiSheetDataRef.current?.sheets;
        if (!sheets) return;

        // Get the workbook's sheetOrder array which reflects the new tab order.
        // The facade getSheets() method returns sheets in display order.
        const orderedSheets = workbook.getSheets();
        const sheetNames: string[] = [];
        for (const ws of orderedSheets) {
          const sheetId = ws.getSheetId();
          const sheetData = sheets.find((s) => s.id === sheetId);
          if (sheetData) {
            sheetNames.push(sheetData.name);
          }
        }

        logger.info("Sheet tabs reordered", { order: sheetNames });
        if (onSheetOrderChangedRef.current) {
          onSheetOrderChangedRef.current(sheetNames);
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
          originalToDisplayMapRef.current.delete(sheetId);
          sortPendingRef.current = true;
          reinsertImagesForSheet(sheetId).finally(() => {
            sortPendingRef.current = false;
          });
          logger.info("Sort cleared", { sheetId, sheetName: sheetData.name });
          return;
        }

        const sortSpec = params.sortColumn[0];
        const mapping =
          columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;
        const dataColIndex = mapping.visualToData.get(sortSpec.column);
        const columnName =
          dataColIndex !== undefined ? headers[dataColIndex] : undefined;
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
        const gridMapping = buildMappingFromGrid(sheetId);
        if (gridMapping) {
          originalToDisplayMapRef.current.set(sheetId, gridMapping);
        } else {
          originalToDisplayMapRef.current.delete(sheetId);
        }

        ipc
          .setSortState(sheetData.path, sheetData.name, {
            column: columnName,
            direction,
          })
          .catch((e) => {
            logger.debug("Failed to persist sort state", {
              error: String(e),
            });
          });

        sortPendingRef.current = true;
        reinsertImagesForSheet(sheetId).finally(() => {
          sortPendingRef.current = false;
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
          logger.info("Filter cleared", {
            sheetId,
            sheetName: sheetData.name,
          });

          // Re-create filter so dropdown arrows remain visible.
          // Use setTimeout to defer until after the current command finishes.
          const capturedSheetId = sheetId;
          setTimeout(() => {
            isRestoringFilterRef.current = true;
            try {
              const wb = instance.univerAPI.getActiveWorkbook();
              if (!wb) return;
              const ws = wb.getSheetBySheetId(capturedSheetId);
              if (!ws) return;
              const headers =
                headersMapRef.current.get(capturedSheetId) ?? [];
              const mapping =
                columnMappingRef.current.get(capturedSheetId) ?? EMPTY_MAPPING;
              const currentSheetData =
                multiSheetDataRef.current?.sheets.find(
                  (s) => s.id === capturedSheetId,
                );
              const rowCount = currentSheetData?.data.rows.length ?? 0;
              if (
                headers.length > 0 &&
                mapping.dataToVisual.length > 0
              ) {
                const firstCol = mapping.dataToVisual[0];
                const lastCol =
                  mapping.dataToVisual[mapping.dataToVisual.length - 1];
                const filterRange = ws.getRange(
                  0,
                  firstCol,
                  rowCount + 1,
                  lastCol - firstCol + 1,
                );
                if (filterRange) {
                  filterRange.createFilter();
                }
              }
            } finally {
              isRestoringFilterRef.current = false;
            }
          }, 0);
        } else if (
          command.id === "sheet.command.set-sheet-filter-criteria"
        ) {
          // Read all column filter criteria and persist to backend
          const filter = activeSheet.getFilter();
          if (!filter) return;

          const headers = headersMapRef.current.get(sheetId) ?? [];
          const mapping =
            columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;

          const filterRequests: ipc.ColumnFilterRequest[] = [];
          for (let i = 0; i < headers.length; i++) {
            const visualCol = mapping.dataToVisual[i];
            if (visualCol === undefined) continue;
            const criteria = filter.getColumnFilterCriteria(visualCol);
            if (!criteria) continue;
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            const filtersObj = (criteria as any).filters;
            if (!filtersObj?.filters || filtersObj.filters.length === 0)
              continue;
            filterRequests.push({
              column: headers[i],
              condition: {
                values: filtersObj.filters.map((v: string) => {
                  // Try to parse as number or boolean
                  if (v === "true") return true;
                  if (v === "false") return false;
                  const n = Number(v);
                  if (!isNaN(n) && v !== "") return n;
                  return v;
                }),
              },
            });
          }

          ipc
            .setFilterState(sheetData.path, sheetData.name, {
              filters: filterRequests,
              active: filterRequests.length > 0,
            })
            .catch((e) => {
              logger.debug("Failed to persist filter state", {
                error: String(e),
              });
            });

          logger.info("Filter criteria persisted", {
            sheetId,
            sheetName: sheetData.name,
            filterCount: filterRequests.length,
          });
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
      const mapping =
        columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;
      const derivedConfigs = derivedColumnStateRef.current?.configs[sheetId];
      const roundedWidth = Math.round(width);

      // Check if this column is a derived column
      const derivedConfig = derivedConfigs?.find(
        (c) =>
          getDerivedColumnIndex(c, derivedConfigs, mapping) === col,
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

      // Otherwise treat as a data column via reverse mapping
      const dataColIndex = mapping.visualToData.get(col);
      if (dataColIndex === undefined || dataColIndex >= headers.length) return;

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
      if (sortPendingRef.current) return;

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

      // Translate data row index to display row using sort mapping
      const inverseMap = originalToDisplayMapRef.current.get(sheetId);
      const displayIndex =
        inverseMap?.get(payload.row_index) ?? payload.row_index;
      const displayRow = displayIndex + 1;

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
      const mapping =
        columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;
      const colIdx = getDerivedColumnIndex(
        derivedConfig,
        sheetConfigs!,
        mapping,
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
      sheetMovedDisposable.dispose();
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
      applyCheckboxValidation(univerAPI, sheet, data, booleanColumns, EMPTY_MAPPING);
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

    // Update headers map and column mappings for all sheets
    headersMapRef.current.clear();
    for (const sheetData of multiSheetData.sheets) {
      headersMapRef.current.set(sheetData.id, sheetData.data.headers);
      const configs = derivedColumnStateRef.current?.configs[sheetData.id];
      columnMappingRef.current.set(
        sheetData.id,
        buildColumnMapping(configs, sheetData.data.headers.length),
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

      const sheetMapping =
        columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;

      logger.debug("Updating changed sheet with batch operation", {
        sheetId: sheetData.id,
        sheetName: sheetData.name,
        rowCount: sheetData.data.rows.length,
        columnCount: sheetData.data.headers.length,
      });
      populateSheetDataBatch(sheet, sheetData.data, sheetMapping);

      // Apply bold styling to columns configured with bold = true
      const sheetColConfigs = columnConfigsRef.current?.[sheetData.id];
      if (sheetColConfigs) {
        for (const colConfig of sheetColConfigs) {
          if (colConfig.bold) {
            const headerIndex = sheetData.data.headers.indexOf(colConfig.key);
            if (headerIndex !== -1 && sheetData.data.rows.length > 0) {
              const visualCol = sheetMapping.dataToVisual[headerIndex];
              if (visualCol !== undefined) {
                const boldRange = sheet.getRange(
                  1,
                  visualCol,
                  sheetData.data.rows.length,
                  1,
                );
                if (boldRange) {
                  boldRange.setFontWeight("bold");
                }
              }
            }
          }
        }
      }

      // Apply checkbox validation to boolean columns after updating data
      const booleanColumns = detectBooleanColumns(sheetData.data);
      applyCheckboxValidation(
        univerAPIRef.current!,
        sheet,
        sheetData.data,
        booleanColumns,
        sheetMapping,
      );

      // Apply dropdown validation from cached enum rules
      const cachedEnumRules = enumRulesRef.current.get(sheetData.path);
      if (cachedEnumRules && cachedEnumRules.length > 0) {
        applyDropdownValidation(
          univerAPIRef.current!,
          sheet,
          sheetData.data,
          cachedEnumRules,
          sheetMapping,
        );
      }

      // Re-apply cached table style after data update
      const cachedStyle = tableStylesRef.current.get(sheetData.path);
      if (cachedStyle) {
        applyTableStyle(sheet, sheetData.data, cachedStyle, sheetMapping);
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
            const cfMapping =
              columnMappingRef.current.get(sheetData.id) ?? EMPTY_MAPPING;
            applyConditionalFormatting(ws, results, cfMapping);
          }
        } catch (e) {
          logger.debug("Failed to re-evaluate conditional formatting", {
            sheetId: sheetData.id,
            error: String(e),
          });
        }
      };
      reevaluateConditionalFormatting();

      // Re-create auto-filter so dropdown arrows survive data reload
      if (
        sheetData.data.headers.length > 0 &&
        sheetMapping.dataToVisual.length > 0
      ) {
        const existingFilter = sheet.getFilter();
        if (!existingFilter) {
          try {
            const firstCol = sheetMapping.dataToVisual[0];
            const lastCol =
              sheetMapping.dataToVisual[sheetMapping.dataToVisual.length - 1];
            const filterRange = sheet.getRange(
              0,
              firstCol,
              sheetData.data.rows.length + 1,
              lastCol - firstCol + 1,
            );
            if (filterRange) {
              filterRange.createFilter();
            }
          } catch (e) {
            logger.debug("Failed to re-create auto-filter on data reload", {
              sheetId: sheetData.id,
              error: String(e),
            });
          }
        }
      }
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

      const mapping =
        columnMappingRef.current.get(sheetId) ?? EMPTY_MAPPING;

      for (const config of configs) {
        const derivedColIndex = getDerivedColumnIndex(
          config,
          configs,
          mapping,
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
          applyDerivedResultToCell(
            sheet,
            cellRow,
            derivedColIndex,
            result,
          );
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
