// logger_frontend.ts - Frontend logging with backend aggregation

import { invoke } from "@tauri-apps/api/core";

export type LogLevel = "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE";

interface LogEntry {
  ts: string;
  level: LogLevel;
  component: string;
  msg: string;
  context?: Record<string, unknown>;
}

const LOG_LEVEL_PRIORITY: Record<LogLevel, number> = {
  ERROR: 0,
  WARN: 1,
  INFO: 2,
  DEBUG: 3,
  TRACE: 4,
};

let currentLogLevel: LogLevel = "INFO";

function isPacificDaylightTime(date: Date): boolean {
  const jan = new Date(date.getFullYear(), 0, 1).getTimezoneOffset();
  const jul = new Date(date.getFullYear(), 6, 1).getTimezoneOffset();
  const stdOffset = Math.max(jan, jul);
  return date.getTimezoneOffset() < stdOffset;
}

function formatPacificTimestamp(): string {
  const now = new Date();
  const utcMs = now.getTime() + now.getTimezoneOffset() * 60000;
  const isDST = isPacificDaylightTime(now);
  const pacificOffsetMs = isDST ? -7 * 3600000 : -8 * 3600000;
  const pacificDate = new Date(utcMs + pacificOffsetMs);
  const offsetStr = isDST ? "-07:00" : "-08:00";

  const year = pacificDate.getFullYear();
  const month = String(pacificDate.getMonth() + 1).padStart(2, "0");
  const day = String(pacificDate.getDate()).padStart(2, "0");
  const hours = String(pacificDate.getHours()).padStart(2, "0");
  const minutes = String(pacificDate.getMinutes()).padStart(2, "0");
  const seconds = String(pacificDate.getSeconds()).padStart(2, "0");
  const ms = String(pacificDate.getMilliseconds()).padStart(3, "0");

  return `${year}-${month}-${day}T${hours}:${minutes}:${seconds}.${ms}${offsetStr}`;
}

function shouldLog(level: LogLevel): boolean {
  return LOG_LEVEL_PRIORITY[level] <= LOG_LEVEL_PRIORITY[currentLogLevel];
}

function mirrorToConsole(level: LogLevel, component: string, msg: string, context?: Record<string, unknown>): void {
  const prefix = `[${component}]`;
  const args: unknown[] = context ? [prefix, msg, context] : [prefix, msg];

  switch (level) {
    case "ERROR":
      console.error(...args);
      break;
    case "WARN":
      console.warn(...args);
      break;
    case "INFO":
      console.info(...args);
      break;
    case "DEBUG":
      console.debug(...args);
      break;
    case "TRACE":
      console.debug(...args);
      break;
  }
}

function sendToBackend(entry: LogEntry): void {
  invoke("log_message", { message: entry }).catch(() => {
    // Silently ignore backend send failures to avoid recursive logging
  });
}

function log(level: LogLevel, component: string, msg: string, context?: Record<string, unknown>): void {
  if (!shouldLog(level)) {
    return;
  }

  mirrorToConsole(level, component, msg, context);

  const entry: LogEntry = {
    ts: formatPacificTimestamp(),
    level,
    component,
    msg,
  };
  if (context !== undefined) {
    entry.context = context;
  }

  sendToBackend(entry);
}

/** Sets the minimum log level for frontend logging. */
export function setLogLevel(level: LogLevel): void {
  currentLogLevel = level;
}

/** Returns the current minimum log level. */
export function getLogLevel(): LogLevel {
  return currentLogLevel;
}

/** Creates a logger bound to a specific component name. */
export function createLogger(component: string): Logger {
  return new Logger(component);
}

export class Logger {
  private readonly component: string;

  constructor(component: string) {
    this.component = component;
  }

  error(msg: string, context?: Record<string, unknown>): void {
    log("ERROR", this.component, msg, context);
  }

  warn(msg: string, context?: Record<string, unknown>): void {
    log("WARN", this.component, msg, context);
  }

  info(msg: string, context?: Record<string, unknown>): void {
    log("INFO", this.component, msg, context);
  }

  debug(msg: string, context?: Record<string, unknown>): void {
    log("DEBUG", this.component, msg, context);
  }

  trace(msg: string, context?: Record<string, unknown>): void {
    log("TRACE", this.component, msg, context);
  }
}
