import { createDreamsign } from "../data/dreamsigns";
import type { DreamsignTemplate, PackageTideId } from "../types/content";
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

function isDreamsignEligible(
  template: DreamsignTemplate,
  selectedPackageTides: readonly PackageTideId[],
): boolean {
  if (selectedPackageTides.length === 0) {
    return true;
  }

  if ((template.packageTides?.length ?? 0) === 0) {
    return true;
  }

  return template.packageTides!.some((packageTideId) =>
    selectedPackageTides.includes(packageTideId),
  );
}

function canonicalizeDreamsignPool(
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
  return canonicalizeDreamsignPool(remainingDreamsignPool, templates);
}

/** Draws unique Dreamsigns from the shared run pool and spends them immediately. */
export function drawDreamsignOptions(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
  selectedPackageTides: readonly PackageTideId[],
  count: number,
): DreamsignPoolDraw {
  const { availableIds, templatesById } = canonicalizeDreamsignPool(
    remainingDreamsignPool,
    templates,
  );
  const eligibleIds = availableIds.filter((id) =>
    isDreamsignEligible(templatesById.get(id)!, selectedPackageTides),
  );
  const offeredIds = shufflePick(
    eligibleIds,
    Math.min(count, eligibleIds.length),
  );

  return {
    offeredIds,
    offeredDreamsigns: offeredIds.map((id) =>
      createDreamsign(templatesById.get(id)!),
    ),
    remainingDreamsignPool:
      offeredIds.length === 0
        ? availableIds
        : availableIds.filter((id) => !offeredIds.includes(id)),
  };
}

/** Resolves the currently available Dreamsign templates from a shared pool. */
export function resolveDreamsignTemplates(
  remainingDreamsignPool: readonly string[],
  templates: readonly DreamsignTemplate[],
  selectedPackageTides: readonly PackageTideId[] = [],
): DreamsignTemplate[] {
  const { availableIds, templatesById } = readDreamsignPool(
    remainingDreamsignPool,
    templates,
  );

  return availableIds
    .filter((id) => isDreamsignEligible(templatesById.get(id)!, selectedPackageTides))
    .map((id) => templatesById.get(id)!);
}
