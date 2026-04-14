import { createDreamsign } from "../data/dreamsigns";
import type { DreamsignTemplate } from "../types/content";
import type { Dreamsign } from "../types/quest";

export interface DreamsignPoolDraw {
  offeredIds: string[];
  offeredDreamsigns: Dreamsign[];
  remainingDreamsignPool: string[];
}

function shufflePick<T>(items: readonly T[], count: number): T[] {
  const pool = [...items];
  for (let index = pool.length - 1; index > 0; index -= 1) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    [pool[index], pool[swapIndex]] = [pool[swapIndex], pool[index]];
  }
  return pool.slice(0, count);
}

/** Draws unique Dreamsigns from the shared run pool and spends them immediately. */
export function drawDreamsignOptions(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
  count: number,
): DreamsignPoolDraw {
  const templatesById = new Map(
    templates.map((template) => [template.id, template]),
  );
  const availableIds = remainingDreamsignPool.filter((id) =>
    templatesById.has(id),
  );
  const offeredIds = shufflePick(availableIds, Math.min(count, availableIds.length));
  const spentIds = new Set(offeredIds);

  return {
    offeredIds,
    offeredDreamsigns: offeredIds.map((id) =>
      createDreamsign(templatesById.get(id)!),
    ),
    remainingDreamsignPool: remainingDreamsignPool.filter((id) => !spentIds.has(id)),
  };
}

/** Resolves the currently available Dreamsign templates from a shared pool. */
export function resolveDreamsignTemplates(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
): DreamsignTemplate[] {
  const templatesById = new Map(
    templates.map((template) => [template.id, template]),
  );
  return remainingDreamsignPool
    .map((id) => templatesById.get(id))
    .filter((template): template is DreamsignTemplate => template !== undefined);
}
