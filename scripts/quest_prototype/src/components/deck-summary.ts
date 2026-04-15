import type { CardData, Rarity } from "../types/cards";
import type { DeckEntry } from "../types/quest";

export const ALL_RARITIES: readonly Rarity[] = [
  "Common",
  "Uncommon",
  "Rare",
  "Legendary",
] as const;

export const RARITY_ORDER: Readonly<Record<Rarity, number>> = {
  Common: 0,
  Uncommon: 1,
  Rare: 2,
  Legendary: 3,
};

export interface DeckSummaryEntry {
  rarity: Rarity;
  count: number;
  percentage: number;
}

export interface DeckSummary {
  total: number;
  characterCount: number;
  eventCount: number;
  averageEnergyCost: number | null;
  rarities: DeckSummaryEntry[];
}

/** Compare two rarities using the canonical deck-display order. */
export function compareRarities(a: Rarity, b: Rarity): number {
  return RARITY_ORDER[a] - RARITY_ORDER[b];
}

/** Computes package-safe summary metrics for a deck. */
export function computeDeckSummary(
  deck: readonly DeckEntry[],
  cardDatabase: Map<number, CardData>,
): DeckSummary {
  const rarityCounts: Record<Rarity, number> = {
    Common: 0,
    Uncommon: 0,
    Rare: 0,
    Legendary: 0,
  };
  let total = 0;
  let characterCount = 0;
  let eventCount = 0;
  let totalEnergyCost = 0;
  let cardsWithEnergyCost = 0;

  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card === undefined) {
      continue;
    }

    rarityCounts[card.rarity] += 1;
    total += 1;

    if (card.cardType === "Character") {
      characterCount += 1;
    } else {
      eventCount += 1;
    }

    if (card.energyCost !== null) {
      totalEnergyCost += card.energyCost;
      cardsWithEnergyCost += 1;
    }
  }

  return {
    total,
    characterCount,
    eventCount,
    averageEnergyCost:
      cardsWithEnergyCost > 0
        ? Math.round((totalEnergyCost / cardsWithEnergyCost) * 10) / 10
        : null,
    rarities: ALL_RARITIES.map((rarity) => ({
      rarity,
      count: rarityCounts[rarity],
      percentage:
        total > 0 ? Math.round((rarityCounts[rarity] / total) * 100) : 0,
    })),
  };
}
