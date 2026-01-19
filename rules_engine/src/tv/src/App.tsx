import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { UniverSpreadsheet, TomlTableData } from "./UniverSpreadsheet";
import "./App.css";

const FILE_PATH =
  "/Users/dthurn/Documents/GoogleDrive/dreamtides/rules_engine/tabula/dreamwell.toml";
const TABLE_NAME = "dreamwell";
const SAVE_DEBOUNCE_MS = 500;

function App() {
  const [data, setData] = useState<TomlTableData | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [saveStatus, setSaveStatus] = useState<"idle" | "saving" | "saved" | "error">("idle");
  const saveTimeoutRef = useRef<number | null>(null);
  const isSavingRef = useRef(false);

  const loadData = async () => {
    try {
      const result = await invoke<TomlTableData>("load_toml_table", {
        filePath: FILE_PATH,
        tableName: TABLE_NAME,
      });
      setData(result);
    } catch (e) {
      setError(String(e));
    }
  };

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
  }, []);

  if (error) {
    return <div className="error">Error loading TOML: {error}</div>;
  }

  return (
    <main className="container">
      <div className="status-bar">
        {saveStatus === "saving" && <span className="status saving">Saving...</span>}
        {saveStatus === "saved" && <span className="status saved">Saved</span>}
        {saveStatus === "error" && <span className="status error">Save failed</span>}
      </div>
      <UniverSpreadsheet
        height="calc(100vh - 30px)"
        data={data}
        onChange={handleChange}
      />
    </main>
  );
}

export default App;
