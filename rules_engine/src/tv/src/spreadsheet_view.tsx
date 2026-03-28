import { useCallback, useRef, useState } from "react";
import type { FUniver } from "@univerjs/core/facade";
import { UniverSpreadsheet } from "./UniverSpreadsheet";
import { DeleteRowOverlay } from "./delete_row_overlay";
import type { TomlTableData, RowConfig, ColumnConfig } from "./ipc_bridge";
import type { MultiSheetData, DerivedColumnState } from "./spreadsheet_types";

interface SpreadsheetViewProps {
  multiSheetData?: MultiSheetData | null;
  error: string | null;
  loading: boolean;
  onChange: (data: TomlTableData, sheetId: string) => void;
  onDeleteRow?: (sheetId: string, displayRowIndex: number) => void;
  onActiveSheetChanged?: (sheetId: string) => void;
  onSheetOrderChanged?: (sheetNames: string[]) => void;
  derivedColumnState?: DerivedColumnState;
  initialActiveSheetId?: string;
  rowConfigs?: Record<string, RowConfig>;
  columnConfigs?: Record<string, ColumnConfig[]>;
  persistedSheetOrder?: string[];
  deleteButtonsVisible?: boolean;
  activeSheetId?: string;
}

export function SpreadsheetView({
  multiSheetData,
  error,
  loading,
  onChange,
  onDeleteRow,
  onActiveSheetChanged,
  onSheetOrderChanged,
  derivedColumnState,
  initialActiveSheetId,
  rowConfigs,
  columnConfigs,
  persistedSheetOrder,
  deleteButtonsVisible,
  activeSheetId,
}: SpreadsheetViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [univerAPI, setUniverAPI] = useState<FUniver | null>(null);

  const handleUniverAPIReady = useCallback((api: FUniver) => {
    setUniverAPI(api);
  }, []);

  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  if (error) {
    return <div className="error">Error loading TOML: {error}</div>;
  }

  return (
    <main className="container" ref={containerRef} style={{ position: "relative" }}>
      <UniverSpreadsheet
        height="100%"
        multiSheetData={multiSheetData ?? undefined}
        onChange={onChange}
        onDeleteRow={onDeleteRow}
        onActiveSheetChanged={onActiveSheetChanged}
        onSheetOrderChanged={onSheetOrderChanged}
        onUniverAPIReady={handleUniverAPIReady}
        derivedColumnState={derivedColumnState}
        initialActiveSheetId={initialActiveSheetId}
        rowConfigs={rowConfigs}
        columnConfigs={columnConfigs}
        persistedSheetOrder={persistedSheetOrder}
      />
      {deleteButtonsVisible && (
        <DeleteRowOverlay
          containerRef={containerRef}
          univerAPI={univerAPI}
          onDeleteRow={onDeleteRow}
          activeSheetId={activeSheetId ?? ""}
        />
      )}
    </main>
  );
}
