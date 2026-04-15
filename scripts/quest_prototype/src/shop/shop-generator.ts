import type { CardData, Rarity } from "../types/cards";
import type { DreamsignTemplate, PackageTideId } from "../types/content";
import type { DeckEntry, Dreamsign } from "../types/quest";

import { pickPackageAdjacentItem } from "../data/tide-weights";
import { readDreamsignPool } from "../dreamsign/dreamsign-pool";
import { createDreamsign } from "../data/dreamsigns";

/** Prices by rarity for card items. */
const RARITY_PRICES: Readonly<Record<Rarity, number>> = {
  Common: 50,
  Uncommon: 100,
  Rare: 200,
  Legendary: 400,
};

/** Fixed price for dreamsign items. */
const DREAMSIGN_PRICE = 150;

/** Base cost for a shop reroll. */
const REROLL_BASE_COST = 50;

/** Additional cost per previous reroll. */
const REROLL_INCREMENT = 25;

/** Chance (out of 6) for a slot to be a dreamsign. */
const DREAMSIGN_CHANCE = 1 / 6;

/** The types of items that can appear in a shop slot. */
export type ShopItemType = "card" | "dreamsign" | "reroll";

/** A single slot in the shop inventory. */
export interface ShopSlot {
  itemType: ShopItemType;
  card: CardData | null;
  dreamsign: Dreamsign | null;
  basePrice: number;
  discountPercent: number;
  purchased: boolean;
}

export interface ShopGenerationOptions {
  selectedPackageTides?: readonly PackageTideId[];
  remainingDreamsignPoolIds?: readonly string[];
  dreamsignTemplates?: readonly DreamsignTemplate[];
}

export interface ShopInventoryResult {
  slots: ShopSlot[];
  remainingDreamsignPoolIds: string[];
  spentDreamsignPoolIds: string[];
}

/** Returns the effective price of a slot after discount. */
export function effectivePrice(slot: ShopSlot): number {
  if (slot.discountPercent === 0) return slot.basePrice;
  return Math.round(slot.basePrice * (1 - slot.discountPercent / 100));
}

/** Computes reroll cost given the number of previous rerolls. */
export function rerollCost(rerollCount: number, isEnhanced: boolean): number {
  if (isEnhanced) return 0;
  return REROLL_BASE_COST + REROLL_INCREMENT * rerollCount;
}

function selectWeightedCard(
  cards: readonly CardData[],
  selectedPackageTides: readonly PackageTideId[] = [],
): CardData | null {
  return pickPackageAdjacentItem(
    cards,
    (card) => card.tides,
    selectedPackageTides,
  );
}

/**
 * Generates shop inventory with 6 slots. Each slot can be a card,
 * dreamsign, or reroll option.
 */
export function generateShopInventory(
  cardDatabase: Map<number, CardData>,
  _playerDeck: DeckEntry[],
  options: ShopGenerationOptions = {},
): ShopInventoryResult {
  const selectedPackageTides = options.selectedPackageTides ?? [];
  const allCards = Array.from(cardDatabase.values());
  const slots: ShopSlot[] = [];
  let remainingDreamsignPoolIds = [...(options.remainingDreamsignPoolIds ?? [])];
  const spentDreamsignPoolIds: string[] = [];

  // Decide if reroll slot appears (50% chance)
  const hasRerollSlot = Math.random() < 0.5;
  const rerollSlotIndex = hasRerollSlot
    ? Math.floor(Math.random() * 6)
    : -1;

  for (let i = 0; i < 6; i++) {
    if (i === rerollSlotIndex) {
      slots.push({
        itemType: "reroll",
        card: null,
        dreamsign: null,
        basePrice: REROLL_BASE_COST,
        discountPercent: 0,
        purchased: false,
      });
      continue;
    }

    // Roll for dreamsign
    if (
      Math.random() < DREAMSIGN_CHANCE &&
      options.dreamsignTemplates !== undefined &&
      remainingDreamsignPoolIds.length > 0
    ) {
      const dreamsignPoolState = readDreamsignPool(
        remainingDreamsignPoolIds,
        options.dreamsignTemplates,
      );
      const template = pickPackageAdjacentItem(
        dreamsignPoolState.availableIds
          .map((id) => dreamsignPoolState.templatesById.get(id))
          .filter((candidate): candidate is DreamsignTemplate => candidate !== undefined),
        (candidate) => candidate.packageTides,
        selectedPackageTides,
      );
      if (template !== null) {
        remainingDreamsignPoolIds = dreamsignPoolState.availableIds.filter(
          (id) => id !== template.id,
        );
        spentDreamsignPoolIds.push(template.id);
        slots.push({
          itemType: "dreamsign",
          card: null,
          dreamsign: createDreamsign(template),
          basePrice: DREAMSIGN_PRICE,
          discountPercent: 0,
          purchased: false,
        });
        continue;
      }
    }

    // Default: card slot
    const card = selectWeightedCard(
      allCards,
      selectedPackageTides,
    );
    if (card) {
      slots.push({
        itemType: "card",
        card,
        dreamsign: null,
        basePrice: RARITY_PRICES[card.rarity],
        discountPercent: 0,
        purchased: false,
      });
    }
  }

  // Apply discounts to 1-2 random non-reroll slots
  const discountableIndices = slots
    .map((s, i) => (s.itemType !== "reroll" ? i : -1))
    .filter((i) => i >= 0);

  const discountCount = Math.random() < 0.5 ? 1 : 2;
  const shuffled = discountableIndices.sort(() => Math.random() - 0.5);
  for (let d = 0; d < discountCount && d < shuffled.length; d++) {
    const idx = shuffled[d];
    // 30-90% discount in increments of 10
    const discount = 30 + Math.floor(Math.random() * 7) * 10;
    slots[idx] = { ...slots[idx], discountPercent: discount };
  }

  return {
    slots,
    remainingDreamsignPoolIds,
    spentDreamsignPoolIds,
  };
}

/**
 * Generates specialty shop inventory: 4 rare cards weighted
 * toward the player's drafted tides.
 */
export function generateSpecialtyShopInventory(
  cardDatabase: Map<number, CardData>,
  _playerDeck: DeckEntry[],
  selectedPackageTides: readonly PackageTideId[] = [],
): ShopSlot[] {
  const rareCards = Array.from(cardDatabase.values()).filter(
    (card) => card.rarity === "Rare",
  );
  const slots: ShopSlot[] = [];

  for (let i = 0; i < 4; i += 1) {
    const card = selectWeightedCard(rareCards, selectedPackageTides);
    if (card) {
      slots.push({
        itemType: "card",
        card,
        dreamsign: null,
        basePrice: RARITY_PRICES.Rare,
        discountPercent: 0,
        purchased: false,
      });
    }
  }

  return slots;
}
