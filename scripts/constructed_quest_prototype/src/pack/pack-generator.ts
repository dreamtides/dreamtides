import type { CardData, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { weightedSample, duplicateWeight } from "../data/tide-weights";

/**
 * Generates loot pack contents for a given tide theme.
 * Filters the card database to matching-tide cards, applies duplicate
 * protection weights based on copies already in the player's pool,
 * then performs weighted sampling without replacement.
 */
export function generateLootPack(
  cardDatabase: Map<number, CardData>,
  playerPool: ReadonlyArray<DeckEntry>,
  packTide: Tide,
  config: QuestConfig,
  isEnhanced: boolean,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === packTide,
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

  return weightedSample(candidates, packSize, (card) => {
    const copies = copyCounts.get(card.cardNumber) ?? 0;
    return duplicateWeight(copies, config);
  });
}
