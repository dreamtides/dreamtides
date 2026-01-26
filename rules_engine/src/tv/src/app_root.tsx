import { useCallback, useEffect, useRef, useState } from "react";
import { SpreadsheetView } from "./spreadsheet_view";
import { ErrorBanner } from "./error_banner";
import * as ipc from "./ipc_bridge";
import type { TomlTableData, DerivedValuePayload } from "./ipc_bridge";
import type { MultiSheetData, SheetData, DerivedColumnState } from "./UniverSpreadsheet";

export type { TomlTableData };

const SAVE_DEBOUNCE_MS = 500;

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
      // Compare values, treating null and undefined as equal
      if (cellA !== cellB) {
        // Handle null/undefined equivalence
        if ((cellA === null || cellA === undefined) && (cellB === null || cellB === undefined)) {
          continue;
        }
        return false;
      }
    }
  }

  return true;
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
  const saveTimeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef<Record<string, boolean>>({});
  const watchersStartedRef = useRef<Set<string>>(new Set());
  // Track last known data for each sheet to detect actual changes
  const lastKnownDataRef = useRef<Record<string, TomlTableData>>({});
  // Ref to track activeSheetId without causing effect re-runs
  const activeSheetIdRef = useRef<string | null>(null);
  activeSheetIdRef.current = activeSheetId;

  const loadSingleFile = useCallback(async (path: string, tableName: string): Promise<TomlTableData | null> => {
    try {
      return await ipc.loadTomlTable(path, tableName);
    } catch (e) {
      console.error(`Failed to load ${path}:`, e);
      return null;
    }
  }, []);

  const loadDerivedColumnsForSheet = useCallback(async (sheetInfo: SheetInfo, data: TomlTableData) => {
    try {
      const configs = await ipc.getDerivedColumnsConfig(sheetInfo.path);
      if (configs.length === 0) return;

      setDerivedColumnState((prev) => ({
        ...prev,
        configs: { ...prev.configs, [sheetInfo.id]: configs },
      }));

      // Build row data and trigger batch computation
      const requests: ipc.ComputeDerivedRequest[] = [];
      for (let rowIndex = 0; rowIndex < data.rows.length; rowIndex++) {
        const rowData: Record<string, unknown> = {};
        for (let colIdx = 0; colIdx < data.headers.length; colIdx++) {
          rowData[data.headers[colIdx]] = data.rows[rowIndex][colIdx];
        }

        for (const config of configs) {
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
      console.error(`Failed to load derived columns for ${sheetInfo.path}:`, e);
    }
  }, []);

  const loadAllFiles = useCallback(async (sheetInfos: SheetInfo[]): Promise<void> => {
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

    setMultiSheetData({ sheets: validSheets });
    if (!activeSheetIdRef.current && validSheets.length > 0) {
      setActiveSheetId(validSheets[0].id);
    }
    setError(null);
    setLoading(false);

    // Load derived column configs and trigger computations for all sheets
    for (const sheet of validSheets) {
      const info = sheetInfos.find((s) => s.id === sheet.id);
      if (info) {
        loadDerivedColumnsForSheet(info, sheet.data);
      }
    }
  }, [loadSingleFile, loadDerivedColumnsForSheet]);

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
    loadDerivedColumnsForSheet(sheetInfo, data);
  }, [sheets, loadSingleFile, loadDerivedColumnsForSheet]);

  const getActiveSheetInfo = useCallback((): SheetInfo | undefined => {
    return sheets.find((s) => s.id === activeSheetId);
  }, [sheets, activeSheetId]);

  const saveData = useCallback(
    async (newData: TomlTableData, sheetId: string) => {
      const sheetInfo = sheets.find((s) => s.id === sheetId);
      if (!sheetInfo) return;

      if (isSavingRef.current[sheetId]) return;
      isSavingRef.current[sheetId] = true;

      try {
        await ipc.saveTomlTable(sheetInfo.path, sheetInfo.tableName, newData);
        // Update last known data after successful save
        lastKnownDataRef.current[sheetId] = newData;
      } catch (e) {
        console.error("Save error:", e);
      } finally {
        isSavingRef.current[sheetId] = false;
      }
    },
    [sheets]
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
  }, []);

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
        await loadAllFiles(sheetInfos);

        for (const info of sheetInfos) {
          if (!watchersStartedRef.current.has(info.path)) {
            ipc.startFileWatcher(info.path).catch((e) =>
              console.error(`Failed to start file watcher for ${info.path}:`, e)
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
        console.log(`Ignoring file change for ${payload.file_path} during save`);
        return;
      }

      console.log(`File changed externally: ${payload.file_path}, reloading sheet...`);
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
      console.log("Sync state changed:", payload.state);
    });

    const conflictSub = ipc.onSyncConflict((payload) => {
      console.log("Conflict detected:", payload.message);
      const sheetInfo = sheets.find((s) => s.path === payload.filePath);
      if (sheetInfo) {
        reloadSheet(sheetInfo.id);
      }
    });

    return () => {
      fileChangedSub.dispose();
      derivedValueSub.dispose();
      syncStateSub.dispose();
      conflictSub.dispose();
    };
  }, [sheets, reloadSheet]);

  useEffect(() => {
    return () => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      for (const path of watchersStartedRef.current) {
        ipc.stopFileWatcher(path).catch((e) =>
          console.error(`Failed to stop file watcher for ${path}:`, e)
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
        />
      </div>
    </div>
  );
}
