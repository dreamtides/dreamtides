import { describe, it, expect } from "vitest";
import type { CardData } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import {
  generateShopInventory,
  generateSpecialtyShopInventory,
  effectivePrice,
  rerollCost,
} from "./shop-generator";

function makeCard(overrides: Partial<CardData> = {}): CardData {
  return {
    name: "Test Card",
    id: "test-id",
    cardNumber: 1,
    cardType: "Character",
    subtype: "",
    rarity: "Common",
    energyCost: 2,
    spark: 1,
    isFast: false,
    tide: "Bloom",
    tideCost: 1,
    renderedText: "Test text",
    imageNumber: 1,
    artOwned: false,
    ...overrides,
  };
}

function makeDatabase(cards: CardData[]): Map<number, CardData> {
  const db = new Map<number, CardData>();
  for (const card of cards) {
    db.set(card.cardNumber, card);
  }
  return db;
}

function makeDeckEntry(cardNumber: number): DeckEntry {
  return {
    entryId: `entry-${String(cardNumber)}`,
    cardNumber,
    transfiguration: null,
    isBane: false,
  };
}

describe("effectivePrice", () => {
  it("returns base price when no discount", () => {
    const result = effectivePrice({
      itemType: "card",
      card: null,
      dreamsign: null,
      tideCrystal: null,
      basePrice: 100,
      discountPercent: 0,
      purchased: false,
    });
    expect(result).toBe(100);
  });

  it("applies discount percentage correctly", () => {
    const result = effectivePrice({
      itemType: "card",
      card: null,
      dreamsign: null,
      tideCrystal: null,
      basePrice: 200,
      discountPercent: 50,
      purchased: false,
    });
    expect(result).toBe(100);
  });

  it("rounds to nearest integer", () => {
    const result = effectivePrice({
      itemType: "card",
      card: null,
      dreamsign: null,
      tideCrystal: null,
      basePrice: 100,
      discountPercent: 30,
      purchased: false,
    });
    expect(result).toBe(70);
  });
});

describe("rerollCost", () => {
  it("returns base cost for first reroll", () => {
    expect(rerollCost(0, false)).toBe(50);
  });

  it("increments cost by 25 per previous reroll", () => {
    expect(rerollCost(1, false)).toBe(75);
    expect(rerollCost(2, false)).toBe(100);
    expect(rerollCost(3, false)).toBe(125);
  });

  it("returns 0 when enhanced", () => {
    expect(rerollCost(0, true)).toBe(0);
    expect(rerollCost(5, true)).toBe(0);
  });
});

describe("generateShopInventory", () => {
  const cards = [
    makeCard({ cardNumber: 1, rarity: "Common", tide: "Bloom" }),
    makeCard({ cardNumber: 2, rarity: "Uncommon", tide: "Arc" }),
    makeCard({ cardNumber: 3, rarity: "Rare", tide: "Ignite" }),
  ];
  const db = makeDatabase(cards);

  it("generates exactly 6 slots", () => {
    const slots = generateShopInventory(db, []);
    expect(slots).toHaveLength(6);
  });

  it("all slots start as not purchased", () => {
    const slots = generateShopInventory(db, []);
    for (const slot of slots) {
      expect(slot.purchased).toBe(false);
    }
  });

  it("has at most one reroll slot", () => {
    // Run multiple times to account for randomness
    for (let i = 0; i < 20; i++) {
      const slots = generateShopInventory(db, []);
      const rerollSlots = slots.filter((s) => s.itemType === "reroll");
      expect(rerollSlots.length).toBeLessThanOrEqual(1);
    }
  });

  it("applies 1-2 discounts to non-reroll slots", () => {
    for (let i = 0; i < 20; i++) {
      const slots = generateShopInventory(db, []);
      const discounted = slots.filter((s) => s.discountPercent > 0);
      expect(discounted.length).toBeGreaterThanOrEqual(1);
      expect(discounted.length).toBeLessThanOrEqual(2);
      for (const slot of discounted) {
        expect(slot.itemType).not.toBe("reroll");
        expect(slot.discountPercent).toBeGreaterThanOrEqual(30);
        expect(slot.discountPercent).toBeLessThanOrEqual(90);
      }
    }
  });

  it("card slots have correct rarity-based prices", () => {
    for (let i = 0; i < 20; i++) {
      const slots = generateShopInventory(db, []);
      for (const slot of slots) {
        if (slot.itemType === "card" && slot.card) {
          const expected: Record<string, number> = {
            Common: 50,
            Uncommon: 100,
            Rare: 200,
            Legendary: 400,
          };
          expect(slot.basePrice).toBe(expected[slot.card.rarity]);
        }
      }
    }
  });

  it("dreamsign slots have price 150", () => {
    for (let i = 0; i < 50; i++) {
      const slots = generateShopInventory(db, []);
      for (const slot of slots) {
        if (slot.itemType === "dreamsign") {
          expect(slot.basePrice).toBe(150);
          expect(slot.dreamsign).not.toBeNull();
        }
      }
    }
  });

  it("tide crystal slots have price 200", () => {
    for (let i = 0; i < 50; i++) {
      const slots = generateShopInventory(db, []);
      for (const slot of slots) {
        if (slot.itemType === "tideCrystal") {
          expect(slot.basePrice).toBe(200);
          expect(slot.tideCrystal).not.toBeNull();
        }
      }
    }
  });
});

describe("generateSpecialtyShopInventory", () => {
  const cards = [
    makeCard({ cardNumber: 1, rarity: "Common", tide: "Bloom" }),
    makeCard({ cardNumber: 2, rarity: "Rare", tide: "Arc" }),
    makeCard({ cardNumber: 3, rarity: "Rare", tide: "Ignite" }),
    makeCard({ cardNumber: 4, rarity: "Rare", tide: "Rime" }),
  ];
  const db = makeDatabase(cards);

  it("generates exactly 4 slots", () => {
    const slots = generateSpecialtyShopInventory(db, []);
    expect(slots).toHaveLength(4);
  });

  it("all slots are rare cards", () => {
    const slots = generateSpecialtyShopInventory(db, []);
    for (const slot of slots) {
      expect(slot.itemType).toBe("card");
      expect(slot.card).not.toBeNull();
      expect(slot.card!.rarity).toBe("Rare");
    }
  });

  it("all slots are priced at 200", () => {
    const slots = generateSpecialtyShopInventory(db, []);
    for (const slot of slots) {
      expect(slot.basePrice).toBe(200);
    }
  });

  it("all slots start as not purchased", () => {
    const slots = generateSpecialtyShopInventory(db, []);
    for (const slot of slots) {
      expect(slot.purchased).toBe(false);
    }
  });

  it("weights toward deck tides", () => {
    const deckEntries = Array.from({ length: 20 }, () => makeDeckEntry(2));
    // Run multiple trials: cards matching Arc tide (card 2) should appear more
    const tideCounts: Record<string, number> = { Arc: 0, Ignite: 0, Rime: 0 };
    for (let i = 0; i < 100; i++) {
      const slots = generateSpecialtyShopInventory(db, deckEntries);
      for (const slot of slots) {
        if (slot.card) {
          tideCounts[slot.card.tide] = (tideCounts[slot.card.tide] ?? 0) + 1;
        }
      }
    }
    // Arc should appear significantly more than average
    expect(tideCounts.Arc).toBeGreaterThan(tideCounts.Ignite);
  });
});
