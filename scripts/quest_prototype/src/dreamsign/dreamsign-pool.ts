import { createDreamsign } from "../data/dreamsigns";
import type { DreamsignTemplate } from "../types/content";
import type { Dreamsign } from "../types/quest";

export interface DreamsignPoolDraw {
  offeredIds: string[];
  offeredDreamsigns: Dreamsign[];
  remainingDreamsignPool: string[];
}

export interface DreamsignPoolState {
  availableIds: string[];
  templatesById: Map<string, DreamsignTemplate>;
}

function shufflePick<T>(items: readonly T[], count: number): T[] {
  const pool = [...items];
  for (let index = pool.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    [pool[index], pool[swapIndex]] = [pool[swapIndex], pool[index]];
  }
  return pool.slice(0, count);
}

/** Returns the canonical remaining Dreamsign ids backed by known templates. */
export function readDreamsignPool(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
): DreamsignPoolState {
  const templatesById = new Map(
    templates.map((template) => [template.id, template]),
  );
  const seenIds = new Set<string>();
  const availableIds: string[] = [];

  for (const id of remainingDreamsignPool) {
    if (seenIds.has(id) || !templatesById.has(id)) {
      continue;
    }

    seenIds.add(id);
    availableIds.push(id);
  }

  return {
    availableIds,
    templatesById,
  };
}

/** Spends Dreamsign ids from the canonical remaining pool. */
export function consumeDreamsignPoolIds(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
  spentIds: readonly string[],
): string[] {
  const { availableIds } = readDreamsignPool(remainingDreamsignPool, templates);
  if (spentIds.length === 0) {
    return availableIds;
  }

  const spentIdSet = new Set(spentIds);
  return availableIds.filter((id) => !spentIdSet.has(id));
}

/** Draws unique Dreamsigns from the shared run pool and spends them immediately. */
export function drawDreamsignOptions(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
  count: number,
): DreamsignPoolDraw {
  const { availableIds, templatesById } = readDreamsignPool(
    remainingDreamsignPool,
    templates,
  );
  const offeredIds = shufflePick(
    availableIds,
    Math.min(count, availableIds.length),
  );

  return {
    offeredIds,
    offeredDreamsigns: offeredIds.map((id) =>
      createDreamsign(templatesById.get(id)!),
    ),
    remainingDreamsignPool: consumeDreamsignPoolIds(
      remainingDreamsignPool,
      templates,
      offeredIds,
    ),
  };
}

/** Resolves the currently available Dreamsign templates from a shared pool. */
export function resolveDreamsignTemplates(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
): DreamsignTemplate[] {
  const { availableIds, templatesById } = readDreamsignPool(
    remainingDreamsignPool,
    templates,
  );

  return availableIds.map((id) => templatesById.get(id)!);
}
