import { UniverSpreadsheet } from "./UniverSpreadsheet";
import type { TomlTableData } from "./ipc_bridge";
import type { MultiSheetData } from "./UniverSpreadsheet";

interface SpreadsheetViewProps {
  multiSheetData?: MultiSheetData | null;
  error: string | null;
  loading: boolean;
  onChange: (data: TomlTableData, sheetId: string) => void;
  onActiveSheetChanged?: (sheetId: string) => void;
}

export function SpreadsheetView({
  multiSheetData,
  error,
  loading,
  onChange,
  onActiveSheetChanged,
}: SpreadsheetViewProps) {
  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  if (error) {
    return <div className="error">Error loading TOML: {error}</div>;
  }

  return (
    <main className="container">
      <UniverSpreadsheet
        height="100vh"
        multiSheetData={multiSheetData ?? undefined}
        onChange={onChange}
        onActiveSheetChanged={onActiveSheetChanged}
      />
    </main>
  );
}
