import { useCallback, useEffect, useRef, useState } from "react";
import { SpreadsheetView } from "./spreadsheet_view";
import { ErrorBanner } from "./error_banner";
import * as ipc from "./ipc_bridge";
import type { TomlTableData, DerivedValuePayload, RowConfig, ColumnConfig } from "./ipc_bridge";
import type { MultiSheetData, SheetData, DerivedColumnState } from "./spreadsheet_types";
import { createLogger } from "./logger_frontend";

export type { TomlTableData };

const logger = createLogger("tv.ui.app");

const SAVE_DEBOUNCE_MS = 500;

function isBooleanNumericEqual(
  a: string | number | boolean | null,
  b: string | number | boolean | null
): boolean {
  if (typeof a === "boolean" && typeof b === "number") {
    return (a === true && b === 1) || (a === false && b === 0);
  }
  if (typeof a === "number" && typeof b === "boolean") {
    return (a === 1 && b === true) || (a === 0 && b === false);
  }
  return false;
}

/**
 * Compares two TomlTableData objects for equality.
 * Returns true if headers and all cell values are identical.
 */
function isDataEqual(a: TomlTableData, b: TomlTableData): boolean {
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
      const cellA = rowA[colIdx];
      const cellB = rowB[colIdx];
      // Compare values, treating null and undefined as equal,
      // and boolean/numeric equivalence (true===1, false===0) for checkbox cells
      if (cellA !== cellB) {
        if ((cellA === null || cellA === undefined) && (cellB === null || cellB === undefined)) {
          continue;
        }
        if (isBooleanNumericEqual(cellA, cellB)) {
          continue;
        }
        return false;
      }
    }
  }

  return true;
}

/**
 * Returns row indices where oldData and newData differ.
 * Used to trigger derived computations only for changed rows after save.
 */
function getChangedRowIndices(
  oldData: TomlTableData,
  newData: TomlTableData
): number[] {
  const changed: number[] = [];
  const maxRows = Math.max(oldData.rows.length, newData.rows.length);
  for (let i = 0; i < maxRows; i++) {
    const oldRow = oldData.rows[i];
    const newRow = newData.rows[i];
    if (!oldRow || !newRow) {
      changed.push(i);
      continue;
    }
    const maxCols = Math.max(oldRow.length, newRow.length);
    let rowChanged = false;
    for (let j = 0; j < maxCols; j++) {
      const a = j < oldRow.length ? (oldRow[j] ?? null) : null;
      const b = j < newRow.length ? (newRow[j] ?? null) : null;
      if (a !== b) {
        if ((a === null || a === undefined) && (b === null || b === undefined)) {
          continue;
        }
        if (isBooleanNumericEqual(a, b)) {
          continue;
        }
        rowChanged = true;
        break;
      }
    }
    if (rowChanged) {
      changed.push(i);
    }
  }
  return changed;
}

interface SheetInfo {
  id: string;
  path: string;
  tableName: string;
}

function extractTableName(filePath: string): string {
  const fileName = filePath.split("/").pop() || filePath;
  return fileName.replace(/\.toml$/, "");
}

function generateSheetId(filePath: string): string {
  return `sheet-${filePath.replace(/[^a-zA-Z0-9]/g, "-")}`;
}

export function AppRoot() {
  const [sheets, setSheets] = useState<SheetInfo[]>([]);
  const [multiSheetData, setMultiSheetData] = useState<MultiSheetData | null>(null);
  const [activeSheetId, setActiveSheetId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [derivedColumnState, setDerivedColumnState] = useState<DerivedColumnState>({
    configs: {},
    values: {},
  });
  const [rowConfigs, setRowConfigs] = useState<Record<string, RowConfig>>({});
  const [columnConfigs, setColumnConfigs] = useState<Record<string, ColumnConfig[]>>({});
  const saveTimeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef<Record<string, boolean>>({});
  const watchersStartedRef = useRef<Set<string>>(new Set());
  // Track last known data for each sheet to detect actual changes
  const lastKnownDataRef = useRef<Record<string, TomlTableData>>({});
  // Track save completion timestamps to suppress self-triggered file watcher reloads
  const lastSaveTimeRef = useRef<Record<string, number>>({});
  // Ref to track activeSheetId without causing effect re-runs
  const activeSheetIdRef = useRef<string | null>(null);
  activeSheetIdRef.current = activeSheetId;

  const loadSingleFile = useCallback(async (path: string, tableName: string): Promise<TomlTableData | null> => {
    try {
      return await ipc.loadTomlTable(path, tableName);
    } catch (e) {
      const msg = String(e);
      if (msg.includes("not found") || msg.includes("not an array of tables")) {
        logger.warn("Skipping incompatible file", { path, error: msg });
      } else {
        logger.error("Failed to load file", { path, error: msg });
      }
      return null;
    }
  }, []);

  const triggerDerivedComputations = useCallback(async (
    sheetInfo: SheetInfo,
    data: TomlTableData,
    configs: ipc.DerivedColumnInfo[],
    rowIndices?: number[],
  ) => {
    try {
      if (configs.length === 0) return;

      const indicesToCompute = rowIndices
        ?? Array.from({ length: data.rows.length }, (_, i) => i);

      const requests: ipc.ComputeDerivedRequest[] = [];
      for (const rowIndex of indicesToCompute) {
        if (rowIndex >= data.rows.length) continue;

        const rowData: Record<string, unknown> = {};
        for (let colIdx = 0; colIdx < data.headers.length; colIdx++) {
          rowData[data.headers[colIdx]] = data.rows[rowIndex][colIdx];
        }

        for (const config of configs) {
          if (config.url_template) {
            rowData["__url_template"] = config.url_template;
          }
          requests.push({
            file_path: sheetInfo.path,
            table_name: sheetInfo.tableName,
            row_index: rowIndex,
            function_name: config.function,
            row_data: rowData,
            is_visible: rowIndex < 50,
          });
        }
      }

      if (requests.length > 0) {
        await ipc.computeDerivedBatch({ requests });
      }
    } catch (e) {
      logger.error("Failed to trigger derived computations", { path: sheetInfo.path, error: String(e) });
    }
  }, []);

  const loadAllFiles = useCallback(async (sheetInfos: SheetInfo[], restoredSheetId?: string): Promise<void> => {
    setLoading(true);
    const loadPromises = sheetInfos.map(async (info): Promise<SheetData | null> => {
      const data = await loadSingleFile(info.path, info.tableName);
      if (data) {
        return {
          id: info.id,
          name: info.tableName,
          path: info.path,
          data,
        };
      }
      return null;
    });

    const results = await Promise.all(loadPromises);
    const validSheets = results.filter((s): s is SheetData => s !== null);

    if (validSheets.length === 0) {
      setError("Failed to load any TOML files");
      setLoading(false);
      return;
    }

    // Store last known data for change detection
    for (const sheet of validSheets) {
      lastKnownDataRef.current[sheet.id] = sheet.data;
    }

    // Pre-load derived column configs and row configs before building
    // the workbook so they are available during initial workbook creation.
    const allDerivedConfigs: Record<string, ipc.DerivedColumnInfo[]> = {};
    const allRowConfigs: Record<string, RowConfig> = {};
    const allColumnConfigs: Record<string, ColumnConfig[]> = {};
    await Promise.all(
      validSheets.map(async (sheet) => {
        try {
          const configs = await ipc.getDerivedColumnsConfig(sheet.path);
          if (configs.length > 0) {
            allDerivedConfigs[sheet.id] = configs;
          }
        } catch (e) {
          logger.error("Failed to load derived columns", { path: sheet.path, error: String(e) });
        }
        try {
          const rowConfig = await ipc.getRowConfig(sheet.path);
          if (rowConfig) {
            allRowConfigs[sheet.id] = rowConfig;
          }
        } catch (e) {
          logger.error("Failed to load row config", { path: sheet.path, error: String(e) });
        }
        try {
          const colConfigs = await ipc.getColumnConfigs(sheet.path);
          if (colConfigs.length > 0) {
            allColumnConfigs[sheet.id] = colConfigs;
          }
        } catch (e) {
          logger.error("Failed to load column configs", { path: sheet.path, error: String(e) });
        }
      })
    );

    // Set derived configs and row configs before multiSheetData so they
    // are available when UniverSpreadsheet first mounts and builds the workbook.
    if (Object.keys(allDerivedConfigs).length > 0) {
      setDerivedColumnState((prev) => ({
        ...prev,
        configs: { ...prev.configs, ...allDerivedConfigs },
      }));
    }
    if (Object.keys(allRowConfigs).length > 0) {
      setRowConfigs((prev) => ({ ...prev, ...allRowConfigs }));
    }
    if (Object.keys(allColumnConfigs).length > 0) {
      setColumnConfigs((prev) => ({ ...prev, ...allColumnConfigs }));
    }

    setMultiSheetData({ sheets: validSheets });
    if (!activeSheetIdRef.current && validSheets.length > 0) {
      const initialId = restoredSheetId && validSheets.some((s) => s.id === restoredSheetId)
        ? restoredSheetId
        : validSheets[0].id;
      setActiveSheetId(initialId);
    }
    setError(null);
    setLoading(false);

    // Trigger derived column computations for all sheets
    for (const sheet of validSheets) {
      const info = sheetInfos.find((s) => s.id === sheet.id);
      const configs = allDerivedConfigs[sheet.id];
      if (info && configs) {
        triggerDerivedComputations(info, sheet.data, configs);
      }
    }
  }, [loadSingleFile, triggerDerivedComputations]);

  const reloadSheet = useCallback(async (sheetId: string) => {
    const sheetInfo = sheets.find((s) => s.id === sheetId);
    if (!sheetInfo) return;

    const data = await loadSingleFile(sheetInfo.path, sheetInfo.tableName);
    if (!data) return;

    // Update last known data for change detection
    lastKnownDataRef.current[sheetId] = data;

    setMultiSheetData((prev) => {
      if (!prev) return prev;
      const newSheets = prev.sheets.map((s) =>
        s.id === sheetId ? { ...s, data } : s
      );
      return { sheets: newSheets };
    });

    // Re-trigger derived column computations for reloaded sheet
    try {
      const configs = await ipc.getDerivedColumnsConfig(sheetInfo.path);
      if (configs.length > 0) {
        setDerivedColumnState((prev) => ({
          ...prev,
          configs: { ...prev.configs, [sheetId]: configs },
        }));
        triggerDerivedComputations(sheetInfo, data, configs);
      }
    } catch (e) {
      logger.error("Failed to reload derived columns", { path: sheetInfo.path, error: String(e) });
    }
  }, [sheets, loadSingleFile, triggerDerivedComputations]);

  const saveData = useCallback(
    async (newData: TomlTableData, sheetId: string) => {
      const sheetInfo = sheets.find((s) => s.id === sheetId);
      if (!sheetInfo) return;

      if (isSavingRef.current[sheetId]) return;
      isSavingRef.current[sheetId] = true;

      try {
        const previousData = lastKnownDataRef.current[sheetId];

        await ipc.saveTomlTable(sheetInfo.path, sheetInfo.tableName, newData);
        lastKnownDataRef.current[sheetId] = newData;
        // Record save time so file watcher can suppress self-triggered reloads
        lastSaveTimeRef.current[sheetId] = Date.now();

        const rowCountChanged = !previousData ||
          previousData.rows.length !== newData.rows.length;

        // When the row count changes, update multiSheetData so the Univer
        // useEffect can re-apply validation (checkboxes, dropdowns) with
        // the expanded range covering the new rows.
        if (rowCountChanged) {
          setMultiSheetData((prev) => {
            if (!prev) return prev;
            return {
              sheets: prev.sheets.map((s) =>
                s.id === sheetId ? { ...s, data: newData } : s
              ),
            };
          });
        }

        // Trigger derived computations for changed rows. When the row count
        // changed, re-trigger all rows because the multiSheetData useEffect
        // clears existing images during repopulation. When only values
        // changed, compute only the affected rows.
        const changedRows = rowCountChanged
          ? undefined
          : getChangedRowIndices(previousData!, newData);

        if (!changedRows || changedRows.length > 0) {
          try {
            const configs = await ipc.getDerivedColumnsConfig(sheetInfo.path);
            if (configs.length > 0) {
              setDerivedColumnState((prev) => ({
                ...prev,
                configs: { ...prev.configs, [sheetId]: configs },
              }));
              triggerDerivedComputations(sheetInfo, newData, configs, changedRows);
            }
          } catch (e) {
            logger.error("Failed to trigger post-save derived computations", {
              error: String(e),
            });
          }
        }
      } catch (e) {
        logger.error("Save error", { error: String(e) });
      } finally {
        isSavingRef.current[sheetId] = false;
      }
    },
    [sheets, triggerDerivedComputations]
  );

  const handleChange = useCallback(
    (newData: TomlTableData, sheetId: string) => {
      // Check if data actually changed - Univer fires set-range-values on cell
      // selection, not just on actual value changes
      const lastKnown = lastKnownDataRef.current[sheetId];
      if (lastKnown && isDataEqual(lastKnown, newData)) {
        // Data hasn't changed, skip save
        return;
      }

      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      saveTimeoutRef.current = window.setTimeout(() => {
        saveData(newData, sheetId);
      }, SAVE_DEBOUNCE_MS);
    },
    [saveData]
  );

  const handleActiveSheetChanged = useCallback((sheetId: string) => {
    setActiveSheetId(sheetId);
    const sheetInfo = sheets.find((s) => s.id === sheetId);
    if (sheetInfo) {
      ipc.saveViewState(sheetInfo.path).catch((e) =>
        logger.error("Failed to save view state", { error: String(e) })
      );
    }
  }, [sheets]);

  useEffect(() => {
    const init = async () => {
      try {
        const paths = await ipc.getAppPaths();
        if (paths.length === 0) {
          setError("No TOML files found");
          setLoading(false);
          return;
        }

        const sheetInfos: SheetInfo[] = paths.map((path) => ({
          id: generateSheetId(path),
          path,
          tableName: extractTableName(path),
        }));

        setSheets(sheetInfos);

        let restoredSheetId: string | undefined;
        try {
          const viewState = await ipc.loadViewState();
          if (viewState.active_sheet_path) {
            const match = sheetInfos.find((s) => s.path === viewState.active_sheet_path);
            if (match) {
              restoredSheetId = match.id;
            }
          }
        } catch (e) {
          logger.error("Failed to load view state", { error: String(e) });
        }

        await loadAllFiles(sheetInfos, restoredSheetId);

        for (const info of sheetInfos) {
          if (!watchersStartedRef.current.has(info.path)) {
            ipc.startFileWatcher(info.path).catch((e) =>
              logger.error("Failed to start file watcher", { path: info.path, error: String(e) })
            );
            watchersStartedRef.current.add(info.path);
          }
        }
      } catch (e) {
        setError(String(e));
        setLoading(false);
      }
    };

    init();
  }, [loadAllFiles]);

  useEffect(() => {
    const fileChangedSub = ipc.onFileChanged((payload) => {
      const sheetInfo = sheets.find((s) => s.path === payload.file_path);
      if (!sheetInfo) return;

      if (isSavingRef.current[sheetInfo.id]) {
        logger.debug("Ignoring file change during save", { filePath: payload.file_path });
        return;
      }

      // Suppress self-triggered reloads: the file watcher fires ~500-1000ms
      // after our own save completes, at which point isSavingRef is already
      // false. Use the save timestamp to detect and skip these events.
      const lastSave = lastSaveTimeRef.current[sheetInfo.id] ?? 0;
      if (Date.now() - lastSave < 1500) {
        logger.debug("Ignoring file change shortly after save", { filePath: payload.file_path });
        return;
      }

      logger.info("File changed externally, reloading sheet", { filePath: payload.file_path });
      reloadSheet(sheetInfo.id);
    });

    const derivedValueSub = ipc.onDerivedValueComputed((payload: DerivedValuePayload) => {
      const sheetInfo = sheets.find((s) => s.path === payload.file_path);
      if (!sheetInfo) return;

      setDerivedColumnState((prev) => {
        const sheetValues = prev.values[sheetInfo.id] ?? {};
        const rowValues = sheetValues[payload.row_index] ?? {};
        return {
          ...prev,
          values: {
            ...prev.values,
            [sheetInfo.id]: {
              ...sheetValues,
              [payload.row_index]: {
                ...rowValues,
                [payload.function_name]: payload.result,
              },
            },
          },
        };
      });
    });

    const syncStateSub = ipc.onSyncStateChanged((payload) => {
      logger.info("Sync state changed", { state: payload.state });
    });

    const conflictSub = ipc.onSyncConflict((payload) => {
      logger.info("Conflict detected", { message: payload.message });
      const sheetInfo = sheets.find((s) => s.path === payload.filePath);
      if (sheetInfo) {
        reloadSheet(sheetInfo.id);
      }
    });

    const derivedFailedSub = ipc.onDerivedFunctionFailed((payload) => {
      setError(`Derived column function "${payload.function_name}" failed: ${payload.error}`);
    });

    return () => {
      fileChangedSub.dispose();
      derivedValueSub.dispose();
      syncStateSub.dispose();
      conflictSub.dispose();
      derivedFailedSub.dispose();
    };
  }, [sheets, reloadSheet]);

  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      for (const path of watchersStartedRef.current) {
        ipc.stopFileWatcher(path).catch((e) =>
          logger.error("Failed to stop file watcher", { path, error: String(e) })
        );
      }
    };
  }, []);

  const handleRetry = useCallback(() => {
    if (sheets.length > 0) {
      loadAllFiles(sheets);
    }
  }, [sheets, loadAllFiles]);

  return (
    <div className={`tv-app ${error ? "has-error" : ""}`}>
      {error && (
        <ErrorBanner
          message={error}
          errorType="error"
          onDismiss={() => setError(null)}
          actions={[{ label: "Retry", onClick: handleRetry }]}
        />
      )}
      <div className="tv-main-content">
        <SpreadsheetView
          multiSheetData={multiSheetData}
          error={null}
          loading={loading}
          onChange={handleChange}
          onActiveSheetChanged={handleActiveSheetChanged}
          derivedColumnState={derivedColumnState}
          initialActiveSheetId={activeSheetId ?? undefined}
          rowConfigs={rowConfigs}
          columnConfigs={columnConfigs}
        />
      </div>
    </div>
  );
}
