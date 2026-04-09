import type { CardData, NamedTide, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { countDeckTides, tideWeight, weightedSample } from "../data/tide-weights";
import { adjacentTides } from "../data/card-database";

/** A single slot in the card shop inventory. */
export interface ShopSlot {
  card: CardData;
  basePrice: number;
  discountPercent: number;
  purchased: boolean;
}

/** A tide crystal available for purchase in the shop. */
export interface TideCrystalSlot {
  tide: NamedTide;
  price: number;
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
 * Cards are filtered to only include tides the player can play (has
 * crystals for), then weighted toward the player's dominant tides.
 * Prices are randomized in [cardPriceMin, cardPriceMax] rounded to
 * nearest 5, with 1-2 random slots receiving a 30-70% discount.
 */
export function generateCardShopInventory(
  cardDatabase: Map<number, CardData>,
  playerPool: DeckEntry[],
  seedTides: Tide[],
  config: QuestConfig,
  playableTides?: Set<Tide>,
): ShopSlot[] {
  const poolTideCounts = countDeckTides(playerPool, cardDatabase);

  // Use starting tides to seed counts if pool is empty
  for (const tide of seedTides) {
    if (!poolTideCounts.has(tide)) {
      poolTideCounts.set(tide, 1);
    }
  }

  const allCards = Array.from(cardDatabase.values()).filter((c) => {
    if (c.rarity === "Starter") return false;
    if (c.tide === "Neutral") return false;
    if (playableTides !== undefined && !playableTides.has(c.tide)) return false;
    return true;
  });

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

/** Price for a tide crystal in the shop. */
const TIDE_CRYSTAL_PRICE = 150;

/**
 * Generates tide crystal slots for the shop. Offers crystals for tides
 * adjacent to the player's current crystals that they don't already own.
 */
export function generateTideCrystalSlots(
  tideCrystals: Record<Tide, number>,
  startingTide: NamedTide | null,
): TideCrystalSlot[] {
  if (startingTide === null) return [];

  // Find tides the player has crystals for
  const ownedTides = new Set<NamedTide>();
  for (const [tide, count] of Object.entries(tideCrystals)) {
    if (count > 0 && tide !== "Neutral") {
      ownedTides.add(tide as NamedTide);
    }
  }

  // Offer crystals for adjacent tides the player doesn't own
  const offered = new Set<NamedTide>();
  for (const owned of ownedTides) {
    for (const adj of adjacentTides(owned) as NamedTide[]) {
      if (!ownedTides.has(adj)) {
        offered.add(adj);
      }
    }
  }

  return Array.from(offered).map((tide) => ({
    tide,
    price: TIDE_CRYSTAL_PRICE,
    purchased: false,
  }));
}
