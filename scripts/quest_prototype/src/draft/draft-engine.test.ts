import { describe, it, expect, beforeEach } from "vitest";
import { resetLog, getLogEntries } from "../logging";
import type { CardData, Tide } from "../types/cards";
import type { ResolvedDreamcallerPackage } from "../types/content";
import {
  initializeDraftState,
  enterDraftSite,
  getPlayerPack,
  processPlayerPick,
  completeDraftSite,
  sortCardsByTide,
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
    tides: [tide],
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

/** Build a card database from an array of cards. */
function buildDB(cards: CardData[]): Map<number, CardData> {
  return new Map(cards.map((c) => [c.cardNumber, c]));
}

/** Create a database with cards for a chosen tide + Neutral. */
function buildTideDB(
  chosenTide: Tide,
  cardsPerTide: number,
): Map<number, CardData> {
  const cards: CardData[] = [];
  let num = 1;
  for (let i = 0; i < cardsPerTide; i++) {
    cards.push(makeCard(num++, chosenTide));
  }
  for (let i = 0; i < cardsPerTide; i++) {
    cards.push(makeCard(num++, "Neutral"));
  }
  // Add some cards from other tides that should be filtered out
  for (let i = 0; i < cardsPerTide; i++) {
    cards.push(makeCard(num++, "Ignite"));
  }
  return buildDB(cards);
}

function buildResolvedPackage(
  copiesByCard: Record<number, number>,
): ResolvedDreamcallerPackage {
  return {
    dreamcaller: {
      id: "test-dreamcaller",
      name: "Test Dreamcaller",
      awakening: 4,
      renderedText: "Test rules text.",
      mandatoryTides: ["core"],
      optionalTides: ["support-a", "support-b", "support-c"],
    },
    mandatoryTides: ["core"],
    optionalSubset: ["support-a", "support-b", "support-c"],
    selectedTides: ["accent:Bloom", "support-a", "support-b", "support-c"],
    draftPoolCopiesByCard: Object.fromEntries(
      Object.entries(copiesByCard).map(([cardNumber, copies]) => [
        cardNumber,
        copies,
      ]),
    ),
    dreamsignPoolIds: [],
    mandatoryOnlyPoolSize: 120,
    draftPoolSize: Object.values(copiesByCard).reduce(
      (total, copies) => total + copies,
      0,
    ),
    doubledCardCount: Object.values(copiesByCard).filter((copies) => copies === 2)
      .length,
    legalSubsetCount: 1,
    preferredSubsetCount: 1,
  };
}

function buildIncludedCardCopies(
  includedCardNumbers: number[],
  copies = 1,
): Record<number, number> {
  return Object.fromEntries(
    includedCardNumbers.map((cardNumber) => [cardNumber, copies]),
  );
}

beforeEach(() => {
  resetLog();
});

describe("initializeDraftState", () => {
  it("creates state from the resolved package pool", () => {
    const db = buildTideDB("Bloom", 10);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(buildIncludedCardCopies([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
      ])),
    );
    expect(state.pool.length).toBe(20); // 10 Bloom + 10 Neutral
    expect(state.draftedCards).toEqual([]);
    expect(state.pickNumber).toBe(1);
    expect(state.sitePicksCompleted).toBe(0);
    expect(state.packStrategy.type).toBe("depletion");
    expect(state.seenCards).toEqual([]);
  });

  it("excludes cards outside the resolved package pool", () => {
    const db = buildTideDB("Bloom", 10);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(buildIncludedCardCopies([
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
      ])),
    );

    for (const cardNum of state.pool) {
      const card = db.get(cardNum)!;
      expect(card.tides[0] === "Bloom" || card.tides[0] === "Neutral").toBe(true);
    }
  });

  it("preserves duplicate copies from draftPoolCopiesByCard", () => {
    const db = buildTideDB("Bloom", 10);
    const state = initializeDraftState(
      db,
      buildResolvedPackage({
        1: 2,
        2: 1,
        11: 2,
      }),
    );

    expect(state.pool.filter((cardNumber) => cardNumber === 1)).toHaveLength(2);
    expect(state.pool.filter((cardNumber) => cardNumber === 11)).toHaveLength(2);
    expect(state.pool).not.toContain(21);
  });

  it("logs pool initialization", () => {
    const db = buildTideDB("Bloom", 5);
    initializeDraftState(
      db,
      buildResolvedPackage(buildIncludedCardCopies([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])),
    );
    const entries = getLogEntries();
    const initEvent = entries.find(
      (e) => e.event === "draft_pool_initialized",
    );
    expect(initEvent).toBeDefined();
    expect((initEvent as Record<string, unknown>).poolSize).toBe(10);
  });
});

describe("enterDraftSite and processPlayerPick", () => {
  it("draws a pack on entering draft site", () => {
    const db = buildTideDB("Bloom", 20);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 40 }, (_, index) => index + 1),
        ),
      ),
    );
    enterDraftSite(state, db);
    expect(state.currentPack.length).toBe(4);
    expect(state.sitePicksCompleted).toBe(0);
  });

  it("processes a player pick correctly", () => {
    const db = buildTideDB("Bloom", 20);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 40 }, (_, index) => index + 1),
        ),
      ),
    );
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

  it("tracks unpicked cards as seen", () => {
    const db = buildTideDB("Bloom", 20);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 40 }, (_, index) => index + 1),
        ),
      ),
    );
    enterDraftSite(state, db);

    const pack = [...state.currentPack];
    const pickedCard = pack[0];
    processPlayerPick(pickedCard, state, db);

    // The 3 unpicked cards should be in seenCards
    expect(state.seenCards.length).toBe(3);
    for (const cardNum of pack) {
      if (cardNum !== pickedCard) {
        expect(state.seenCards).toContain(cardNum);
      }
    }
  });

  it("throws when picking a card not in the pack", () => {
    const db = buildTideDB("Bloom", 20);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 40 }, (_, index) => index + 1),
        ),
      ),
    );
    enterDraftSite(state, db);
    expect(() => processPlayerPick(99999, state, db)).toThrow();
  });

  it("completes after SITE_PICKS picks", () => {
    const db = buildTideDB("Bloom", 40);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 80 }, (_, index) => index + 1),
        ),
      ),
    );
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
    const db = buildTideDB("Bloom", 40);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 80 }, (_, index) => index + 1),
        ),
      ),
    );
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
    expect(sorted.map((c) => c.tides[0])).toEqual([
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
    const db = buildTideDB("Bloom", 40);
    const state = initializeDraftState(
      db,
      buildResolvedPackage(
        buildIncludedCardCopies(
          Array.from({ length: 80 }, (_, index) => index + 1),
        ),
      ),
    );

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
