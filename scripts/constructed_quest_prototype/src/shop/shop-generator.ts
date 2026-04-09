import type { CardData, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { countDeckTides, tideWeight, weightedSample } from "../data/tide-weights";

/** A single slot in the card shop inventory. */
export interface ShopSlot {
  card: CardData;
  basePrice: number;
  discountPercent: number;
  purchased: boolean;
}

/** Returns the effective price of a slot after discount. */
export function effectivePrice(slot: ShopSlot): number {
  if (slot.discountPercent === 0) return slot.basePrice;
  return Math.round(slot.basePrice * (1 - slot.discountPercent / 100));
}

/** Computes reroll cost given the number of previous rerolls. */
export function rerollCost(
  rerollCount: number,
  isEnhanced: boolean,
  config: QuestConfig,
): number {
  if (isEnhanced) return 0;
  return config.rerollBase + config.rerollIncrement * rerollCount;
}

/**
 * Generates card shop inventory with `config.cardShopSize` card slots.
 * Cards are weighted toward the player's starting tides. Prices are
 * randomized in [cardPriceMin, cardPriceMax] rounded to nearest 5,
 * with 1-2 random slots receiving a 30-70% discount.
 */
export function generateCardShopInventory(
  cardDatabase: Map<number, CardData>,
  playerPool: DeckEntry[],
  seedTides: Tide[],
  config: QuestConfig,
): ShopSlot[] {
  const poolTideCounts = countDeckTides(playerPool, cardDatabase);

  // Use starting tides to seed counts if pool is empty
  for (const tide of seedTides) {
    if (!poolTideCounts.has(tide)) {
      poolTideCounts.set(tide, 1);
    }
  }

  const allCards = Array.from(cardDatabase.values()).filter(
    (c) => c.tide !== "Neutral" && c.rarity !== "Starter",
  );

  const selected = weightedSample(
    allCards,
    config.cardShopSize,
    (card) => tideWeight(card.tide, poolTideCounts),
  );

  const slots: ShopSlot[] = selected.map((card) => {
    const rawPrice =
      config.cardPriceMin +
      Math.random() * (config.cardPriceMax - config.cardPriceMin);
    const basePrice = Math.round(rawPrice / 5) * 5;
    return {
      card,
      basePrice,
      discountPercent: 0,
      purchased: false,
    };
  });

  // Apply discounts to 1-2 random slots
  const discountCount = Math.random() < 0.5 ? 1 : 2;
  const indices = slots.map((_, i) => i).sort(() => Math.random() - 0.5);
  for (let d = 0; d < discountCount && d < indices.length; d++) {
    // 30, 40, 50, 60, or 70 percent discount
    const discount = 30 + Math.floor(Math.random() * 5) * 10;
    slots[indices[d]] = { ...slots[indices[d]], discountPercent: discount };
  }

  return slots;
}
