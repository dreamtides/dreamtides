import { describe, expect, it } from "vitest";
import type { CardData, Rarity, Tide } from "../types/cards";
import {
  draftPoolCards,
  findStarterCards,
  neutralStartingCandidates,
  offerableCards,
  randomStartingTideCandidates,
} from "./card-pools";

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

function buildDatabase(cards: CardData[]): Map<number, CardData> {
  return new Map(cards.map((card) => [card.cardNumber, card]));
}

describe("card-pools", () => {
  const db = buildDatabase([
    makeCard(30, "Bloom"),
    makeCard(10, "Neutral", "Starter"),
    makeCard(20, "Bloom", "Starter"),
    makeCard(40, "Arc"),
    makeCard(15, "Bloom"),
    makeCard(50, "Neutral"),
    makeCard(60, "Neutral", "Legendary"),
    makeCard(70, "Pact"),
    makeCard(80, "Arc"),
  ]);

  it("findStarterCards returns Starter cards sorted by card number", () => {
    expect(findStarterCards(db).map((card) => card.cardNumber)).toEqual([
      10,
      20,
    ]);
  });

  it("offerableCards excludes Starter cards and debug-excluded tides", () => {
    expect(
      offerableCards(db, { excludedTides: ["Arc"] }).map(
        (card) => card.cardNumber,
      ),
    ).toEqual([15, 30, 50, 60, 70]);
  });

  it("randomStartingTideCandidates returns named-tide cards for the selected tide", () => {
    expect(
      randomStartingTideCandidates(db, "Bloom").map((card) => card.cardNumber),
    ).toEqual([15, 30]);
  });

  it("neutralStartingCandidates excludes Neutral Legendary cards", () => {
    expect(neutralStartingCandidates(db).map((card) => card.cardNumber)).toEqual([
      50,
    ]);
  });

  it("draftPoolCards excludes consumed starting grants", () => {
    expect(
      draftPoolCards(db, { consumedCardNumbers: new Set([15, 50, 70]) }).map(
        (card) => card.cardNumber,
      ),
    ).toEqual([30, 40, 60, 80]);
  });
});
