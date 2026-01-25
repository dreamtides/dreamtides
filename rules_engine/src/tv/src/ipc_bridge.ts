// ipc_bridge.ts - Tauri command and event wrappers with TypeScript types

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// ============ Data Types ============

export interface TomlTableData {
  headers: string[];
  rows: (string | number | boolean | null)[][];
}

export interface TomlMetadata {
  columns?: Record<string, ColumnConfig>;
  validation?: Record<string, ValidationRule[]>;
  derived?: Record<string, DerivedColumnConfig>;
}

export interface ColumnConfig {
  width?: number;
  alignment?: "left" | "center" | "right";
  wrap?: boolean;
}

export interface ValidationRule {
  type: "enum" | "range" | "pattern" | "required" | "type";
  values?: string[];
  min?: number;
  max?: number;
  pattern?: string;
  message?: string;
}

export interface DerivedColumnConfig {
  function: string;
  inputs?: string[];
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
  path: string;
}

export interface DerivedValuePayload {
  rowIndex: number;
  columnKey: string;
  value: unknown;
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
