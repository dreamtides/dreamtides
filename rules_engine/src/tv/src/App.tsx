import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { UniverSpreadsheet, TomlTableData } from "./UniverSpreadsheet";
import "./App.css";

function App() {
  const [data, setData] = useState<TomlTableData | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<TomlTableData>("load_toml_table", {
      filePath: "/Users/dthurn/Documents/GoogleDrive/dreamtides/rules_engine/tabula/dreamwell.toml",
      tableName: "dreamwell",
    })
      .then(setData)
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return <div className="error">Error loading TOML: {error}</div>;
  }

  return (
    <main className="container">
      <UniverSpreadsheet height="100vh" data={data} />
    </main>
  );
}

export default App;
