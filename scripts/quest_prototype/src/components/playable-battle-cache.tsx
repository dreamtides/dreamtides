import { createContext, useContext, type ReactNode } from "react";
import type { BattleInit, BattleMutableState } from "../battle/types";

export interface PlayableBattleSessionCacheEntry {
  snapshotKey: string;
  battleInit: BattleInit;
  initialStateTemplate: BattleMutableState;
}

/**
 * React-scoped cache for playable-mode battle bootstrap data. Each
 * `QuestProvider` instance owns its own `Map`, so sibling provider instances
 * (dev overlays, embedded demos, parallel tests) cannot alias on shared
 * `battleEntryKey` values (bug-013).
 */
export interface PlayableBattleCache {
  get: (battleEntryKey: string) => PlayableBattleSessionCacheEntry | undefined;
  set: (battleEntryKey: string, entry: PlayableBattleSessionCacheEntry) => void;
  reset: () => void;
}

const PlayableBattleCacheContext = createContext<PlayableBattleCache | null>(null);

export function createPlayableBattleCache(): PlayableBattleCache {
  const map = new Map<string, PlayableBattleSessionCacheEntry>();
  return {
    get: (battleEntryKey) => map.get(battleEntryKey),
    set: (battleEntryKey, entry) => {
      map.set(battleEntryKey, entry);
    },
    reset: () => {
      map.clear();
    },
  };
}

export function PlayableBattleCacheProvider({
  cache,
  children,
}: {
  cache: PlayableBattleCache;
  children: ReactNode;
}) {
  return (
    <PlayableBattleCacheContext.Provider value={cache}>
      {children}
    </PlayableBattleCacheContext.Provider>
  );
}

export function usePlayableBattleCache(): PlayableBattleCache {
  const cache = useContext(PlayableBattleCacheContext);
  if (cache === null) {
    throw new Error(
      "usePlayableBattleCache must be used within a PlayableBattleCacheProvider",
    );
  }
  return cache;
}
