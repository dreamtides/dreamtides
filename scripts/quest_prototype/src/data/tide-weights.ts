import type { CardData } from "../types/cards";
import type { PackageTideId } from "../types/content";
import { isStarterCard } from "./card-database";
import { packageAdjacentCandidatesOrFallback } from "./quest-content";

export function weightedSample<T>(
  pool: ReadonlyArray<T>,
  count: number,
  weightFn: (item: T) => number,
): T[] {
  const weighted: Array<[T, number]> = pool.map((item) => [
    item,
    Math.max(0, weightFn(item)),
  ]);
  const selected: T[] = [];
  const remaining = [...weighted];

  for (let pick = 0; pick < count && remaining.length > 0; pick += 1) {
    const total = remaining.reduce((sum, [, weight]) => sum + weight, 0);
    if (total <= 0) {
      selected.push(remaining.shift()![0]);
      continue;
    }

    let roll = Math.random() * total;
    let chosenIndex = remaining.length - 1;

    for (let index = 0; index < remaining.length; index += 1) {
      roll -= remaining[index][1];
      if (roll <= 0) {
        chosenIndex = index;
        break;
      }
    }

    selected.push(remaining[chosenIndex][0]);
    remaining.splice(chosenIndex, 1);
  }

  return selected;
}

export function pickPackageAdjacentItem<T>(
  items: readonly T[],
  packageTides: (item: T) => readonly PackageTideId[],
  selectedPackageTides: readonly PackageTideId[],
): T | null {
  return (
    samplePackageAdjacentItems(items, 1, packageTides, selectedPackageTides)[0] ??
    null
  );
}

export function samplePackageAdjacentItems<T>(
  items: readonly T[],
  count: number,
  packageTides: (item: T) => readonly PackageTideId[],
  selectedPackageTides: readonly PackageTideId[],
): T[] {
  const candidates = packageAdjacentCandidatesOrFallback(
    items,
    packageTides,
    selectedPackageTides,
  );

  return weightedSample(candidates, count, (candidate) => candidate.overlapCount).map(
    (candidate) => candidate.item,
  );
}

/** Selects rare rewards from the package-adjacent pool without touching the draft multiset. */
export function sampleRewardCards(
  cardDatabase: ReadonlyMap<number, CardData>,
  count: number,
  selectedPackageTides: readonly PackageTideId[] = [],
): CardData[] {
  return samplePackageAdjacentItems(
    Array.from(cardDatabase.values()).filter((card) => !isStarterCard(card)),
    count,
    (card) => card.tides,
    selectedPackageTides,
  );
}

export function selectBattleRewards(
  cardDatabase: ReadonlyMap<number, CardData>,
  selectedPackageTides: readonly PackageTideId[] = [],
): CardData[] {
  return sampleRewardCards(cardDatabase, 4, selectedPackageTides);
}
