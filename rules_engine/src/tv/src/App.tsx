import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  UniverSpreadsheet,
  UniverSpreadsheetHandle,
  TomlTableData,
} from "./UniverSpreadsheet";
import "./App.css";

const FILE_PATH =
  "/Users/dthurn/Documents/GoogleDrive/dreamtides/rules_engine/tabula/dreamwell.toml";
const TABLE_NAME = "dreamwell";

function App() {
  const [data, setData] = useState<TomlTableData | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const [saveStatus, setSaveStatus] = useState<string | null>(null);
  const spreadsheetRef = useRef<UniverSpreadsheetHandle>(null);

  useEffect(() => {
    invoke<TomlTableData>("load_toml_table", {
      filePath: FILE_PATH,
      tableName: TABLE_NAME,
    })
      .then(setData)
      .catch((e) => setError(String(e)));
  }, []);

  const handleSave = async () => {
    const currentData = spreadsheetRef.current?.getData();
    if (!currentData) {
      setSaveStatus("No data to save");
      return;
    }

    setSaveStatus("Saving...");
    try {
      await invoke("save_toml_table", {
        filePath: FILE_PATH,
        tableName: TABLE_NAME,
        data: currentData,
      });
      setSaveStatus("Saved successfully!");
      setTimeout(() => setSaveStatus(null), 2000);
    } catch (e) {
      setSaveStatus(`Error: ${e}`);
    }
  };

  if (error) {
    return <div className="error">Error loading TOML: {error}</div>;
  }

  return (
    <main className="container">
      <div className="toolbar">
        <button onClick={handleSave} className="save-button">
          Save to TOML
        </button>
        {saveStatus && <span className="save-status">{saveStatus}</span>}
      </div>
      <UniverSpreadsheet ref={spreadsheetRef} height="calc(100vh - 50px)" data={data} />
    </main>
  );
}

export default App;
