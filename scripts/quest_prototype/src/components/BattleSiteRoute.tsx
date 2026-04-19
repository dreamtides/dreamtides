import type { CardData } from "../types/cards";
import type { SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import type { RuntimeConfig } from "../runtime/runtime-config";
import { PlayableBattleScreen } from "../battle/components/PlayableBattleScreen";
import { prepareInitialBattleState } from "../battle/engine/turn-flow";
import { createBattleInit } from "../battle/integration/create-battle-init";
import { cloneBattleMutableState, createInitialBattleState } from "../battle/state/create-initial-state";
import type { BattleInit, BattleMutableState } from "../battle/types";
import { AutoBattleScreen } from "../screens/AutoBattleScreen";
import {
  usePlayableBattleCache,
  type PlayableBattleCache,
} from "./playable-battle-cache";

export function createBattleEntryKey(
  dreamscapeId: string | null,
  siteId: string,
  completionLevel: number,
): string {
  return `${siteId}::${String(completionLevel)}::${dreamscapeId ?? "none"}`;
}

/**
 * Spec A-5 `BattleScreen` wrapper. Named `BattleSiteRoute` in code because it
 * also owns the site-context cache lookup (`battleEntryKey` → session), but it
 * is the component the spec refers to as the battle-mode dispatcher between
 * `AutoBattleScreen` and `PlayableBattleScreen` (bug-026).
 */
export function BattleSiteRoute({
  site,
  cardDatabase,
  runtimeConfig,
}: {
  site: SiteState;
  cardDatabase: Map<number, CardData>;
  runtimeConfig: RuntimeConfig;
}) {
  const { state, questContent } = useQuest();
  const cache = usePlayableBattleCache();
  const battleEntryKey = createBattleEntryKey(
    state.currentDreamscape,
    site.id,
    state.completionLevel,
  );

  const cached = getOrCreateBattleSession(
    cache,
    battleEntryKey,
    site,
    state,
    cardDatabase,
    questContent.dreamcallers,
    runtimeConfig.seedOverride,
  );

  if (runtimeConfig.battleMode === "playable") {
    return (
      <PlayableBattleScreen
        key={battleEntryKey}
        battleInit={cached.battleInit}
        initialState={cloneBattleMutableState(cached.initialStateTemplate)}
        site={site}
      />
    );
  }

  return (
    <AutoBattleScreen
      key={battleEntryKey}
      battleInit={cached.battleInit}
      site={site}
    />
  );
}

interface BattleSessionCacheValue {
  battleInit: BattleInit;
  initialStateTemplate: BattleMutableState;
}

function getOrCreateBattleSession(
  cache: PlayableBattleCache,
  battleEntryKey: string,
  site: SiteState,
  state: ReturnType<typeof useQuest>["state"],
  cardDatabase: ReadonlyMap<number, CardData>,
  dreamcallers: ReturnType<typeof useQuest>["questContent"]["dreamcallers"],
  seedOverride: number | null,
): BattleSessionCacheValue {
  const snapshotKey = createPlayableBattleCacheSnapshotKey(seedOverride);
  const cached = cache.get(battleEntryKey);
  if (cached !== undefined && cached.snapshotKey === snapshotKey) {
    return {
      battleInit: cached.battleInit,
      initialStateTemplate: cached.initialStateTemplate,
    };
  }

  const battleInit = createBattleInit({
    battleEntryKey,
    site,
    state,
    cardDatabase,
    dreamcallers,
    seedOverride,
  });
  const created = {
    snapshotKey,
    battleInit,
    initialStateTemplate: prepareInitialBattleState(
      createInitialBattleState(battleInit),
      battleInit,
    ).state,
  };
  cache.set(battleEntryKey, created);
  return {
    battleInit,
    initialStateTemplate: created.initialStateTemplate,
  };
}

/**
 * Snapshot identity for a cached battle session. `battleEntryKey` already
 * encodes `siteId`, `completionLevel`, and `dreamscapeId`; the snapshot key
 * only needs to cover fields *orthogonal* to the cache bucket (bug-010). Today
 * that is just `seedOverride`, which re-seeds an otherwise identical entry.
 */
function createPlayableBattleCacheSnapshotKey(
  seedOverride: number | null,
): string {
  return `seed:${seedOverride === null ? "none" : String(seedOverride)}`;
}
