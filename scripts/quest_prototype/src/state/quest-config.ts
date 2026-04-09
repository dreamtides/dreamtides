import { useMemo } from "react";

/** Configuration derived from URL parameters. */
export interface QuestConfig {
  /** Number of core tides to exclude at quest start (0-4, default 2). */
  excludedTideCount: number;
  /** Whether to show tide cost symbols on cards (default true). */
  showTideSymbols: boolean;
  /** Whether to enable weighted pool bias for featured tides (default false). */
  poolBias: boolean;
}

const DEFAULT_EXCLUDED_TIDE_COUNT = 2;

/** Parses quest configuration from the current URL search parameters. */
export function getQuestConfig(): QuestConfig {
  const params = new URLSearchParams(window.location.search);

  const excludedTidesRaw = params.get("excludedTides");
  let excludedTideCount = DEFAULT_EXCLUDED_TIDE_COUNT;
  if (excludedTidesRaw !== null) {
    const parsed = parseInt(excludedTidesRaw, 10);
    if (!isNaN(parsed) && parsed >= 0 && parsed <= 4) {
      excludedTideCount = parsed;
    }
  }

  const showTideSymbolsRaw = params.get("showTideSymbols");
  const showTideSymbols = showTideSymbolsRaw !== "false";

  const poolBias = params.get("poolBias") === "true";

  return { excludedTideCount, showTideSymbols, poolBias };
}

/** React hook that returns quest configuration from URL parameters. */
export function useQuestConfig(): QuestConfig {
  return useMemo(() => getQuestConfig(), []);
}
