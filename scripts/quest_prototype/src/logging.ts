/** Base structure for all log events. */
export interface LogEntry {
  timestamp: string;
  event: string;
  seq: number;
  [key: string]: unknown;
}

const RESERVED_KEYS: ReadonlySet<string> = new Set([
  "timestamp",
  "event",
  "seq",
]);

let sequenceCounter = 0;
const logAccumulator: LogEntry[] = [];

/**
 * Log a structured event. Assigns timestamp and sequence number
 * automatically, writes single-line JSON to console.log, and stores
 * the entry in the in-memory accumulator.
 *
 * Reserved fields (`timestamp`, `event`, `seq`) in the additional
 * fields parameter are silently stripped so that logger-assigned
 * values are always authoritative.
 */
export function logEvent(
  event: string,
  fields: Record<string, unknown> = {},
): Readonly<LogEntry> {
  sequenceCounter += 1;
  const sanitized: Record<string, unknown> = {};
  for (const key of Object.keys(fields)) {
    if (!RESERVED_KEYS.has(key)) {
      sanitized[key] = fields[key];
    }
  }
  const entry: LogEntry = {
    ...sanitized,
    timestamp: new Date().toISOString(),
    event,
    seq: sequenceCounter,
  };
  console.log(JSON.stringify(entry));
  logAccumulator.push(entry);
  fetch("/api/log", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(entry),
  }).catch(() => {});
  return Object.freeze({ ...entry });
}

/** Returns a deep-copied snapshot of all accumulated log entries. */
export function getLogEntries(): ReadonlyArray<Readonly<LogEntry>> {
  return logAccumulator.map((e) => Object.freeze({ ...e }));
}

/** Clears the in-memory log accumulator and resets the sequence counter. */
export function resetLog(): void {
  sequenceCounter = 0;
  logAccumulator.length = 0;
}

/**
 * Downloads the accumulated log as a `.jsonl` file. Each entry is
 * serialized as a single JSON line. The filename includes an ISO
 * timestamp for uniqueness.
 */
export function downloadLog(): void {
  const lines = logAccumulator.map((entry) => JSON.stringify(entry));
  const content = lines.join("\n") + "\n";
  const blob = new Blob([content], { type: "application/x-jsonlines" });
  const url = URL.createObjectURL(blob);

  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const filename = `quest-log-${timestamp}.jsonl`;

  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}
