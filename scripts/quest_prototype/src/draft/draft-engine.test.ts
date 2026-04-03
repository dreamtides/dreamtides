import { describe, it, expect, beforeEach } from "vitest";
import { resetLog, getLogEntries } from "../logging";
import type { CardData, Tide } from "../types/cards";
import type { PackStrategy } from "../types/draft";
import {
  initializeDraftState,
  enterDraftSite,
  getPlayerPack,
  processPlayerPick,
  completeDraftSite,
  sortCardsByTide,
  computeTideAffinity,
  computeFocus,
  generatePack,
  selectFeaturedTides,
  SITE_PICKS,
} from "./draft-engine";

/** Helper to build a minimal CardData for testing. */
function makeCard(
  cardNumber: number,
  tide: Tide = "Bloom",
  rarity: "Common" | "Uncommon" | "Rare" | "Legendary" = "Common",
): CardData {
  return {
    name: `TestCard${String(cardNumber)}`,
    id: `test-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity,
    energyCost: 3,
    spark: 1,
    isFast: false,
    tide,
    tideCost: 1,
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

/** Build a card database from an array of cards. */
function buildDB(cards: CardData[]): Map<number, CardData> {
  return new Map(cards.map((c) => [c.cardNumber, c]));
}

/** Create a database with evenly distributed tides. */
function buildEvenDB(cardsPerTide: number): Map<number, CardData> {
  const tides: Tide[] = [
    "Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge", "Neutral",
  ];
  const cards: CardData[] = [];
  let num = 1;
  for (const tide of tides) {
    for (let i = 0; i < cardsPerTide; i++) {
      cards.push(makeCard(num, tide));
      num++;
    }
  }
  return buildDB(cards);
}

beforeEach(() => {
  resetLog();
});

describe("initializeDraftState", () => {
  it("creates state with all cards when no tides excluded", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, []);
    expect(state.pool.length).toBe(80); // 8 tides * 10 cards
    expect(state.draftedCards).toEqual([]);
    expect(state.pickNumber).toBe(1);
    expect(state.sitePicksCompleted).toBe(0);
    expect(state.packStrategy.type).toBe("tide_current");
  });

  it("excludes specified tides from pool", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, ["Bloom", "Arc"]);
    expect(state.pool.length).toBe(60); // 6 tides * 10 cards

    // Verify no Bloom or Arc cards in pool
    for (const cardNum of state.pool) {
      const card = db.get(cardNum)!;
      expect(card.tide).not.toBe("Bloom");
      expect(card.tide).not.toBe("Arc");
    }
  });

  it("never excludes Neutral even if specified tides are excluded", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, ["Bloom", "Arc"]);
    const neutralCards = state.pool.filter(
      (num) => db.get(num)!.tide === "Neutral",
    );
    expect(neutralCards.length).toBe(10);
  });

  it("logs pool initialization", () => {
    const db = buildEvenDB(5);
    initializeDraftState(db, []);
    const entries = getLogEntries();
    const initEvent = entries.find(
      (e) => e.event === "draft_pool_initialized",
    );
    expect(initEvent).toBeDefined();
    expect((initEvent as Record<string, unknown>).poolSize).toBe(40);
  });

  it("uses tide_current strategy by default", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, []);
    expect(state.packStrategy).toEqual({ type: "tide_current" });
  });

  it("uses pool_bias strategy when poolBias is true", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, [], true);
    expect(state.packStrategy.type).toBe("pool_bias");
    if (state.packStrategy.type === "pool_bias") {
      expect(state.packStrategy.featuredTides).toHaveLength(2);
      expect(state.packStrategy.featuredWeight).toBe(2.0);
    }
  });
});

describe("computeTideAffinity", () => {
  it("returns base affinity when no cards drafted", () => {
    const db = buildEvenDB(5);
    const affinity = computeTideAffinity([], db);
    for (const tide of ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"]) {
      expect(affinity.get(tide)).toBe(1.0);
    }
  });

  it("increases same-tide affinity when a card is drafted", () => {
    const db = buildDB([makeCard(1, "Bloom")]);
    // draftedCards is newest first
    const affinity = computeTideAffinity([1], db);
    expect(affinity.get("Bloom")!).toBeGreaterThan(1.0);
    // Same tide gets full 1.0 similarity
    expect(affinity.get("Bloom")!).toBeCloseTo(2.0, 1);
  });

  it("increases allied-tide affinity at lower rate", () => {
    const db = buildDB([makeCard(1, "Bloom")]);
    const affinity = computeTideAffinity([1], db);
    // Arc is distance-1 from Bloom (ally)
    expect(affinity.get("Arc")!).toBeCloseTo(1.5, 1);
    // Pact is distance-3 from Bloom
    expect(affinity.get("Pact")!).toBeCloseTo(1.05, 1);
  });

  it("applies recency decay", () => {
    const db = buildDB([
      makeCard(1, "Bloom"),
      makeCard(2, "Bloom"),
    ]);
    // [2, 1] = card 2 is newest (position 0), card 1 is older (position 1)
    const affinity = computeTideAffinity([2, 1], db);
    // Expected: base(1.0) + 1.0*0.85^0 + 1.0*0.85^1 = 1.0 + 1.0 + 0.85 = 2.85
    expect(affinity.get("Bloom")!).toBeCloseTo(2.85, 1);
  });

  it("handles neutral cards contributing to all core tides", () => {
    const db = buildDB([makeCard(1, "Neutral")]);
    const affinity = computeTideAffinity([1], db);
    // Each core tide gets base(1.0) + 0.4*decay^0 = 1.4
    for (const tide of ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"]) {
      expect(affinity.get(tide)!).toBeCloseTo(1.4, 1);
    }
  });
});

describe("computeFocus", () => {
  it("returns 0 for early picks", () => {
    expect(computeFocus(1)).toBe(0);
    expect(computeFocus(2)).toBe(0);
  });

  it("returns positive value for later picks", () => {
    // focus = max(0, (pick - 3) * 0.35)
    expect(computeFocus(3)).toBeCloseTo(0, 5);
    expect(computeFocus(5)).toBeCloseTo(0.7, 1);
    expect(computeFocus(10)).toBeCloseTo(2.45, 1);
  });
});

describe("generatePack", () => {
  it("draws exactly packSize cards with tide_current strategy", () => {
    const db = buildEvenDB(10);
    const pool = Array.from(db.keys());
    const pack = generatePack(
      { type: "tide_current" },
      { pool, cardDatabase: db, draftedCards: [], pickNumber: 1, packSize: 4 },
    );
    expect(pack.length).toBe(4);
  });

  it("draws fewer cards if pool is smaller than packSize", () => {
    const db = buildDB([makeCard(1), makeCard(2)]);
    const pool = [1, 2];
    const pack = generatePack(
      { type: "tide_current" },
      { pool, cardDatabase: db, draftedCards: [], pickNumber: 1, packSize: 4 },
    );
    expect(pack.length).toBe(2);
  });

  it("draws unique cards (no duplicates)", () => {
    const db = buildEvenDB(10);
    const pool = Array.from(db.keys());
    const pack = generatePack(
      { type: "tide_current" },
      { pool, cardDatabase: db, draftedCards: [], pickNumber: 1, packSize: 4 },
    );
    expect(new Set(pack).size).toBe(pack.length);
  });

  it("pool_bias strategy statistically favors featured tides", () => {
    const db = buildEvenDB(50);
    const pool = Array.from(db.keys());
    const strategy: PackStrategy = {
      type: "pool_bias",
      featuredTides: ["Bloom", "Arc"],
      featuredWeight: 2.0,
    };

    const tideCounts: Record<string, number> = {};
    const trials = 500;
    for (let i = 0; i < trials; i++) {
      const pack = generatePack(
        strategy,
        { pool, cardDatabase: db, draftedCards: [], pickNumber: 1, packSize: 4 },
      );
      for (const cardNum of pack) {
        const card = db.get(cardNum)!;
        tideCounts[card.tide] = (tideCounts[card.tide] ?? 0) + 1;
      }
    }

    // Featured tides should appear significantly more than non-featured core tides
    const featuredCount = (tideCounts["Bloom"] ?? 0) + (tideCounts["Arc"] ?? 0);
    const nonFeaturedCount = (tideCounts["Ignite"] ?? 0) + (tideCounts["Pact"] ?? 0);
    expect(featuredCount).toBeGreaterThan(nonFeaturedCount * 1.5);
  });

  it("pool_bias does not affect non-featured tide cards", () => {
    // With only non-featured tides in the pool, bias has no effect
    const cards: CardData[] = [];
    for (let i = 1; i <= 40; i++) {
      cards.push(makeCard(i, "Ignite"));
    }
    const db = buildDB(cards);
    const pool = Array.from(db.keys());
    const strategy: PackStrategy = {
      type: "pool_bias",
      featuredTides: ["Bloom", "Arc"],
      featuredWeight: 2.0,
    };

    const pack = generatePack(
      strategy,
      { pool, cardDatabase: db, draftedCards: [], pickNumber: 1, packSize: 4 },
    );
    expect(pack.length).toBe(4);
    for (const cardNum of pack) {
      expect(db.get(cardNum)!.tide).toBe("Ignite");
    }
  });
});

describe("selectFeaturedTides", () => {
  it("returns 2 adjacent tides from available pool", () => {
    const available: Tide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra"];
    const featured = selectFeaturedTides(available);
    expect(featured).toHaveLength(2);

    // Both tides should be in the available pool
    expect(available).toContain(featured[0]);
    expect(available).toContain(featured[1]);
  });

  it("returns only adjacent pairs on the tide circle", () => {
    const available: Tide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
    // Run multiple times to check all returned pairs are adjacent
    const adjacentPairs = new Set([
      "Bloom,Arc", "Arc,Ignite", "Ignite,Pact", "Pact,Umbra",
      "Umbra,Rime", "Rime,Surge", "Surge,Bloom",
    ]);
    for (let i = 0; i < 100; i++) {
      const featured = selectFeaturedTides(available);
      const key = `${featured[0]},${featured[1]}`;
      expect(adjacentPairs.has(key)).toBe(true);
    }
  });

  it("returns empty array when no adjacent pairs are available", () => {
    // Bloom and Ignite are not adjacent (distance 2)
    const available: Tide[] = ["Bloom", "Ignite"];
    const featured = selectFeaturedTides(available);
    expect(featured).toHaveLength(0);
  });

  it("handles wrap-around (Surge + Bloom)", () => {
    const available: Tide[] = ["Surge", "Bloom"];
    const featured = selectFeaturedTides(available);
    expect(featured).toHaveLength(2);
    expect(featured).toContain("Surge");
    expect(featured).toContain("Bloom");
  });
});

describe("enterDraftSite and processPlayerPick", () => {
  it("draws a pack on entering draft site", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, []);
    enterDraftSite(state, db);
    expect(state.currentPack.length).toBe(4);
    expect(state.sitePicksCompleted).toBe(0);
  });

  it("processes a player pick correctly", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, []);
    enterDraftSite(state, db);

    const pack = [...state.currentPack];
    const pickedCard = pack[0];
    const poolBefore = state.pool.length;

    const complete = processPlayerPick(pickedCard, state, db);
    expect(complete).toBe(false);
    expect(state.draftedCards[0]).toBe(pickedCard);
    expect(state.pickNumber).toBe(2);
    expect(state.sitePicksCompleted).toBe(1);
    // All 4 pack cards removed from pool
    expect(state.pool.length).toBe(poolBefore - 4);
    for (const cardNum of pack) {
      expect(state.pool).not.toContain(cardNum);
    }
  });

  it("throws when picking a card not in the pack", () => {
    const db = buildEvenDB(10);
    const state = initializeDraftState(db, []);
    enterDraftSite(state, db);
    expect(() => processPlayerPick(99999, state, db)).toThrow();
  });

  it("completes after SITE_PICKS picks", () => {
    const db = buildEvenDB(20);
    const state = initializeDraftState(db, []);
    enterDraftSite(state, db);

    for (let i = 0; i < SITE_PICKS; i++) {
      const pack = getPlayerPack(state);
      const complete = processPlayerPick(pack[0], state, db);
      if (i < SITE_PICKS - 1) {
        expect(complete).toBe(false);
      } else {
        expect(complete).toBe(true);
      }
    }

    expect(state.sitePicksCompleted).toBe(SITE_PICKS);
    expect(state.draftedCards.length).toBe(SITE_PICKS);
  });
});

describe("completeDraftSite", () => {
  it("logs the drafted cards", () => {
    const db = buildEvenDB(20);
    const state = initializeDraftState(db, []);
    enterDraftSite(state, db);

    for (let i = 0; i < SITE_PICKS; i++) {
      const pack = getPlayerPack(state);
      processPlayerPick(pack[0], state, db);
    }

    resetLog();
    completeDraftSite(state);
    const entries = getLogEntries();
    const completionEvent = entries.find(
      (e) => e.event === "draft_site_completed",
    );
    expect(completionEvent).toBeDefined();
    const eventData = completionEvent as Record<string, unknown>;
    expect(eventData.cardsDrafted).toHaveLength(SITE_PICKS);
  });
});

describe("sortCardsByTide", () => {
  it("sorts cards in tide order", () => {
    const cards = [
      makeCard(1, "Surge"),
      makeCard(2, "Bloom"),
      makeCard(3, "Neutral"),
      makeCard(4, "Arc"),
    ];
    const sorted = sortCardsByTide(cards);
    expect(sorted.map((c) => c.tide)).toEqual([
      "Bloom", "Arc", "Surge", "Neutral",
    ]);
  });

  it("does not mutate the original array", () => {
    const cards = [makeCard(1, "Surge"), makeCard(2, "Bloom")];
    const original = [...cards];
    sortCardsByTide(cards);
    expect(cards).toEqual(original);
  });
});

describe("draft state persistence across sites", () => {
  it("maintains pool and pick number across site visits", () => {
    const db = buildEvenDB(20);
    const state = initializeDraftState(db, []);

    // First site visit
    enterDraftSite(state, db);
    for (let i = 0; i < SITE_PICKS; i++) {
      const pack = getPlayerPack(state);
      processPlayerPick(pack[0], state, db);
    }
    completeDraftSite(state);

    const poolAfterFirst = state.pool.length;
    const pickAfterFirst = state.pickNumber;

    // Second site visit
    enterDraftSite(state, db);
    expect(state.sitePicksCompleted).toBe(0);
    expect(state.pool.length).toBe(poolAfterFirst);

    const pack = getPlayerPack(state);
    processPlayerPick(pack[0], state, db);
    expect(state.pickNumber).toBe(pickAfterFirst + 1);
  });
});
