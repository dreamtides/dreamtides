import { describe, it, expect } from "vitest";
import { extractDraftDebugInfo } from "./debug-helpers";
import type { DraftState } from "../types/draft";
import type { CardData } from "../types/cards";

function makeCard(num: number, name: string): CardData {
  return {
    name,
    id: `card-${String(num)}`,
    cardNumber: num,
    cardType: "Character",
    subtype: "",
    rarity: "Common",
    energyCost: 1,
    spark: 1,
    isFast: false,
    tides: ["package"],
    renderedText: "",
    imageNumber: num,
    artOwned: true,
  };
}

function makeDraftState(
  overrides: Partial<DraftState> = {},
): DraftState {
  return {
    remainingCopiesByCard: {},
    currentOffer: [],
    pickNumber: 1,
    sitePicksCompleted: 0,
    ...overrides,
  };
}

describe("extractDraftDebugInfo", () => {
  it("returns null when draft state is null", () => {
    expect(extractDraftDebugInfo(null, new Map())).toBeNull();
  });

  it("returns basic pool info", () => {
    const result = extractDraftDebugInfo(
      makeDraftState({
        remainingCopiesByCard: { "1": 2, "2": 1 },
      }),
      new Map(),
    );

    expect(result).not.toBeNull();
    expect(result?.pickNumber).toBe(1);
    expect(result?.sitePicksCompleted).toBe(0);
    expect(result?.remainingCards).toBe(3);
    expect(result?.remainingUniqueCards).toBe(2);
  });

  it("resolves current offer card data", () => {
    const cardDatabase = new Map<number, CardData>([
      [1, makeCard(1, "Rose Golem")],
      [2, makeCard(2, "Lightning Sprite")],
    ]);

    const result = extractDraftDebugInfo(
      makeDraftState({
        currentOffer: [2, 1],
      }),
      cardDatabase,
    );

    expect(result?.currentOfferSize).toBe(2);
    expect(result?.currentOffer.map((card) => card.name)).toEqual([
      "Lightning Sprite",
      "Rose Golem",
    ]);
  });
});
