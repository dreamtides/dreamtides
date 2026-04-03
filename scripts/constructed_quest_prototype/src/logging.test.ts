import { describe, it, expect, beforeEach, vi } from "vitest";
import { logEvent, getLogEntries, resetLog } from "./logging";

beforeEach(() => {
  resetLog();
  vi.restoreAllMocks();
});

describe("logEvent", () => {
  it("assigns incrementing sequence numbers starting at 1", () => {
    const a = logEvent("alpha");
    const b = logEvent("beta");
    expect(a.seq).toBe(1);
    expect(b.seq).toBe(2);
  });

  it("includes timestamp, event, and seq on every entry", () => {
    const entry = logEvent("test_event", { extra: 42 });
    expect(entry.timestamp).toBeDefined();
    expect(typeof entry.timestamp).toBe("string");
    expect(entry.event).toBe("test_event");
    expect(entry.seq).toBe(1);
    expect(entry.extra).toBe(42);
  });

  it("writes single-line JSON to console.log", () => {
    const spy = vi.spyOn(console, "log").mockImplementation(() => {});
    logEvent("console_test", { key: "value" });
    expect(spy).toHaveBeenCalledOnce();
    const logged = spy.mock.calls[0][0] as string;
    const parsed = JSON.parse(logged) as Record<string, unknown>;
    expect(parsed.event).toBe("console_test");
    expect(parsed.key).toBe("value");
    expect(logged).not.toContain("\n");
  });

  it("strips reserved keys from caller-supplied fields", () => {
    const entry = logEvent("secure_event", {
      seq: 999,
      event: "tampered",
      timestamp: "bad",
      custom: "kept",
    });
    expect(entry.seq).toBe(1);
    expect(entry.event).toBe("secure_event");
    expect(entry.timestamp).not.toBe("bad");
    expect(entry.custom).toBe("kept");
  });

  it("returns a frozen object that cannot be mutated", () => {
    const entry = logEvent("frozen_test");
    expect(() => {
      (entry as Record<string, unknown>).event = "mutated";
    }).toThrow();
  });
});

describe("getLogEntries", () => {
  it("returns all accumulated entries", () => {
    logEvent("a");
    logEvent("b");
    logEvent("c");
    const entries = getLogEntries();
    expect(entries).toHaveLength(3);
    expect(entries[0].event).toBe("a");
    expect(entries[2].event).toBe("c");
  });

  it("returns copies that do not share references with internal state", () => {
    logEvent("original");
    const first = getLogEntries();
    logEvent("second");
    const second = getLogEntries();
    expect(first).toHaveLength(1);
    expect(second).toHaveLength(2);
  });

  it("returns frozen entry objects", () => {
    logEvent("freeze_check");
    const entries = getLogEntries();
    expect(() => {
      (entries[0] as Record<string, unknown>).event = "mutated";
    }).toThrow();
  });
});

describe("resetLog", () => {
  it("clears the accumulator and resets the sequence counter", () => {
    logEvent("before_reset");
    logEvent("before_reset_2");
    expect(getLogEntries()).toHaveLength(2);

    resetLog();

    expect(getLogEntries()).toHaveLength(0);
    const entry = logEvent("after_reset");
    expect(entry.seq).toBe(1);
  });
});
