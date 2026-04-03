import type { Tide, CardData } from "../types/cards";
import type { DraftState } from "../types/draft";
import { NAMED_TIDES } from "../data/card-database";
import { computeTideAffinity, computeFocus } from "../draft/draft-engine";

/** Debug info for the player's draft state. */
export interface DraftDebugInfo {
  draftedCards: CardData[];
  cardsByTide: Record<string, number>;
  totalCards: number;
  pickNumber: number;
  poolSize: number;
  focus: number;
  tideAffinities: Record<string, number>;
  primaryTide: Tide | null;
  secondaryTide: Tide | null;
}

/** Extract debug info from the current draft state. */
export function extractDraftDebugInfo(
  draftState: DraftState | null,
  cardDatabase: Map<number, CardData>,
): DraftDebugInfo | null {
  if (draftState === null) {
    return null;
  }

  const draftedCards: CardData[] = [];
  const cardsByTide: Record<string, number> = {};
  for (const tide of [...NAMED_TIDES, "Neutral" as Tide]) {
    cardsByTide[tide] = 0;
  }

  for (const cardNum of draftState.draftedCards) {
    const card = cardDatabase.get(cardNum);
    if (card) {
      draftedCards.push(card);
      cardsByTide[card.tide] = (cardsByTide[card.tide] ?? 0) + 1;
    }
  }

  const affinity = computeTideAffinity(
    draftState.draftedCards,
    cardDatabase,
  );
  const tideAffinities: Record<string, number> = {};
  for (const [tide, value] of affinity.entries()) {
    tideAffinities[tide] = Math.round(value * 100) / 100;
  }

  const sorted = Object.entries(tideAffinities)
    .filter(([tide]) => tide !== "Neutral")
    .sort(([, a], [, b]) => b - a);
  const primaryTide: Tide | null =
    sorted.length > 0 && sorted[0][1] > 1.0 ? (sorted[0][0] as Tide) : null;
  const secondaryTide: Tide | null =
    sorted.length > 1 && sorted[1][1] > 1.0 ? (sorted[1][0] as Tide) : null;

  return {
    draftedCards,
    cardsByTide,
    totalCards: draftedCards.length,
    pickNumber: draftState.pickNumber,
    poolSize: draftState.pool.length,
    focus: Math.round(computeFocus(draftState.pickNumber) * 100) / 100,
    tideAffinities,
    primaryTide,
    secondaryTide,
  };
}
