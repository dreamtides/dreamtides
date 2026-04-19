import type { BattleHistoryEntry, MarkerDiffState } from "../types";

export const BATTLE_MARKER_SET_EVENT = "battle_proto_marker_set";

export function diffMarkerValue(previous: boolean, next: boolean): MarkerDiffState {
  if (previous === next) {
    return "unchanged";
  }
  return next ? "set" : "cleared";
}

export function isMarkerDiffState(value: unknown): value is MarkerDiffState {
  return value === "set" || value === "cleared" || value === "unchanged";
}

/**
 * Shared reader for the marker-set diff on a history entry — bug-109. The
 * LogDrawer used to inline this lookup, which silently broke if the event
 * name or fields shape drifted. All consumers now go through this helper.
 */
export function readMarkerDiffFromHistoryEntry(
  entry: BattleHistoryEntry,
): { prevented: MarkerDiffState; copied: MarkerDiffState } | null {
  const event = entry.after.lastTransition?.logEvents.find(
    (logEvent) => logEvent.event === BATTLE_MARKER_SET_EVENT,
  );
  if (event === undefined) {
    return null;
  }
  const diff = event.fields.diff;
  if (
    typeof diff !== "object"
    || diff === null
    || !isMarkerDiffState((diff as { prevented: unknown }).prevented)
    || !isMarkerDiffState((diff as { copied: unknown }).copied)
  ) {
    return null;
  }
  return {
    prevented: (diff as { prevented: MarkerDiffState }).prevented,
    copied: (diff as { copied: MarkerDiffState }).copied,
  };
}
