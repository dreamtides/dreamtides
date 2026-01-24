import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SpreadsheetView } from "./spreadsheet_view";
import { TomlTableData } from "./UniverSpreadsheet";
import "./App.css";

export type { TomlTableData };

const SAVE_DEBOUNCE_MS = 500;

export type SyncState = "idle" | "loading" | "saving" | "saved" | "error";

function extractTableName(filePath: string): string {
  const fileName = filePath.split("/").pop() || filePath;
  return fileName.replace(/\.toml$/, "");
}

export function AppRoot() {
  const [filePath, setFilePath] = useState<string | null>(null);
  const [tableName, setTableName] = useState<string | null>(null);
  const [data, setData] = useState<TomlTableData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveStatus, setSaveStatus] = useState<SyncState>("idle");
  const saveTimeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef(false);

  const loadData = useCallback(async (path: string, table: string) => {
    try {
      setLoading(true);
      const result = await invoke<TomlTableData>("load_toml_table", {
        filePath: path,
        tableName: table,
      });
      setData(result);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const saveData = useCallback(async (newData: TomlTableData) => {
    if (isSavingRef.current || !filePath || !tableName) return;
    isSavingRef.current = true;
    setSaveStatus("saving");

    try {
      await invoke("save_toml_table", {
        filePath,
        tableName,
        data: newData,
      });
      setSaveStatus("saved");
      setTimeout(() => setSaveStatus("idle"), 1500);
    } catch (e) {
      console.error("Save error:", e);
      setSaveStatus("error");
    } finally {
      isSavingRef.current = false;
    }
  }, [filePath, tableName]);

  const handleChange = useCallback(
    (newData: TomlTableData) => {
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
      saveTimeoutRef.current = window.setTimeout(() => {
        saveData(newData);
      }, SAVE_DEBOUNCE_MS);
    },
    [saveData]
  );

  useEffect(() => {
    const init = async () => {
      try {
        const paths = await invoke<string[]>("get_app_paths");
        if (paths.length > 0) {
          const path = paths[0];
          const table = extractTableName(path);
          setFilePath(path);
          setTableName(table);
          await loadData(path, table);
          invoke("start_file_watcher", { filePath: path }).catch((e) =>
            console.error("Failed to start file watcher:", e)
          );
        } else {
          setError("No TOML files found");
          setLoading(false);
        }
      } catch (e) {
        setError(String(e));
        setLoading(false);
      }
    };

    init();

    const unlisten = listen<{ path: string }>("toml-file-changed", () => {
      if (!isSavingRef.current && filePath && tableName) {
        console.log("File changed externally, reloading...");
        loadData(filePath, tableName);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }
    };
  }, [loadData]);

  return (
    <SpreadsheetView
      data={data}
      error={error}
      loading={loading}
      saveStatus={saveStatus}
      onChange={handleChange}
    />
  );
}
