import { describe, expect, it } from "vitest";
import { computeDeckSummary } from "./deck-summary";
import type { CardData } from "../types/cards";
import type { DeckEntry } from "../types/quest";

function makeCard(
  cardNumber: number,
  overrides: Partial<CardData> = {},
): CardData {
  return {
    name: `Card ${String(cardNumber)}`,
    id: `card-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity: "Common",
    energyCost: 2,
    spark: 1,
    isFast: false,
    tides: ["package"],
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: true,
    ...overrides,
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

describe("computeDeckSummary", () => {
  it("returns zeroed metrics for an empty deck", () => {
    const summary = computeDeckSummary([], new Map());

    expect(summary.total).toBe(0);
    expect(summary.characterCount).toBe(0);
    expect(summary.eventCount).toBe(0);
    expect(summary.averageEnergyCost).toBeNull();
    expect(summary.rarities.map((rarity) => rarity.count)).toEqual([0, 0, 0, 0, 0]);
  });

  it("summarizes types, energy cost, and rarity counts", () => {
    const database = new Map<number, CardData>([
      [1, makeCard(1, { rarity: "Starter", energyCost: 1 })],
      [2, makeCard(2, { rarity: "Rare", energyCost: 3, cardType: "Event" })],
      [3, makeCard(3, { rarity: "Rare", energyCost: null })],
      [4, makeCard(4, { rarity: "Legendary", energyCost: 4 })],
    ]);

    const summary = computeDeckSummary(
      [1, 2, 3, 4].map(makeDeckEntry),
      database,
    );

    expect(summary.total).toBe(4);
    expect(summary.characterCount).toBe(3);
    expect(summary.eventCount).toBe(1);
    expect(summary.averageEnergyCost).toBe(2.7);
    expect(summary.rarities).toEqual([
      { rarity: "Starter", count: 1, percentage: 25 },
      { rarity: "Common", count: 0, percentage: 0 },
      { rarity: "Uncommon", count: 0, percentage: 0 },
      { rarity: "Rare", count: 2, percentage: 50 },
      { rarity: "Legendary", count: 1, percentage: 25 },
    ]);
  });
});
