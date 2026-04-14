import { cardAccentTide } from "./card-database";
import type { CardData, Tide } from "../types/cards";

/** Counts tide occurrences in the player's deck. */
export function countDeckTides(
  deck: ReadonlyArray<{ cardNumber: number }>,
  cardDatabase: ReadonlyMap<number, CardData>,
): Map<Tide, number> {
  const counts = new Map<Tide, number>();
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card) {
      const accentTide = cardAccentTide(card);
      counts.set(accentTide, (counts.get(accentTide) ?? 0) + 1);
    }
  }
  return counts;
}

/**
 * Performs a weighted random selection of `count` items from `pool`.
 * Items with higher weights are more likely to be selected.
 * Selected items are not re-picked (sampling without replacement).
 */
export function weightedSample<T>(
  pool: ReadonlyArray<T>,
  count: number,
  weightFn: (item: T) => number,
): T[] {
  const weighted: Array<[T, number]> = pool.map((item) => [
    item,
    weightFn(item),
  ]);

  const selected: T[] = [];
  const remaining = [...weighted];

  for (let pick = 0; pick < count && remaining.length > 0; pick++) {
    const total = remaining.reduce((sum, [, w]) => sum + w, 0);
    let roll = Math.random() * total;
    let chosenIndex = remaining.length - 1;

    for (let i = 0; i < remaining.length; i++) {
      roll -= remaining[i][1];
      if (roll <= 0) {
        chosenIndex = i;
        break;
      }
    }

    selected.push(remaining[chosenIndex][0]);
    remaining.splice(chosenIndex, 1);
  }

  return selected;
}

/**
 * Computes a weight for a tide-bearing item based on how common that
 * tide is in the player's deck. Items matching the player's dominant
 * tides receive up to 4x the base weight.
 */
export function tideWeight(
  tide: Tide,
  deckTideCounts: ReadonlyMap<Tide, number>,
): number {
  const maxCount = Math.max(...deckTideCounts.values(), 1);
  const tideCount = deckTideCounts.get(tide) ?? 0;
  return 1 + (tideCount / maxCount) * 3;
}

/**
 * Selects rare cards weighted toward the player's deck tides.
 * Draws from an infinite pool (does not deplete the draft cube).
 */
export function selectRareRewards(
  cardDatabase: Map<number, CardData>,
  deckTideCounts: Map<Tide, number>,
  excludedTides: Tide[] = [],
): CardData[] {
  const excludedSet = new Set(excludedTides);
  const rareCards = Array.from(cardDatabase.values()).filter(
    (card) => card.rarity === "Rare" && !excludedSet.has(cardAccentTide(card)),
  );

  if (rareCards.length === 0) return [];

  return weightedSample(rareCards, 4, (card) =>
    tideWeight(cardAccentTide(card), deckTideCounts),
  );
}
