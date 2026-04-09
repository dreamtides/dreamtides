import { describe, it, expect } from "vitest";
import { extractDraftDebugInfo } from "./debug-helpers";
import type { DraftState } from "../types/draft";
import type { CardData } from "../types/cards";

function makeCard(num: number, tide: string, name: string): CardData {
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
    tide: tide as CardData["tide"],
    tideCost: 1,
    renderedText: "",
    imageNumber: num,
    artOwned: true,
  };
}

function makeDraftState(
  draftedCards: number[] = [],
  pickNumber = 1,
): DraftState {
  return {
    pool: [],
    draftedCards,
    currentPack: [],
    pickNumber,
    sitePicksCompleted: 0,
    packStrategy: { type: "depletion" },
    seenCards: [],
  };
}

describe("extractDraftDebugInfo", () => {
  it("returns null when draft state is null", () => {
    const result = extractDraftDebugInfo(null, new Map());
    expect(result).toBeNull();
  });

  it("returns basic info for empty draft state", () => {
    const state = makeDraftState();
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.totalCards).toBe(0);
    expect(result.pickNumber).toBe(1);
    expect(result.seenCards).toBe(0);
  });

  it("resolves drafted card data", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "Rose Golem"));
    db.set(2, makeCard(2, "Arc", "Lightning Sprite"));

    const state = makeDraftState([2, 1], 3);
    const result = extractDraftDebugInfo(state, db)!;

    expect(result.draftedCards).toHaveLength(2);
    expect(result.draftedCards[0].name).toBe("Lightning Sprite");
    expect(result.draftedCards[1].name).toBe("Rose Golem");
    expect(result.totalCards).toBe(2);
  });

  it("groups drafted cards by tide", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "Rose Golem"));
    db.set(2, makeCard(2, "Bloom", "Vine Crawler"));
    db.set(3, makeCard(3, "Arc", "Spark Mage"));

    const state = makeDraftState([3, 2, 1], 4);
    const result = extractDraftDebugInfo(state, db)!;

    expect(result.cardsByTide.Bloom).toBe(2);
    expect(result.cardsByTide.Arc).toBe(1);
  });

  it("includes pack strategy", () => {
    const state = makeDraftState();
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.packStrategy).toEqual({ type: "depletion" });
  });

  it("passes through chosen tide", () => {
    const state = makeDraftState();
    const result = extractDraftDebugInfo(state, new Map(), "Surge")!;
    expect(result.chosenTide).toBe("Surge");
  });

  it("tracks seen card count", () => {
    const state: DraftState = {
      ...makeDraftState(),
      seenCards: [1, 2, 3],
    };
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.seenCards).toBe(3);
  });
});
