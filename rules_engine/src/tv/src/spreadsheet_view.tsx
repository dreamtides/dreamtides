import { UniverSpreadsheet } from "./UniverSpreadsheet";
import type { TomlTableData, RowConfig, ColumnConfig } from "./ipc_bridge";
import type { MultiSheetData, DerivedColumnState } from "./spreadsheet_types";

interface SpreadsheetViewProps {
  multiSheetData?: MultiSheetData | null;
  error: string | null;
  loading: boolean;
  onChange: (data: TomlTableData, sheetId: string) => void;
  onActiveSheetChanged?: (sheetId: string) => void;
  onSheetOrderChanged?: (sheetNames: string[]) => void;
  derivedColumnState?: DerivedColumnState;
  initialActiveSheetId?: string;
  rowConfigs?: Record<string, RowConfig>;
  columnConfigs?: Record<string, ColumnConfig[]>;
  persistedSheetOrder?: string[];
}

export function SpreadsheetView({
  multiSheetData,
  error,
  loading,
  onChange,
  onActiveSheetChanged,
  onSheetOrderChanged,
  derivedColumnState,
  initialActiveSheetId,
  rowConfigs,
  columnConfigs,
  persistedSheetOrder,
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
        onSheetOrderChanged={onSheetOrderChanged}
        derivedColumnState={derivedColumnState}
        initialActiveSheetId={initialActiveSheetId}
        rowConfigs={rowConfigs}
        columnConfigs={columnConfigs}
        persistedSheetOrder={persistedSheetOrder}
      />
    </main>
  );
}
