import type { CardData, Tide } from "../types/cards";
import { cardAccentTide, NAMED_TIDES } from "../data/card-database";
import type { DeckEntry } from "../types/quest";

/** All tides including Neutral, used for distribution computation. */
const ALL_TIDES: readonly Tide[] = [...NAMED_TIDES, "Neutral"] as const;

/** A single tide's distribution entry. */
export interface TideDistributionEntry {
  tide: Tide;
  count: number;
  percentage: number;
  isDominant: boolean;
}

/** The full tide distribution result for a deck. */
export interface TideDistribution {
  tides: TideDistributionEntry[];
  total: number;
}

/**
 * Computes the tide distribution for a deck, returning counts, percentages,
 * and dominant tide indicators. Cards not found in the database are skipped.
 */
export function computeTideDistribution(
  deck: DeckEntry[],
  cardDatabase: Map<number, CardData>,
): TideDistribution {
  const counts: Record<string, number> = {};
  for (const tide of ALL_TIDES) {
    counts[tide] = 0;
  }

  let total = 0;
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card === undefined) continue;
    const accentTide = cardAccentTide(card);
    counts[accentTide] = (counts[accentTide] ?? 0) + 1;
    total += 1;
  }

  const maxCount = Math.max(0, ...ALL_TIDES.map((t) => counts[t] ?? 0));

  const tides: TideDistributionEntry[] = ALL_TIDES.map((tide) => {
    const count = counts[tide] ?? 0;
    return {
      tide,
      count,
      percentage: total > 0 ? Math.round((count / total) * 100) : 0,
      isDominant: count > 0 && count === maxCount,
    };
  });

  return { tides, total };
}
