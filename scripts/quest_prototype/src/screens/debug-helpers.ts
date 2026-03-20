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

/** Summary of a single seat's draft state for display. */
export interface SeatSummary {
  seatIndex: number;
  isPlayer: boolean;
  receivesFromSeat: number;
  primaryTide: Tide | null;
  secondaryTide: Tide | null;
  preferenceWeights: Record<string, number>;
  draftedCards: CardData[];
  cardsByTide: Record<string, number>;
  totalCards: number;
}

/** Debug info for the entire draft table. */
export interface DraftDebugInfo {
  seats: SeatSummary[];
  currentRound: number;
  displayRound: number;
  seatPassingToPlayer: number;
}

/** Extract seat summaries and passing info from draft state. */
export function extractDraftDebugInfo(
  draftState: DraftState | null,
  cardDatabase: Map<number, CardData>,
): DraftDebugInfo | null {
  if (draftState === null) {
    return null;
  }

  const seatCount = draftState.agents.length;
  const currentRound = draftState.currentRound;

  const seats: SeatSummary[] = [];

  for (let i = 0; i < seatCount; i++) {
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
    for (const tide of [...NAMED_TIDES, "Neutral" as Tide]) {
      cardsByTide[tide] = 0;
    }

    for (const cardNum of agent.picks) {
      const card = cardDatabase.get(cardNum);
      if (card) {
        draftedCards.push(card);
        cardsByTide[card.tide] = (cardsByTide[card.tide] ?? 0) + 1;
      }
    }

    // Packs always pass left: seat (i-1) passes to seat i
    const receivesFromSeat = (i - 1 + seatCount) % seatCount;

    seats.push({
      seatIndex: i,
      isPlayer: i === 0,
      receivesFromSeat,
      primaryTide,
      secondaryTide,
      preferenceWeights: normalizedWeights,
      draftedCards,
      cardsByTide,
      totalCards: draftedCards.length,
    });
  }

  // Seat (count-1) always passes to seat 0
  const seatPassingToPlayer = seatCount - 1;

  return {
    seats,
    currentRound,
    displayRound: currentRound + 1,
    seatPassingToPlayer,
  };
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
