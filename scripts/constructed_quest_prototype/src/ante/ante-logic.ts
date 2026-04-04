import type { CardData } from "../types/cards";
import type { AnteState, DeckEntry } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { countDeckTides, weightedSample, tideWeight } from "../data/tide-weights";

/**
 * Generates opponent ante cards weighted toward the player's dominant tides
 * and higher rarity. Returns an array of cardNumbers (1-2 cards).
 */
export function generateOpponentAnteCards(
  cardDatabase: Map<number, CardData>,
  playerPool: DeckEntry[],
  config: QuestConfig,
): number[] {
  const deckTideCounts = countDeckTides(playerPool, cardDatabase);
  const allCards = Array.from(cardDatabase.values());

  if (allCards.length === 0) return [];

  const count = Math.min(config.maxAnteCards, 2);

  const selected = weightedSample(allCards, count, (card) => {
    const tw = tideWeight(card.tide, deckTideCounts);
    const rarityBonus =
      card.rarity === "Legendary"
        ? 3
        : card.rarity === "Rare"
          ? 2
          : card.rarity === "Uncommon"
            ? 1.5
            : 1;
    return tw * rarityBonus;
  });

  return selected.map((c) => c.cardNumber);
}

/**
 * AI escalation decision. Since the player always wins, the AI is always
 * "behind" and has a 50% chance to bluff-escalate.
 */
export function aiEscalationDecision(isAhead: boolean): boolean {
  if (isAhead) return true;
  return Math.random() < 0.5;
}

/**
 * Resolves the ante outcome after battle completion.
 * Returns cards gained and lost by the player.
 */
export function resolveAnte(
  anteState: AnteState,
  playerWon: boolean,
): { cardsGained: number[]; cardsLost: number[] } {
  if (!anteState.anteAccepted) {
    return { cardsGained: [], cardsLost: [] };
  }

  if (anteState.playerConceded) {
    // Conceded: lose only the first ante card (not escalation card)
    const firstCard = anteState.playerAnteCards[0];
    return {
      cardsGained: [],
      cardsLost: firstCard !== undefined ? [firstCard] : [],
    };
  }

  if (playerWon) {
    return {
      cardsGained: [...anteState.opponentAnteCards],
      cardsLost: [],
    };
  }

  // Lost with ante accepted (shouldn't happen since player always wins,
  // but handle for completeness)
  return {
    cardsGained: [],
    cardsLost: [...anteState.playerAnteCards],
  };
}
