import { describe, it, expect } from "vitest";
import { extractDraftDebugInfo, expectedDominantAtPick } from "./debug-helpers";
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
    packStrategy: { type: "tide_current" },
    consumedStartingCardNumbers: [],
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
    expect(result.primaryTide).toBeNull();
    expect(result.secondaryTide).toBeNull();
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

  it("computes tide affinities", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "Rose Golem"));

    const state = makeDraftState([1], 2);
    const result = extractDraftDebugInfo(state, db)!;

    // Bloom should have highest affinity
    expect(result.tideAffinities.Bloom).toBeGreaterThan(
      result.tideAffinities.Pact,
    );
  });

  it("identifies primary and secondary tides", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "B1"));
    db.set(2, makeCard(2, "Bloom", "B2"));
    db.set(3, makeCard(3, "Bloom", "B3"));
    db.set(4, makeCard(4, "Arc", "A1"));

    const state = makeDraftState([4, 3, 2, 1], 5);
    const result = extractDraftDebugInfo(state, db)!;

    expect(result.primaryTide).toBe("Bloom");
  });

  it("computes focus value", () => {
    const state = makeDraftState([], 10);
    const result = extractDraftDebugInfo(state, new Map())!;
    // focus = max(0, (10 - 3) * 0.35) = 2.45
    expect(result.focus).toBeCloseTo(2.45, 1);
  });

  it("includes pack strategy", () => {
    const state = makeDraftState();
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.packStrategy).toEqual({ type: "tide_current" });
  });

  it("includes pool_bias strategy with featured tides", () => {
    const state: DraftState = {
      ...makeDraftState(),
      packStrategy: { type: "pool_bias", featuredTides: ["Bloom", "Arc"], featuredWeight: 2.0 },
    };
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.packStrategy.type).toBe("pool_bias");
    if (result.packStrategy.type === "pool_bias") {
      expect(result.packStrategy.featuredTides).toEqual(["Bloom", "Arc"]);
    }
  });

  it("passes through excluded tides", () => {
    const state = makeDraftState();
    const result = extractDraftDebugInfo(state, new Map(), ["Surge", "Umbra"])!;
    expect(result.excludedTides).toEqual(["Surge", "Umbra"]);
  });

  it("computes effective weights for tide_current (equal to affinity^focus)", () => {
    const state = makeDraftState([], 1);
    const result = extractDraftDebugInfo(state, new Map())!;
    // At pick 1, focus = 0, so all weights should be 1.0 (1.0^0 = 1)
    for (const weight of Object.values(result.effectiveWeights)) {
      expect(weight).toBeCloseTo(1.0, 2);
    }
  });

  it("applies featured multiplier to effective weights for pool_bias", () => {
    const state: DraftState = {
      ...makeDraftState([], 1),
      packStrategy: { type: "pool_bias", featuredTides: ["Bloom", "Arc"], featuredWeight: 2.0 },
    };
    const result = extractDraftDebugInfo(state, new Map())!;
    // At pick 1, focus = 0, weights = 1.0 * multiplier
    expect(result.effectiveWeights.Bloom).toBeCloseTo(2.0, 2);
    expect(result.effectiveWeights.Arc).toBeCloseTo(2.0, 2);
    expect(result.effectiveWeights.Ignite).toBeCloseTo(1.0, 2);
  });

  it("tide probabilities sum to approximately 100%", () => {
    const state = makeDraftState();
    const result = extractDraftDebugInfo(state, new Map())!;
    const total = Object.values(result.tideProbabilities).reduce((s, p) => s + p, 0);
    expect(total).toBeCloseTo(100, 0);
  });

  it("computes decay profile for drafted cards", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "Rose Golem"));
    db.set(2, makeCard(2, "Arc", "Spark Mage"));

    const state = makeDraftState([2, 1], 3);
    const result = extractDraftDebugInfo(state, db)!;

    expect(result.decayProfile).toHaveLength(2);
    expect(result.decayProfile[0].cardName).toBe("Spark Mage");
    expect(result.decayProfile[0].decay).toBeCloseTo(1.0, 2);
    expect(result.decayProfile[1].cardName).toBe("Rose Golem");
    expect(result.decayProfile[1].decay).toBeCloseTo(0.85, 2);
  });
});

describe("expectedDominantAtPick", () => {
  it("returns correct values at table boundaries", () => {
    expect(expectedDominantAtPick(1)).toBeCloseTo(0.67, 2);
    expect(expectedDominantAtPick(5)).toBeCloseTo(1.0, 2);
    expect(expectedDominantAtPick(10)).toBeCloseTo(2.15, 2);
    expect(expectedDominantAtPick(15)).toBeCloseTo(3.14, 2);
    expect(expectedDominantAtPick(25)).toBeCloseTo(3.54, 2);
  });

  it("interpolates between table points", () => {
    const val = expectedDominantAtPick(7);
    // Between pick 5 (1.0) and pick 10 (2.15): 1.0 + (2/5) * 1.15 = 1.46
    expect(val).toBeCloseTo(1.46, 1);
  });

  it("clamps below minimum pick", () => {
    expect(expectedDominantAtPick(0)).toBeCloseTo(0.67, 2);
  });

  it("clamps above maximum pick", () => {
    expect(expectedDominantAtPick(30)).toBeCloseTo(3.54, 2);
  });
});
