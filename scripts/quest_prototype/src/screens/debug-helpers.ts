import type { Tide, CardData } from "../types/cards";
import type { DraftState } from "../types/draft";
import { NAMED_TIDES } from "../data/card-database";

/** Ordered tide names matching the 7-element preference vector layout. */
const TIDE_VECTOR_ORDER: readonly Tide[] = [
  "Bloom",
  "Arc",
  "Ignite",
  "Pact",
  "Umbra",
  "Rime",
  "Surge",
] as const;

/** Summary of a single bot's draft state for display. */
export interface BotSummary {
  seatIndex: number;
  primaryTide: Tide | null;
  secondaryTide: Tide | null;
  preferenceWeights: Record<string, number>;
  draftedCards: CardData[];
  cardsByTide: Record<string, number>;
  totalCards: number;
}

/** Extract and sort bot summaries from draft state for display. */
export function extractBotSummaries(
  draftState: DraftState | null,
  cardDatabase: Map<number, CardData>,
): BotSummary[] {
  if (draftState === null) {
    return [];
  }

  const summaries: BotSummary[] = [];

  for (let i = 1; i < draftState.agents.length; i++) {
    const agent = draftState.agents[i];
    const pref = agent.preference;

    const normalizedWeights = normalizePreference(pref);

    const sorted = tidesSortedByWeight(normalizedWeights);
    const primaryTide: Tide | null =
      sorted.length > 0 && normalizedWeights[sorted[0]] > 0
        ? (sorted[0] as Tide)
        : null;
    const secondaryTide: Tide | null =
      sorted.length > 1 && normalizedWeights[sorted[1]] > 0
        ? (sorted[1] as Tide)
        : null;

    const draftedCards: CardData[] = [];
    const cardsByTide: Record<string, number> = {};
    for (const tide of [...NAMED_TIDES, "Wild" as Tide]) {
      cardsByTide[tide] = 0;
    }

    for (const cardNum of agent.picks) {
      const card = cardDatabase.get(cardNum);
      if (card) {
        draftedCards.push(card);
        cardsByTide[card.tide] = (cardsByTide[card.tide] ?? 0) + 1;
      }
    }

    summaries.push({
      seatIndex: i,
      primaryTide,
      secondaryTide,
      preferenceWeights: normalizedWeights,
      draftedCards,
      cardsByTide,
      totalCards: draftedCards.length,
    });
  }

  summaries.sort((a, b) => {
    const aTide = a.primaryTide ?? "";
    const bTide = b.primaryTide ?? "";
    if (aTide !== bTide) {
      return aTide.localeCompare(bTide);
    }
    return a.seatIndex - b.seatIndex;
  });

  return summaries;
}

/** Normalize preference vector into a Record keyed by tide name with values summing to 1. */
function normalizePreference(pref: number[]): Record<string, number> {
  const result: Record<string, number> = {};
  const sum = pref.reduce((s, v) => s + v, 0);

  for (let i = 0; i < TIDE_VECTOR_ORDER.length; i++) {
    const tideName = TIDE_VECTOR_ORDER[i];
    result[tideName] = sum > 0 ? pref[i] / sum : 0;
  }

  return result;
}

/** Return tide names sorted by descending weight. */
function tidesSortedByWeight(weights: Record<string, number>): string[] {
  return Object.entries(weights)
    .sort(([, a], [, b]) => b - a)
    .map(([tide]) => tide);
}
