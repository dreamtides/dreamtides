import { useCallback, useEffect, useRef, useState } from "react";
import { SpreadsheetView } from "./spreadsheet_view";
import { ErrorBanner } from "./error_banner";
import { StatusIndicator } from "./status_indicator";
import * as ipc from "./ipc_bridge";
import type { TomlTableData, SyncState } from "./ipc_bridge";
import type { MultiSheetData, SheetData } from "./UniverSpreadsheet";

export type { TomlTableData, SyncState };

const SAVE_DEBOUNCE_MS = 500;

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
  const [saveStatus, setSaveStatus] = useState<SyncState>("idle");
  const saveTimeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef<Record<string, boolean>>({});
  const watchersStartedRef = useRef<Set<string>>(new Set());

  const loadSingleFile = useCallback(async (path: string, tableName: string): Promise<TomlTableData | null> => {
    try {
      return await ipc.loadTomlTable(path, tableName);
    } catch (e) {
      console.error(`Failed to load ${path}:`, e);
      return null;
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

    setMultiSheetData({ sheets: validSheets });
    if (!activeSheetId && validSheets.length > 0) {
      setActiveSheetId(validSheets[0].id);
    }
    setError(null);
    setLoading(false);
  }, [loadSingleFile, activeSheetId]);

  const reloadSheet = useCallback(async (sheetId: string) => {
    const sheetInfo = sheets.find((s) => s.id === sheetId);
    if (!sheetInfo) return;

    const data = await loadSingleFile(sheetInfo.path, sheetInfo.tableName);
    if (!data) return;

    setMultiSheetData((prev) => {
      if (!prev) return prev;
      const newSheets = prev.sheets.map((s) =>
        s.id === sheetId ? { ...s, data } : s
      );
      return { sheets: newSheets };
    });
  }, [sheets, loadSingleFile]);

  const getActiveSheetInfo = useCallback((): SheetInfo | undefined => {
    return sheets.find((s) => s.id === activeSheetId);
  }, [sheets, activeSheetId]);

  const saveData = useCallback(
    async (newData: TomlTableData, sheetId: string) => {
      const sheetInfo = sheets.find((s) => s.id === sheetId);
      if (!sheetInfo) return;

      if (isSavingRef.current[sheetId]) return;
      isSavingRef.current[sheetId] = true;
      setSaveStatus("saving");

      try {
        await ipc.saveTomlTable(sheetInfo.path, sheetInfo.tableName, newData);
        setSaveStatus("saved");
        setTimeout(() => setSaveStatus("idle"), 1500);
      } catch (e) {
        console.error("Save error:", e);
        setSaveStatus("error");
      } finally {
        isSavingRef.current[sheetId] = false;
      }
    },
    [sheets]
  );

  const handleChange = useCallback(
    (newData: TomlTableData, sheetId: string) => {
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
          saveStatus={saveStatus}
          onChange={handleChange}
          onActiveSheetChanged={handleActiveSheetChanged}
        />
      </div>
      <StatusIndicator
        syncState={saveStatus}
        autoHideDelay={2000}
      />
    </div>
  );
}
