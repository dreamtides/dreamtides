import { describe, it, expect } from "vitest";
import { computeTideDistribution } from "./tide-distribution";
import type { CardData, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";

function makeCard(cardNumber: number, tide: Tide): CardData {
  return {
    name: `Card ${String(cardNumber)}`,
    id: `card-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "Beast",
    rarity: "Common",
    energyCost: 3,
    spark: 2,
    isFast: false,
    tide,
    tideCost: 1,
    renderedText: "",
    imageNumber: 1,
    artOwned: true,
  };
}

function makeDeckEntry(cardNumber: number): DeckEntry {
  return {
    entryId: `entry-${String(cardNumber)}`,
    cardNumber,
    transfiguration: null,
    isBane: false,
  };
}

describe("computeTideDistribution", () => {
  it("returns zero counts for an empty deck", () => {
    const db = new Map<number, CardData>();
    const result = computeTideDistribution([], db);
    expect(result.total).toBe(0);
    for (const entry of result.tides) {
      expect(entry.count).toBe(0);
      expect(entry.percentage).toBe(0);
    }
  });

  it("counts cards by tide correctly", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom"));
    db.set(2, makeCard(2, "Bloom"));
    db.set(3, makeCard(3, "Ignite"));
    db.set(4, makeCard(4, "Rime"));

    const deck: DeckEntry[] = [
      makeDeckEntry(1),
      makeDeckEntry(2),
      makeDeckEntry(3),
      makeDeckEntry(4),
    ];

    const result = computeTideDistribution(deck, db);
    expect(result.total).toBe(4);

    const bloom = result.tides.find((t) => t.tide === "Bloom");
    expect(bloom?.count).toBe(2);
    expect(bloom?.percentage).toBe(50);

    const ignite = result.tides.find((t) => t.tide === "Ignite");
    expect(ignite?.count).toBe(1);
    expect(ignite?.percentage).toBe(25);

    const rime = result.tides.find((t) => t.tide === "Rime");
    expect(rime?.count).toBe(1);
    expect(rime?.percentage).toBe(25);

    const arc = result.tides.find((t) => t.tide === "Arc");
    expect(arc?.count).toBe(0);
    expect(arc?.percentage).toBe(0);
  });

  it("identifies the dominant tide", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Pact"));
    db.set(2, makeCard(2, "Pact"));
    db.set(3, makeCard(3, "Pact"));
    db.set(4, makeCard(4, "Surge"));

    const deck: DeckEntry[] = [
      makeDeckEntry(1),
      makeDeckEntry(2),
      makeDeckEntry(3),
      makeDeckEntry(4),
    ];

    const result = computeTideDistribution(deck, db);
    const pact = result.tides.find((t) => t.tide === "Pact");
    expect(pact?.isDominant).toBe(true);

    const surge = result.tides.find((t) => t.tide === "Surge");
    expect(surge?.isDominant).toBe(false);
  });

  it("marks multiple tides as dominant when they are tied", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Arc"));
    db.set(2, makeCard(2, "Umbra"));

    const deck: DeckEntry[] = [makeDeckEntry(1), makeDeckEntry(2)];

    const result = computeTideDistribution(deck, db);
    const arc = result.tides.find((t) => t.tide === "Arc");
    const umbra = result.tides.find((t) => t.tide === "Umbra");
    expect(arc?.isDominant).toBe(true);
    expect(umbra?.isDominant).toBe(true);
  });

  it("skips cards not found in the database", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom"));

    const deck: DeckEntry[] = [makeDeckEntry(1), makeDeckEntry(999)];

    const result = computeTideDistribution(deck, db);
    expect(result.total).toBe(1);

    const bloom = result.tides.find((t) => t.tide === "Bloom");
    expect(bloom?.count).toBe(1);
    expect(bloom?.percentage).toBe(100);
  });

  it("includes all 8 tides in the output", () => {
    const db = new Map<number, CardData>();
    const result = computeTideDistribution([], db);
    const tideNames = result.tides.map((t) => t.tide);
    expect(tideNames).toContain("Bloom");
    expect(tideNames).toContain("Arc");
    expect(tideNames).toContain("Ignite");
    expect(tideNames).toContain("Pact");
    expect(tideNames).toContain("Umbra");
    expect(tideNames).toContain("Rime");
    expect(tideNames).toContain("Surge");
    expect(tideNames).toContain("Wild");
    expect(result.tides).toHaveLength(8);
  });

  it("rounds percentages to integers", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom"));
    db.set(2, makeCard(2, "Bloom"));
    db.set(3, makeCard(3, "Ignite"));

    const deck: DeckEntry[] = [
      makeDeckEntry(1),
      makeDeckEntry(2),
      makeDeckEntry(3),
    ];

    const result = computeTideDistribution(deck, db);
    const bloom = result.tides.find((t) => t.tide === "Bloom");
    expect(bloom?.percentage).toBe(67);

    const ignite = result.tides.find((t) => t.tide === "Ignite");
    expect(ignite?.percentage).toBe(33);
  });

  it("no tide is dominant when deck is empty", () => {
    const db = new Map<number, CardData>();
    const result = computeTideDistribution([], db);
    for (const entry of result.tides) {
      expect(entry.isDominant).toBe(false);
    }
  });
});
