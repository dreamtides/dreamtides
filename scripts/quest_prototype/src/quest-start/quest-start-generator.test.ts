import { describe, expect, it, vi } from "vitest";
import type { CardData, Rarity, Tide } from "../types/cards";
import {
  buildStartingDeckPlan,
  selectStartingTideOptions,
} from "./quest-start-generator";

function makeCard(
  cardNumber: number,
  tide: Tide,
  rarity: Rarity = "Common",
): CardData {
  return {
    name: `Card ${String(cardNumber)}`,
    id: `card-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity,
    energyCost: 1,
    spark: 1,
    isFast: false,
    tide,
    tideCost: tide === "Neutral" ? 0 : 1,
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

function buildDatabase(): Map<number, CardData> {
  const cards: CardData[] = [
    makeCard(10, "Neutral", "Starter"),
    makeCard(11, "Neutral", "Starter"),
    makeCard(12, "Neutral", "Starter"),
    makeCard(13, "Neutral", "Starter"),
    makeCard(14, "Neutral", "Starter"),
    makeCard(15, "Neutral", "Starter"),
    makeCard(16, "Neutral", "Starter"),
    makeCard(17, "Neutral", "Starter"),
    makeCard(18, "Neutral", "Starter"),
    makeCard(19, "Neutral", "Starter"),
    makeCard(21, "Bloom", "Common"),
    makeCard(22, "Bloom", "Common"),
    makeCard(23, "Bloom", "Common"),
    makeCard(24, "Bloom", "Common"),
    makeCard(25, "Bloom", "Legendary"),
    makeCard(26, "Bloom", "Common"),
    makeCard(27, "Bloom", "Common"),
    makeCard(28, "Bloom", "Common"),
    makeCard(29, "Bloom", "Common"),
    makeCard(30, "Bloom", "Common"),
    makeCard(31, "Neutral", "Common"),
    makeCard(32, "Neutral", "Common"),
    makeCard(33, "Neutral", "Common"),
    makeCard(34, "Neutral", "Common"),
    makeCard(35, "Neutral", "Legendary"),
    makeCard(36, "Neutral", "Common"),
    makeCard(37, "Neutral", "Common"),
    makeCard(38, "Neutral", "Common"),
    makeCard(39, "Neutral", "Common"),
    makeCard(40, "Neutral", "Common"),
    makeCard(44, "Neutral", "Common"),
    makeCard(41, "Arc", "Common"),
    makeCard(42, "Pact", "Common"),
    makeCard(43, "Surge", "Common"),
  ];

  return new Map(cards.map((card) => [card.cardNumber, card]));
}

describe("selectStartingTideOptions", () => {
  it("returns 3 distinct named tides", () => {
    const options = selectStartingTideOptions([]);
    expect(options).toHaveLength(3);
    expect(new Set(options).size).toBe(3);
    expect(options).not.toContain("Neutral");
  });

  it("excludes named tides passed in the excluded list", () => {
    const options = selectStartingTideOptions([
      "Bloom",
      "Arc",
      "Ignite",
      "Pact",
    ]);
    expect(options).toHaveLength(3);
    expect(options).not.toContain("Bloom");
    expect(options).not.toContain("Arc");
    expect(options).not.toContain("Ignite");
    expect(options).not.toContain("Pact");
  });
});

describe("buildStartingDeckPlan", () => {
  it("builds a 30-card starting loadout from starter, tide, and neutral groups", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const db = buildDatabase();
    const plan = buildStartingDeckPlan(db, "Bloom");

    expect(plan.starterCardNumbers).toEqual([
      10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    ]);
    expect(plan.tideCardNumbers).toHaveLength(10);
    expect(plan.neutralCardNumbers).toHaveLength(10);
    expect(plan.deckCardNumbers).toHaveLength(30);
    expect(plan.consumedRandomCardNumbers).toHaveLength(20);
    expect(plan.neutralCardNumbers).not.toContain(35);
    expect(plan.tideCardNumbers).toContain(25);
  });

  it("uses the selected tide for the random tide package", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const db = buildDatabase();
    const plan = buildStartingDeckPlan(db, "Bloom");
    for (const cardNumber of plan.tideCardNumbers) {
      expect(db.get(cardNumber)?.tide).toBe("Bloom");
    }
  });
});
