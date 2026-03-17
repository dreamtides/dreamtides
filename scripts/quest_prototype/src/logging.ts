/** Base structure for all log events. */
export interface LogEntry {
  timestamp: string;
  event: string;
  seq: number;
  [key: string]: unknown;
}

let sequenceCounter = 0;
const logAccumulator: LogEntry[] = [];

/**
 * Log a structured event. Assigns timestamp and sequence number
 * automatically, writes single-line JSON to console.log, and stores
 * the entry in the in-memory accumulator.
 */
export function logEvent(
  event: string,
  fields: Record<string, unknown> = {},
): LogEntry {
  sequenceCounter += 1;
  const entry: LogEntry = {
    timestamp: new Date().toISOString(),
    event,
    seq: sequenceCounter,
    ...fields,
  };
  console.log(JSON.stringify(entry));
  logAccumulator.push(entry);
  return entry;
}

/** Returns a shallow copy of all accumulated log entries. */
export function getLogEntries(): ReadonlyArray<LogEntry> {
  return [...logAccumulator];
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
