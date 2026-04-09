import { describe, expect, it, vi } from "vitest";
import type { CardData, NamedTide, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import {
  computeQuestTideProfile,
  tideProfileWeight,
  weightedSampleByProfile,
} from "./quest-tide-profile";

function makeCard(
  cardNumber: number,
  tide: Tide,
  rarity: CardData["rarity"] = "Common",
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

function makeDeckEntry(cardNumber: number): DeckEntry {
  return {
    entryId: `entry-${String(cardNumber)}`,
    cardNumber,
    transfiguration: null,
    isBane: false,
  };
}

function makeDreamcaller(tides: readonly [NamedTide, NamedTide]) {
  return { tides };
}

describe("computeQuestTideProfile", () => {
  const cardDatabase = new Map<number, CardData>([
    [1, makeCard(1, "Neutral", "Starter")],
    [2, makeCard(2, "Neutral")],
    [3, makeCard(3, "Pact")],
    [4, makeCard(4, "Pact")],
    [5, makeCard(5, "Pact")],
    [6, makeCard(6, "Pact")],
    [7, makeCard(7, "Bloom")],
    [8, makeCard(8, "Bloom")],
    [9, makeCard(9, "Bloom")],
    [10, makeCard(10, "Bloom")],
  ]);

  it("gives the starting tide the strongest early weight and neighbors secondary weight", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [],
      cardDatabase,
      dreamcaller: null,
      tideCrystals: {
        Bloom: 1,
        Arc: 0,
        Ignite: 0,
        Pact: 0,
        Umbra: 0,
        Rime: 0,
        Surge: 0,
        Neutral: 0,
      },
      recentDraftPicks: [],
    });

    expect(profile.weights.Bloom).toBeGreaterThan(profile.weights.Arc);
    expect(profile.weights.Arc).toBeGreaterThan(profile.weights.Pact);
    expect(profile.weights.Surge).toBeGreaterThan(profile.weights.Pact);
  });

  it("ignores Starter entries and heavily discounts Neutral deck cards", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [makeDeckEntry(1), makeDeckEntry(2), makeDeckEntry(7)],
      cardDatabase,
      dreamcaller: null,
      tideCrystals: {
        Bloom: 1,
        Arc: 0,
        Ignite: 0,
        Pact: 0,
        Umbra: 0,
        Rime: 0,
        Surge: 0,
        Neutral: 0,
      },
      recentDraftPicks: [],
    });

    expect(profile.weights.Bloom).toBeGreaterThan(profile.weights.Neutral);
    expect(profile.weights.Neutral).toBeGreaterThan(0);
  });

  it("can pivot toward a deck with many cards in another tide", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [
        makeDeckEntry(3),
        makeDeckEntry(4),
        makeDeckEntry(5),
        makeDeckEntry(6),
      ],
      cardDatabase,
      dreamcaller: null,
      tideCrystals: {
        Bloom: 1,
        Arc: 0,
        Ignite: 0,
        Pact: 0,
        Umbra: 0,
        Rime: 0,
        Surge: 0,
        Neutral: 0,
      },
      recentDraftPicks: [],
    });

    expect(profile.weights.Pact).toBeGreaterThan(profile.weights.Bloom);
  });

  it("adds weight for both dreamcaller tides", () => {
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [],
      cardDatabase,
      dreamcaller: makeDreamcaller(["Umbra", "Rime"]),
      tideCrystals: {
        Bloom: 1,
        Arc: 0,
        Ignite: 0,
        Pact: 0,
        Umbra: 0,
        Rime: 0,
        Surge: 0,
        Neutral: 0,
      },
      recentDraftPicks: [],
    });

    expect(profile.contributions.dreamcaller.Umbra).toBeGreaterThan(0);
    expect(profile.contributions.dreamcaller.Rime).toBeGreaterThan(0);
  });
});

describe("weightedSampleByProfile", () => {
  it("samples without replacement and does not mutate the input pool", () => {
    const cards = [
      makeCard(20, "Bloom"),
      makeCard(21, "Bloom"),
      makeCard(22, "Pact"),
    ];
    const profile = computeQuestTideProfile({
      startingTide: "Bloom",
      deck: [],
      cardDatabase: new Map(cards.map((card) => [card.cardNumber, card])),
      dreamcaller: null,
      tideCrystals: {
        Bloom: 1,
        Arc: 0,
        Ignite: 0,
        Pact: 0,
        Umbra: 0,
        Rime: 0,
        Surge: 0,
        Neutral: 0,
      },
      recentDraftPicks: [],
    });

    const randomSpy = vi.spyOn(Math, "random").mockReturnValue(0);
    const sample = weightedSampleByProfile(cards, profile, 2);
    randomSpy.mockRestore();

    expect(sample).toHaveLength(2);
    expect(new Set(sample.map((card) => card.cardNumber)).size).toBe(2);
    expect(cards.map((card) => card.cardNumber)).toEqual([20, 21, 22]);
    expect(tideProfileWeight(profile, "Bloom")).toBeGreaterThan(
      tideProfileWeight(profile, "Pact"),
    );
  });
});
