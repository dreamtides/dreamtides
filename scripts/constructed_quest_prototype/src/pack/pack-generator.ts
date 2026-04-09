import type { CardData, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { weightedSample, duplicateWeight } from "../data/tide-weights";

/**
 * Generates loot pack contents for a given tide theme.
 * Filters the card database to matching-tide cards, applies duplicate
 * protection weights based on copies already in the player's pool,
 * then performs weighted sampling without replacement.
 * Approximately 50% of the time, one slot is replaced with a Neutral card.
 */
export function generateLootPack(
  cardDatabase: Map<number, CardData>,
  playerPool: ReadonlyArray<DeckEntry>,
  packTide: Tide,
  config: QuestConfig,
  isEnhanced: boolean,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === packTide && c.rarity !== "Starter",
  );

  if (candidates.length === 0) return [];

  // Count copies of each card number in the player's pool
  const copyCounts = new Map<number, number>();
  for (const entry of playerPool) {
    copyCounts.set(entry.cardNumber, (copyCounts.get(entry.cardNumber) ?? 0) + 1);
  }

  const packSize = isEnhanced
    ? config.lootPackSize * 2
    : config.lootPackSize;

  const tideCards = weightedSample(candidates, packSize, (card) => {
    const copies = copyCounts.get(card.cardNumber) ?? 0;
    return duplicateWeight(copies, config);
  });

  // Approximately 50% chance to replace one card with a Neutral
  const neutralCandidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === "Neutral" && c.rarity !== "Starter" && c.rarity !== "Legendary",
  );
  if (neutralCandidates.length > 0 && tideCards.length > 0 && Math.random() < 0.5) {
    const neutralCard = neutralCandidates[Math.floor(Math.random() * neutralCandidates.length)];
    tideCards[tideCards.length - 1] = neutralCard;
  }

  return tideCards;
}
