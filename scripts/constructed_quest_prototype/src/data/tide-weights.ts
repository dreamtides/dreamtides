import type { CardData, NamedTide, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { adjacentTides, NAMED_TIDES } from "./card-database";

/** Derives seed tides from a starting tide: the tide itself + its two neighbors. */
export function startingTideSeedTides(startingTide: NamedTide | null): Tide[] {
  if (startingTide === null) return [];
  return [startingTide, ...adjacentTides(startingTide)];
}

/** Counts tide occurrences in the player's deck. */
export function countDeckTides(
  deck: ReadonlyArray<{ cardNumber: number }>,
  cardDatabase: ReadonlyMap<number, { tide: Tide }>,
): Map<Tide, number> {
  const counts = new Map<Tide, number>();
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card) {
      counts.set(card.tide, (counts.get(card.tide) ?? 0) + 1);
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
    (c) => c.rarity === "Rare" && !excludedSet.has(c.tide),
  );

  if (rareCards.length === 0) return [];

  return weightedSample(rareCards, 4, (card) =>
    tideWeight(card.tide, deckTideCounts),
  );
}

/**
 * Selects a pack tide for a LootPack site based on the player's pool composition.
 * Weighted random: on-theme (dominant tides), adjacent (neighbors of dominant), or explore (fewest cards).
 * Re-rolls duplicates of existingPackTides up to 10 attempts.
 */
export function selectPackTide(
  playerPool: ReadonlyArray<{ cardNumber: number }>,
  cardDatabase: ReadonlyMap<number, CardData>,
  config: QuestConfig,
  existingPackTides: Tide[],
): Tide {
  const tideCounts = countDeckTides(playerPool, cardDatabase);

  // Compute dominant tides: top 2-3 by card count
  const sortedTides = NAMED_TIDES.map((t) => ({
    tide: t,
    count: tideCounts.get(t) ?? 0,
  })).sort((a, b) => b.count - a.count);

  const dominantTides: Tide[] = [];
  if (sortedTides.length > 0) {
    dominantTides.push(sortedTides[0].tide);
    if (sortedTides.length > 1 && sortedTides[1].count > 0) {
      dominantTides.push(sortedTides[1].tide);
    }
    if (sortedTides.length > 2 && sortedTides[2].count > 0 && sortedTides[2].count >= sortedTides[1].count * 0.5) {
      dominantTides.push(sortedTides[2].tide);
    }
  }

  // If no dominant tides, fall back to all named tides
  const effectiveDominant = dominantTides.length > 0 ? dominantTides : [...NAMED_TIDES];

  // Adjacent tides of dominant
  const adjacentSet = new Set<Tide>();
  for (const t of effectiveDominant) {
    for (const adj of adjacentTides(t)) {
      if (!effectiveDominant.includes(adj)) {
        adjacentSet.add(adj);
      }
    }
  }
  const adjacentArray = Array.from(adjacentSet);

  // Explore tides: fewest cards in pool
  const exploreTides = NAMED_TIDES.filter(
    (t) => !effectiveDominant.includes(t) && !adjacentSet.has(t),
  );
  // If no explore tides, fall back to adjacent
  const effectiveExplore = exploreTides.length > 0 ? exploreTides : adjacentArray.length > 0 ? adjacentArray : effectiveDominant;

  const totalWeight = config.packOnTheme + config.packAdjacent + config.packExplore;

  for (let attempt = 0; attempt < 10; attempt++) {
    const roll = Math.random() * totalWeight;
    let picked: Tide;

    if (roll < config.packOnTheme) {
      picked = effectiveDominant[Math.floor(Math.random() * effectiveDominant.length)];
    } else if (roll < config.packOnTheme + config.packAdjacent) {
      const pool = adjacentArray.length > 0 ? adjacentArray : effectiveDominant;
      picked = pool[Math.floor(Math.random() * pool.length)];
    } else {
      picked = effectiveExplore[Math.floor(Math.random() * effectiveExplore.length)];
    }

    if (!existingPackTides.includes(picked) || attempt === 9) {
      return picked;
    }
  }

  // Fallback (should not reach here)
  return effectiveDominant[0];
}

/** Returns a weight multiplier based on how many copies the player already owns. */
export function duplicateWeight(copiesOwned: number, config: QuestConfig): number {
  if (copiesOwned <= 1) return 1.0;
  if (copiesOwned === 2) return 1.0 - config.dupePenalty2 / 100;
  return 1.0 - config.dupePenalty3 / 100;
}

/**
 * For each tide, computes the number of pool cards minus deck cards of that tide.
 * Used by the Forge system to identify which tides the player has surplus cards in.
 */
export function excessCardsByTide(
  playerPool: DeckEntry[],
  playerDeck: DeckEntry[],
  cardDatabase: Map<number, CardData>,
): Map<Tide, number> {
  const poolCounts = countDeckTides(playerPool, cardDatabase);
  const deckCounts = countDeckTides(playerDeck, cardDatabase);
  const result = new Map<Tide, number>();
  for (const tide of NAMED_TIDES) {
    const excess = (poolCounts.get(tide) ?? 0) - (deckCounts.get(tide) ?? 0);
    result.set(tide, excess);
  }
  return result;
}
