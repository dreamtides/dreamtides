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

/** Performance log entry sent to the backend for the dedicated perf log file. */
interface PerfLogEntry {
  ts: string;
  component: string;
  operation: string;
  duration_ms: number;
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

/** Whether performance logging is enabled. */
let perfLoggingEnabled = true;

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
  if (import.meta.env.PROD) {
    return;
  }

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

function sendPerfToBackend(entry: PerfLogEntry): void {
  invoke("log_perf", { entry }).catch(() => {
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

/** Enables or disables performance logging. */
export function setPerfLoggingEnabled(enabled: boolean): void {
  perfLoggingEnabled = enabled;
}

/** Returns whether performance logging is enabled. */
export function isPerfLoggingEnabled(): boolean {
  return perfLoggingEnabled;
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

  /**
   * Starts a performance timer for measuring operation duration.
   * Returns a PerfTimer that should be stopped when the operation completes.
   *
   * @param operation - Name of the operation being measured
   * @param context - Optional additional context about the operation
   * @returns A PerfTimer instance to stop when the operation completes
   */
  startPerfTimer(operation: string, context?: Record<string, unknown>): PerfTimer {
    return new PerfTimer(this.component, operation, context);
  }

  /**
   * Logs a performance metric directly without using a timer.
   * Useful when duration is already known or computed externally.
   *
   * @param operation - Name of the operation being measured
   * @param durationMs - Duration of the operation in milliseconds
   * @param context - Optional additional context about the operation
   */
  perf(operation: string, durationMs: number, context?: Record<string, unknown>): void {
    if (!perfLoggingEnabled) {
      return;
    }

    if (!import.meta.env.PROD) {
      const prefix = `[${this.component}]`;
      const args: unknown[] = context
        ? [prefix, operation, `${durationMs.toFixed(2)}ms`, context]
        : [prefix, operation, `${durationMs.toFixed(2)}ms`];
      console.debug(...args);
    }

    const entry: PerfLogEntry = {
      ts: formatPacificTimestamp(),
      component: this.component,
      operation,
      duration_ms: durationMs,
    };
    if (context !== undefined) {
      entry.context = context;
    }

    sendPerfToBackend(entry);
  }
}

/**
 * A performance timer for measuring operation duration with millisecond precision.
 * Created via Logger.startPerfTimer() and stopped via stop().
 */
export class PerfTimer {
  private readonly component: string;
  private readonly operation: string;
  private readonly context?: Record<string, unknown>;
  private readonly startTime: number;
  private stopped = false;

  constructor(component: string, operation: string, context?: Record<string, unknown>) {
    this.component = component;
    this.operation = operation;
    this.context = context;
    this.startTime = performance.now();
  }

  /**
   * Stops the timer and logs the elapsed duration.
   * Can optionally add or override context values.
   *
   * @param additionalContext - Optional additional context to merge with initial context
   * @returns The elapsed duration in milliseconds
   */
  stop(additionalContext?: Record<string, unknown>): number {
    if (this.stopped) {
      return 0;
    }
    this.stopped = true;

    const durationMs = performance.now() - this.startTime;

    if (!perfLoggingEnabled) {
      return durationMs;
    }

    const mergedContext =
      this.context || additionalContext
        ? { ...this.context, ...additionalContext }
        : undefined;

    if (!import.meta.env.PROD) {
      const prefix = `[${this.component}]`;
      const args: unknown[] = mergedContext
        ? [prefix, this.operation, `${durationMs.toFixed(2)}ms`, mergedContext]
        : [prefix, this.operation, `${durationMs.toFixed(2)}ms`];
      console.debug(...args);
    }

    const entry: PerfLogEntry = {
      ts: formatPacificTimestamp(),
      component: this.component,
      operation: this.operation,
      duration_ms: durationMs,
    };
    if (mergedContext !== undefined) {
      entry.context = mergedContext;
    }

    sendPerfToBackend(entry);

    return durationMs;
  }

  /**
   * Returns the elapsed time without stopping the timer.
   * Useful for intermediate measurements.
   *
   * @returns The elapsed duration in milliseconds
   */
  elapsed(): number {
    return performance.now() - this.startTime;
  }
}
