import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SpreadsheetView } from "./spreadsheet_view";
import { TomlTableData } from "./UniverSpreadsheet";
import "./App.css";

export type { TomlTableData };

const FILE_PATH =
  "/Users/dthurn/Documents/GoogleDrive/dreamtides/rules_engine/tabula/dreamwell.toml";
const TABLE_NAME = "dreamwell";
const SAVE_DEBOUNCE_MS = 500;

export type SyncState = "idle" | "loading" | "saving" | "saved" | "error";

export function AppRoot() {
  const [data, setData] = useState<TomlTableData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saveStatus, setSaveStatus] = useState<SyncState>("idle");
  const saveTimeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef(false);

  const loadData = useCallback(async () => {
    try {
      setLoading(true);
      const result = await invoke<TomlTableData>("load_toml_table", {
        filePath: FILE_PATH,
        tableName: TABLE_NAME,
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
    if (isSavingRef.current) return;
    isSavingRef.current = true;
    setSaveStatus("saving");

    try {
      await invoke("save_toml_table", {
        filePath: FILE_PATH,
        tableName: TABLE_NAME,
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
  }, []);

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
    loadData();

    invoke("start_file_watcher", { filePath: FILE_PATH }).catch((e) =>
      console.error("Failed to start file watcher:", e)
    );

    const unlisten = listen<{ path: string }>("toml-file-changed", () => {
      if (!isSavingRef.current) {
        console.log("File changed externally, reloading...");
        loadData();
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
