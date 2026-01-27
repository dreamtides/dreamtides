import type {
  TomlTableData,
  DerivedColumnInfo,
  DerivedResultValue,
  RowConfig,
  ColumnConfig,
} from "./ipc_bridge";

/**
 * Tracks derived column configurations and computed values per sheet.
 *
 * configs: Maps sheet ID to its derived column configurations.
 * values: Maps sheet ID -> row index -> function name -> computed result.
 */
export interface DerivedColumnState {
  configs: Record<string, DerivedColumnInfo[]>;
  values: Record<string, Record<number, Record<string, DerivedResultValue>>>;
}

/**
 * Represents a single sheet within a multi-sheet workbook.
 * Each sheet corresponds to one TOML file.
 */
export interface SheetData {
  /** Unique sheet ID */
  id: string;
  /** Display name (filename without .toml extension) */
  name: string;
  /** Full file path for reference */
  path: string;
  /** The actual TOML table data */
  data: TomlTableData;
}

/**
 * Multi-sheet workbook data structure.
 * Contains multiple sheets, each representing a TOML file.
 */
export interface MultiSheetData {
  sheets: SheetData[];
}

export interface UniverSpreadsheetHandle {
  /** Get data from the currently active sheet */
  getData: () => TomlTableData | null;
  /** Get the ID of the currently active sheet */
  getActiveSheetId: () => string | null;
  /** Get data from a specific sheet by ID */
  getSheetData: (sheetId: string) => TomlTableData | null;
}

export interface UniverSpreadsheetProps {
  width?: string | number;
  height?: string | number;
  /** Single sheet data (legacy support) */
  data?: TomlTableData;
  /** Multi-sheet data - takes precedence over single data prop */
  multiSheetData?: MultiSheetData;
  /** Called when cell data changes in any sheet */
  onChange?: (data: TomlTableData, sheetId: string) => void;
  /** Called when the active sheet changes */
  onActiveSheetChanged?: (sheetId: string) => void;
  /** Called when the user reorders sheet tabs via drag */
  onSheetOrderChanged?: (sheetNames: string[]) => void;
  /** Derived column configurations and computed values */
  derivedColumnState?: DerivedColumnState;
  /** Sheet ID to activate on initial workbook creation */
  initialActiveSheetId?: string;
  /** Row configurations per sheet ID (default_height, per-row overrides) */
  rowConfigs?: Record<string, RowConfig>;
  /** Column configurations per sheet ID (widths from metadata) */
  columnConfigs?: Record<string, ColumnConfig[]>;
  /** Persisted sheet order (filenames without .toml extension) for tab ordering */
  persistedSheetOrder?: string[];
}
