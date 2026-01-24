import { UniverSpreadsheet, TomlTableData } from "./UniverSpreadsheet";
import { SyncState } from "./app_root";

interface SpreadsheetViewProps {
  data: TomlTableData | null;
  error: string | null;
  loading: boolean;
  saveStatus: SyncState;
  onChange: (data: TomlTableData) => void;
}

export function SpreadsheetView({
  data,
  error,
  loading,
  saveStatus,
  onChange,
}: SpreadsheetViewProps) {
  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  if (error) {
    return <div className="error">Error loading TOML: {error}</div>;
  }

  return (
    <main className="container">
      <div className="status-bar">
        {saveStatus === "saving" && (
          <span className="status saving">Saving...</span>
        )}
        {saveStatus === "saved" && <span className="status saved">Saved</span>}
        {saveStatus === "error" && (
          <span className="status error">Save failed</span>
        )}
      </div>
      <UniverSpreadsheet
        height="calc(100vh - 30px)"
        data={data ?? undefined}
        onChange={onChange}
      />
    </main>
  );
}
