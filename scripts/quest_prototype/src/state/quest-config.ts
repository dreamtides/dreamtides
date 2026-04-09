import { useMemo } from "react";

/** Configuration derived from URL parameters. */
export interface QuestConfig {
  /** Whether to show tide cost symbols on cards (default false). */
  showTideSymbols: boolean;
}

/** Parses quest configuration from the current URL search parameters. */
export function getQuestConfig(): QuestConfig {
  const params = new URLSearchParams(window.location.search);

  const showTideSymbolsRaw = params.get("showTideSymbols");
  const showTideSymbols = showTideSymbolsRaw === "true";

  return { showTideSymbols };
}

/** React hook that returns quest configuration from URL parameters. */
export function useQuestConfig(): QuestConfig {
  return useMemo(() => getQuestConfig(), []);
}
