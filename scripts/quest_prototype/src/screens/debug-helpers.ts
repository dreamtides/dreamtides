import type { Tide, CardData } from "../types/cards";
import type { DraftState, PackStrategy } from "../types/draft";
import { NAMED_TIDES } from "../data/card-database";

/** Debug info for the player's draft state. */
export interface DraftDebugInfo {
  draftedCards: CardData[];
  cardsByTide: Record<string, number>;
  totalCards: number;
  pickNumber: number;
  poolSize: number;
  seenCards: number;
  packStrategy: PackStrategy;
  chosenTide: Tide | null;
}

/** Extract debug info from the current draft state. */
export function extractDraftDebugInfo(
  draftState: DraftState | null,
  cardDatabase: Map<number, CardData>,
  chosenTide: Tide | null = null,
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

  return {
    draftedCards,
    cardsByTide,
    totalCards: draftedCards.length,
    pickNumber: draftState.pickNumber,
    poolSize: draftState.pool.length,
    seenCards: draftState.seenCards.length,
    packStrategy: draftState.packStrategy,
    chosenTide,
  };
}
