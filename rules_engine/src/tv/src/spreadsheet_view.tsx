import { UniverSpreadsheet } from "./UniverSpreadsheet";
import type { TomlTableData, RowConfig } from "./ipc_bridge";
import type { MultiSheetData, DerivedColumnState } from "./UniverSpreadsheet";

interface SpreadsheetViewProps {
  multiSheetData?: MultiSheetData | null;
  error: string | null;
  loading: boolean;
  onChange: (data: TomlTableData, sheetId: string) => void;
  onActiveSheetChanged?: (sheetId: string) => void;
  derivedColumnState?: DerivedColumnState;
  initialActiveSheetId?: string;
  rowConfigs?: Record<string, RowConfig>;
}

export function SpreadsheetView({
  multiSheetData,
  error,
  loading,
  onChange,
  onActiveSheetChanged,
  derivedColumnState,
  initialActiveSheetId,
  rowConfigs,
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
        derivedColumnState={derivedColumnState}
        initialActiveSheetId={initialActiveSheetId}
        rowConfigs={rowConfigs}
      />
    </main>
  );
}
