// ipc_bridge.ts - Tauri command and event wrappers with TypeScript types

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// ============ Data Types ============

export interface TomlTableData {
  headers: string[];
  rows: (string | number | boolean | null)[][];
}

// ============ Metadata Schema Types ============
// Schema version 1 - see appendix_a_metadata_schema.md for documentation

/** Current metadata schema version */
export const CURRENT_SCHEMA_VERSION = 1;

/**
 * Root metadata structure for a TOML file.
 * Contains all spreadsheet configuration that persists across sessions.
 * The metadata section is never displayed in the spreadsheet grid.
 */
export interface Metadata {
  /** Schema version for forward compatibility. Defaults to 1. */
  schema_version?: number;
  /** Column configurations for data columns. */
  columns?: ColumnConfig[];
  /** Derived column definitions for computed values. */
  derived_columns?: DerivedColumnConfig[];
  /** Validation rules for data entry constraints. */
  validation_rules?: ValidationRule[];
  /** Conditional formatting rules. */
  conditional_formatting?: ConditionalFormatRule[];
  /** Table styling configuration. */
  table_style?: TableStyle;
  /** Sort state for visual ordering. */
  sort?: SortConfig;
  /** Filter configuration. */
  filter?: FilterConfig;
  /** Row-specific configurations. */
  rows?: RowConfig;
  /** Application settings stored with the file. */
  app_settings?: AppSettings;
}

/**
 * Configuration for a single data column.
 */
export interface ColumnConfig {
  /** TOML field name this configuration applies to. Required. */
  key: string;
  /** Column width in pixels. Defaults to 100. */
  width?: number;
  /** Text alignment. Defaults to "left". */
  alignment?: Alignment;
  /** Enable text wrapping. Defaults to false. */
  wrap?: boolean;
  /** Freeze this column in place. Defaults to false. */
  frozen?: boolean;
  /** Hide this column from view. Defaults to false. */
  hidden?: boolean;
  /** Number or date format pattern (e.g., "#,##0.00", "yyyy-mm-dd"). */
  format?: string;
}

/** Text alignment options. */
export type Alignment = "left" | "center" | "right";

/**
 * Configuration for a derived (computed) column.
 */
export interface DerivedColumnConfig {
  /** Display name for the column header. Required. */
  name: string;
  /** Registered function name to compute values. Required. */
  function: string;
  /** Column position (0-indexed). Defaults to end of columns. */
  position?: number;
  /** Column width in pixels. Defaults to 100. */
  width?: number;
  /** Input field names passed to the function. */
  inputs?: string[];
}

/**
 * Validation rule for data entry constraints.
 */
export type ValidationRule =
  | EnumValidationRule
  | RangeValidationRule
  | PatternValidationRule
  | RequiredValidationRule
  | TypeValidationRule;

export interface EnumValidationRule {
  type: "enum";
  column: string;
  enum: string[];
  message?: string;
}

export interface RangeValidationRule {
  type: "range";
  column: string;
  min?: number;
  max?: number;
  message?: string;
}

export interface PatternValidationRule {
  type: "pattern";
  column: string;
  pattern: string;
  message?: string;
}

export interface RequiredValidationRule {
  type: "required";
  column: string;
  message?: string;
}

export interface TypeValidationRule {
  type: "type";
  column: string;
  value_type: ValueType;
  message?: string;
}

/** Value type for type validation. */
export type ValueType = "string" | "integer" | "float" | "boolean";

/**
 * Conditional formatting rule.
 */
export interface ConditionalFormatRule {
  /** Column this rule applies to. */
  column: string;
  /** Condition to evaluate. */
  condition: FormatCondition;
  /** Style to apply when condition is met. */
  style: FormatStyle;
}

/** Condition for conditional formatting. */
export type FormatCondition =
  | { equals: unknown }
  | { contains: string }
  | { greater_than: number }
  | { less_than: number }
  | { is_empty: true }
  | { not_empty: true }
  | { matches: string };

/**
 * Style for conditional formatting.
 */
export interface FormatStyle {
  /** Background color as hex string (e.g., "#FF0000"). */
  background_color?: string;
  /** Font color as hex string. */
  font_color?: string;
  /** Bold text. */
  bold?: boolean;
  /** Italic text. */
  italic?: boolean;
  /** Underlined text. */
  underline?: boolean;
}

/**
 * Table styling configuration.
 */
export interface TableStyle {
  /** Color scheme name (e.g., "blue_light", "green_medium", "gray_classic"). */
  color_scheme?: string;
  /** Show alternating row colors. Defaults to true. */
  show_row_stripes?: boolean;
  /** Show alternating column colors. Defaults to false. */
  show_column_stripes?: boolean;
  /** Bold header text. Defaults to true. */
  header_bold?: boolean;
  /** Override hex color for header row. */
  header_background?: string;
}

/**
 * Sort configuration.
 */
export interface SortConfig {
  /** Column key to sort by. */
  column: string;
  /** Sort direction. True for ascending. Defaults to true. */
  ascending?: boolean;
}

/**
 * Filter configuration.
 */
export interface FilterConfig {
  /** Individual column filters. */
  filters?: ColumnFilter[];
  /** Whether filters are currently active. */
  active?: boolean;
}

/**
 * Filter for a single column.
 */
export interface ColumnFilter {
  /** Column key this filter applies to. */
  column: string;
  /** Filter condition. */
  condition: FilterCondition;
}

/** Filter condition types. */
export type FilterCondition =
  | { contains: string }
  | { equals: unknown }
  | { range: { min?: number; max?: number } }
  | { boolean: boolean };

/**
 * Row-specific configuration.
 */
export interface RowConfig {
  /** Height overrides by row. */
  heights?: RowHeight[];
  /** Hidden row indices. */
  hidden?: number[];
}

/**
 * Height configuration for a specific row.
 */
export interface RowHeight {
  /** Row index (0-indexed, excluding header). */
  row: number;
  /** Height in pixels. */
  height: number;
}

/**
 * Application settings stored with the file.
 */
export interface AppSettings {
  /** Last selected cell in A1 notation (e.g., "B5"). */
  last_selected_cell?: string;
  /** Scroll position for view restoration. */
  scroll_position?: ScrollPosition;
  /** Zoom level as a multiplier. Defaults to 1.0. */
  zoom_level?: number;
}

/**
 * Scroll position for view restoration.
 */
export interface ScrollPosition {
  /** First visible row index. */
  row: number;
  /** First visible column index. */
  column: number;
}

// Legacy interface for backwards compatibility
/** @deprecated Use Metadata instead */
export interface TomlMetadata {
  columns?: Record<string, { width?: number; alignment?: Alignment; wrap?: boolean }>;
  validation?: Record<string, ValidationRule[]>;
  derived?: Record<string, { function: string; inputs?: string[] }>;
}

// ============ Sort State ============

export type SortDirection = "ascending" | "descending";

export interface SortState {
  column: string;
  direction: SortDirection;
}

export interface SortStateResponse {
  column: string | null;
  direction: SortDirection | null;
}

// ============ Sync State ============

export type SyncState = "idle" | "saving" | "loading" | "saved" | "error";

export interface CellUpdate {
  filePath: string;
  rowIndex: number;
  columnKey: string;
  value: unknown;
}

export interface SaveResult {
  success: boolean;
  generatedValues?: Record<string, unknown>;
  error?: string;
}

export interface BatchCellUpdate {
  rowIndex: number;
  columnKey: string;
  value: unknown;
}

export interface FailedUpdate {
  rowIndex: number;
  columnKey: string;
  reason: string;
}

export interface SaveBatchResult {
  success: boolean;
  appliedCount: number;
  failedCount: number;
  failedUpdates: FailedUpdate[];
}

// ============ Rich Text Types ============

export interface UniverRichText {
  p: Paragraph[];
}

export interface Paragraph {
  ts: TextRun[];
}

export interface TextRun {
  t: string;
  s?: TextStyle;
}

export interface TextStyle {
  bl?: number;
  it?: number;
  ul?: UnderlineStyle;
  cl?: FontColor;
}

export interface UnderlineStyle {
  s: number;
}

export interface FontColor {
  rgb: string;
}

// ============ Derived Result Types ============

export type DerivedResultValue =
  | { type: "text"; value: string }
  | { type: "number"; value: number }
  | { type: "boolean"; value: boolean }
  | { type: "image"; value: string }
  | { type: "richText"; value: UniverRichText }
  | { type: "error"; value: string };

// ============ Event Payloads ============

export interface FileChangedPayload {
  file_path: string;
  event_type: string;
}

export interface DerivedValuePayload {
  file_path: string;
  table_name: string;
  row_index: number;
  function_name: string;
  result: DerivedResultValue;
  generation: number;
}

export interface SaveCompletedPayload {
  success: boolean;
  filePath: string;
  error?: string;
}

export interface ErrorPayload {
  message: string;
  errorType: string;
  filePath?: string;
}

export interface SyncStateChangedPayload {
  state: SyncState;
  timestamp: number;
}

export interface SyncErrorPayload {
  message: string;
}

export interface SyncConflictPayload {
  filePath: string;
  message: string;
}

// ============ Commands ============

export async function loadTomlTable(
  filePath: string,
  tableName: string
): Promise<TomlTableData> {
  return invoke<TomlTableData>("load_toml_table", { filePath, tableName });
}

export async function saveTomlTable(
  filePath: string,
  tableName: string,
  data: TomlTableData
): Promise<void> {
  return invoke("save_toml_table", { filePath, tableName, data });
}

export async function saveCell(
  filePath: string,
  tableName: string,
  rowIndex: number,
  columnKey: string,
  value: unknown
): Promise<SaveResult> {
  return invoke<SaveResult>("save_cell", {
    filePath,
    tableName,
    rowIndex,
    columnKey,
    value,
  });
}

export async function saveBatch(
  filePath: string,
  tableName: string,
  updates: BatchCellUpdate[]
): Promise<SaveBatchResult> {
  return invoke<SaveBatchResult>("save_batch", {
    filePath,
    tableName,
    updates,
  });
}

export async function startFileWatcher(filePath: string): Promise<void> {
  return invoke("start_file_watcher", { filePath });
}

export async function stopFileWatcher(filePath: string): Promise<void> {
  return invoke("stop_file_watcher", { filePath });
}

export async function getAppPaths(): Promise<string[]> {
  return invoke<string[]>("get_app_paths");
}

export async function getSortState(
  filePath: string,
  tableName: string
): Promise<SortStateResponse> {
  return invoke<SortStateResponse>("get_sort_state", { filePath, tableName });
}

export async function setSortState(
  filePath: string,
  tableName: string,
  sort: SortState | null
): Promise<SortStateResponse> {
  return invoke<SortStateResponse>("set_sort_state", {
    filePath,
    tableName,
    sort,
  });
}

export async function clearSortState(
  filePath: string,
  tableName: string
): Promise<SortStateResponse> {
  return invoke<SortStateResponse>("clear_sort_state", { filePath, tableName });
}

export async function getSortRowMapping(
  filePath: string,
  tableName: string
): Promise<number[]> {
  return invoke<number[]>("get_sort_row_mapping", { filePath, tableName });
}

export async function translateRowIndex(
  filePath: string,
  tableName: string,
  displayIndex: number
): Promise<number> {
  return invoke<number>("translate_row_index", {
    filePath,
    tableName,
    displayIndex,
  });
}

// ============ Image Commands ============

/**
 * Fetches an image by URL and returns the local cache file path.
 *
 * On cache hit, returns immediately. On cache miss, fetches the image
 * over HTTP, validates it, stores it in the content-addressed cache,
 * and returns the cached file path. The returned path should be converted
 * to an asset URL using `convertFileSrc()` before passing to Univer.
 */
export async function fetchImage(url: string): Promise<string> {
  return invoke<string>("fetch_image", { url });
}

// ============ Derived Column Commands ============

export interface DerivedColumnInfo {
  name: string;
  function: string;
  position?: number;
  width: number;
  inputs: string[];
}

export interface ComputeDerivedRequest {
  file_path: string;
  table_name: string;
  row_index: number;
  function_name: string;
  row_data: Record<string, unknown>;
  is_visible: boolean;
}

export interface ComputeDerivedBatchRequest {
  requests: ComputeDerivedRequest[];
}

export async function getDerivedColumnsConfig(
  filePath: string
): Promise<DerivedColumnInfo[]> {
  return invoke<DerivedColumnInfo[]>("get_derived_columns_config", { filePath });
}

export async function computeDerived(
  request: ComputeDerivedRequest
): Promise<void> {
  return invoke("compute_derived", { request });
}

export async function computeDerivedBatch(
  batch: ComputeDerivedBatchRequest
): Promise<void> {
  return invoke("compute_derived_batch", { batch });
}

export async function incrementRowGeneration(
  filePath: string,
  tableName: string,
  rowIndex: number
): Promise<number> {
  return invoke<number>("increment_row_generation", {
    filePath,
    tableName,
    rowIndex,
  });
}

export async function clearComputationQueue(): Promise<void> {
  return invoke("clear_computation_queue");
}

// ============ Validation Commands ============

export interface EnumValidationInfo {
  column: string;
  allowed_values: string[];
}

export async function getEnumValidationRules(
  filePath: string
): Promise<EnumValidationInfo[]> {
  return invoke<EnumValidationInfo[]>("get_enum_validation_rules", { filePath });
}

// ============ Style Commands ============

export interface ColorPalette {
  header_background: string;
  header_font_color: string;
  row_even_background: string;
  row_odd_background: string;
  accent_color: string;
}

export interface ResolvedTableStyle {
  palette: ColorPalette | null;
  show_row_stripes: boolean;
  show_column_stripes: boolean;
  header_bold: boolean;
  header_background: string | null;
}

export async function getTableStyle(
  filePath: string
): Promise<ResolvedTableStyle | null> {
  return invoke<ResolvedTableStyle | null>("get_table_style", { filePath });
}

export async function getAvailableColorSchemes(): Promise<string[]> {
  return invoke<string[]>("get_available_color_schemes");
}

// ============ Logging Commands ============

export interface FrontendLogMessage {
  ts: string;
  level: string;
  component: string;
  msg: string;
  context?: Record<string, unknown>;
}

export async function logMessage(
  message: FrontendLogMessage
): Promise<void> {
  return invoke("log_message", { message });
}

// ============ Events ============

export type Disposable = { dispose: () => void };

export function onFileChanged(
  callback: (payload: FileChangedPayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<FileChangedPayload>("toml-file-changed", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}

export function onDerivedValueComputed(
  callback: (payload: DerivedValuePayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<DerivedValuePayload>("derived-value-computed", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}

export function onSaveCompleted(
  callback: (payload: SaveCompletedPayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<SaveCompletedPayload>("save-completed", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}

export function onError(
  callback: (payload: ErrorPayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<ErrorPayload>("error-occurred", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}

export function onSyncStateChanged(
  callback: (payload: SyncStateChangedPayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<SyncStateChangedPayload>("sync-state-changed", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}

export function onSyncError(
  callback: (payload: SyncErrorPayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<SyncErrorPayload>("sync-error", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}

export function onSyncConflict(
  callback: (payload: SyncConflictPayload) => void
): Disposable {
  let unlisten: UnlistenFn | null = null;

  listen<SyncConflictPayload>("sync-conflict-detected", (event) => {
    callback(event.payload);
  }).then((fn) => {
    unlisten = fn;
  });

  return {
    dispose: () => {
      if (unlisten) unlisten();
    },
  };
}
