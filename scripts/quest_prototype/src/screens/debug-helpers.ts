import type { CardData } from "../types/cards";
import type { DraftState } from "../types/draft";

/** Debug info for the player's fixed draft pool state. */
export interface DraftDebugInfo {
  currentOffer: CardData[];
  currentOfferSize: number;
  pickNumber: number;
  sitePicksCompleted: number;
  remainingCards: number;
  remainingUniqueCards: number;
}

/** Extract debug info from the current draft state. */
export function extractDraftDebugInfo(
  draftState: DraftState | null,
  cardDatabase: Map<number, CardData>,
): DraftDebugInfo | null {
  if (draftState === null) {
    return null;
  }

  return {
    currentOffer: draftState.currentOffer
      .map((cardNumber) => cardDatabase.get(cardNumber))
      .filter((card): card is CardData => card !== undefined),
    currentOfferSize: draftState.currentOffer.length,
    pickNumber: draftState.pickNumber,
    sitePicksCompleted: draftState.sitePicksCompleted,
    remainingCards: Object.values(draftState.remainingCopiesByCard).reduce(
      (total, copies) => total + copies,
      0,
    ),
    remainingUniqueCards: Object.keys(draftState.remainingCopiesByCard).length,
  };
}
