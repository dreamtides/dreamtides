import type { CardData, Tide, Rarity } from "../types/cards";
import type { DeckEntry, Dreamsign } from "../types/quest";
import { NAMED_TIDES } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";

/** Prices by rarity for card items. */
const RARITY_PRICES: Readonly<Record<Rarity, number>> = {
  Common: 50,
  Uncommon: 100,
  Rare: 200,
  Legendary: 400,
};

/** Fixed price for dreamsign items. */
const DREAMSIGN_PRICE = 150;

/** Fixed price for tide crystal items. */
const TIDE_CRYSTAL_PRICE = 200;

/** Base cost for a shop reroll. */
const REROLL_BASE_COST = 50;

/** Additional cost per previous reroll. */
const REROLL_INCREMENT = 25;

/** Chance (out of 6) for a slot to be a dreamsign. */
const DREAMSIGN_CHANCE = 1 / 6;

/** Chance (out of 6) for a non-dreamsign slot to be a tide crystal. */
const TIDE_CRYSTAL_CHANCE = 1 / 6;

/** The types of items that can appear in a shop slot. */
export type ShopItemType = "card" | "dreamsign" | "tideCrystal" | "reroll";

/** A single slot in the shop inventory. */
export interface ShopSlot {
  itemType: ShopItemType;
  card: CardData | null;
  dreamsign: Dreamsign | null;
  tideCrystal: Tide | null;
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
export function rerollCost(rerollCount: number, isEnhanced: boolean): number {
  if (isEnhanced) return 0;
  return REROLL_BASE_COST + REROLL_INCREMENT * rerollCount;
}

/**
 * Selects a card weighted toward the player's drafted tides.
 * Cards matching drafted tides get proportional weight; undrafted tides
 * get a baseline weight.
 */
function selectWeightedCard(
  cards: CardData[],
  deckTideCounts: Record<Tide, number>,
): CardData | null {
  if (cards.length === 0) return null;

  const totalDeckCards = Object.values(deckTideCounts).reduce(
    (sum, c) => sum + c,
    0,
  );
  const baseline = 1;

  const weights = cards.map((card) => {
    if (totalDeckCards === 0) return 1;
    const tideCount = deckTideCounts[card.tide] ?? 0;
    return baseline + (tideCount / totalDeckCards) * 10;
  });

  const totalWeight = weights.reduce((sum, w) => sum + w, 0);
  if (totalWeight <= 0) return cards[0];

  let roll = Math.random() * totalWeight;
  for (let i = 0; i < cards.length; i++) {
    roll -= weights[i];
    if (roll <= 0) return cards[i];
  }
  return cards[cards.length - 1];
}

/** Counts tide occurrences in the player's deck. */
function countDeckTides(
  deck: DeckEntry[],
  cardDatabase: Map<number, CardData>,
): Record<Tide, number> {
  const counts: Record<Tide, number> = {
    Bloom: 0,
    Arc: 0,
    Ignite: 0,
    Pact: 0,
    Umbra: 0,
    Rime: 0,
    Surge: 0,
    Wild: 0,
  };
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card) {
      counts[card.tide] += 1;
    }
  }
  return counts;
}

/** Selects a random dreamsign from the synthetic pool. */
function selectRandomDreamsign(): Dreamsign {
  const template = DREAMSIGNS[Math.floor(Math.random() * DREAMSIGNS.length)];
  return { ...template, isBane: false };
}

/** Selects a random named tide for a tide crystal. */
function selectRandomTide(): Tide {
  return NAMED_TIDES[Math.floor(Math.random() * NAMED_TIDES.length)];
}

/**
 * Generates shop inventory with 6 slots. Each slot can be a card,
 * dreamsign, tide crystal, or reroll option.
 */
export function generateShopInventory(
  cardDatabase: Map<number, CardData>,
  playerDeck: DeckEntry[],
): ShopSlot[] {
  const allCards = Array.from(cardDatabase.values());
  const deckTideCounts = countDeckTides(playerDeck, cardDatabase);
  const slots: ShopSlot[] = [];

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
        tideCrystal: null,
        basePrice: REROLL_BASE_COST,
        discountPercent: 0,
        purchased: false,
      });
      continue;
    }

    // Roll for dreamsign
    if (Math.random() < DREAMSIGN_CHANCE) {
      slots.push({
        itemType: "dreamsign",
        card: null,
        dreamsign: selectRandomDreamsign(),
        tideCrystal: null,
        basePrice: DREAMSIGN_PRICE,
        discountPercent: 0,
        purchased: false,
      });
      continue;
    }

    // Roll for tide crystal
    if (Math.random() < TIDE_CRYSTAL_CHANCE) {
      slots.push({
        itemType: "tideCrystal",
        card: null,
        dreamsign: null,
        tideCrystal: selectRandomTide(),
        basePrice: TIDE_CRYSTAL_PRICE,
        discountPercent: 0,
        purchased: false,
      });
      continue;
    }

    // Default: card slot
    const card = selectWeightedCard(allCards, deckTideCounts);
    if (card) {
      slots.push({
        itemType: "card",
        card,
        dreamsign: null,
        tideCrystal: null,
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

  return slots;
}

/**
 * Generates specialty shop inventory: 4 rare cards weighted
 * toward the player's drafted tides.
 */
export function generateSpecialtyShopInventory(
  cardDatabase: Map<number, CardData>,
  playerDeck: DeckEntry[],
): ShopSlot[] {
  const rareCards = Array.from(cardDatabase.values()).filter(
    (c) => c.rarity === "Rare",
  );
  const deckTideCounts = countDeckTides(playerDeck, cardDatabase);
  const slots: ShopSlot[] = [];

  for (let i = 0; i < 4; i++) {
    const card = selectWeightedCard(rareCards, deckTideCounts);
    if (card) {
      slots.push({
        itemType: "card",
        card,
        dreamsign: null,
        tideCrystal: null,
        basePrice: RARITY_PRICES.Rare,
        discountPercent: 0,
        purchased: false,
      });
    }
  }

  return slots;
}
