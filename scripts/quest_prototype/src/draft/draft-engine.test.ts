import { describe, it, expect, beforeEach } from "vitest";
import { resetLog, getLogEntries } from "../logging";
import type { CardData, Tide } from "../types/cards";
import type { DraftState } from "../types/draft";
import type { ResolvedDreamcallerPackage } from "../types/content";
import {
  completeDraftSite,
  enterDraftSite,
  getPlayerPack,
  initializeDraftState,
  processPlayerPick,
  SITE_PICKS,
  sortCardsByTide,
} from "./draft-engine";

function makeCard(
  cardNumber: number,
  tide: Tide = "Bloom",
): CardData {
  return {
    name: `TestCard${String(cardNumber)}`,
    id: `test-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity: "Common",
    energyCost: 3,
    spark: 1,
    isFast: false,
    tides: [tide],
    renderedText: "",
    imageNumber: cardNumber,
    artOwned: false,
  };
}

function buildDB(cards: CardData[]): Map<number, CardData> {
  return new Map(cards.map((card) => [card.cardNumber, card]));
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

function makeDraftState(
  overrides: Partial<DraftState> = {},
): DraftState {
  return {
    remainingCopiesByCard: {},
    currentOffer: [],
    draftedCardNumbers: [],
    pickNumber: 1,
    sitePicksCompleted: 0,
    ...overrides,
  };
}

beforeEach(() => {
  resetLog();
});

describe("initializeDraftState", () => {
  it("creates state from the resolved package pool", () => {
    const cardDatabase = buildDB([
      makeCard(1),
      makeCard(2),
      makeCard(3),
    ]);

    const state = initializeDraftState(
      cardDatabase,
      buildResolvedPackage({ 1: 2, 2: 1, 999: 3 }),
    );

    expect(state.remainingCopiesByCard).toEqual({
      "1": 2,
      "2": 1,
    });
    expect(state.currentOffer).toEqual([]);
    expect(state.draftedCardNumbers).toEqual([]);
    expect(state.pickNumber).toBe(1);
    expect(state.sitePicksCompleted).toBe(0);
  });

  it("logs pool initialization", () => {
    const cardDatabase = buildDB([
      makeCard(1),
      makeCard(2),
      makeCard(3),
      makeCard(4),
    ]);

    initializeDraftState(
      cardDatabase,
      buildResolvedPackage({ 1: 1, 2: 1, 3: 1, 4: 1 }),
    );

    const initEvent = getLogEntries().find(
      (entry) => entry.event === "draft_pool_initialized",
    );
    expect(initEvent).toBeDefined();
    expect(initEvent?.poolSize).toBe(4);
    expect(initEvent?.uniqueCardCount).toBe(4);
  });
});

describe("enterDraftSite and processPlayerPick", () => {
  it("draws a 4-unique-card offer on entering a draft site", () => {
    const cardDatabase = buildDB(
      Array.from({ length: 6 }, (_, index) => makeCard(index + 1)),
    );
    const state = initializeDraftState(
      cardDatabase,
      buildResolvedPackage({ 1: 1, 2: 2, 3: 1, 4: 1, 5: 1, 6: 1 }),
    );

    enterDraftSite(state, cardDatabase);

    expect(state.currentOffer).toHaveLength(4);
    expect(new Set(state.currentOffer).size).toBe(4);
    expect(getPlayerPack(state)).toEqual(state.currentOffer);
  });

  it("spends shown cards from the remaining copy pool", () => {
    const cardDatabase = buildDB(
      Array.from({ length: 8 }, (_, index) => makeCard(index + 1)),
    );
    const state = makeDraftState({
      remainingCopiesByCard: {
        "1": 2,
        "2": 1,
        "3": 1,
        "4": 1,
        "5": 1,
        "6": 1,
        "7": 1,
        "8": 1,
      },
      currentOffer: [1, 2, 3, 4],
    });

    const isComplete = processPlayerPick(1, state, cardDatabase);

    expect(isComplete).toBe(false);
    expect(state.pickNumber).toBe(2);
    expect(state.sitePicksCompleted).toBe(1);
    expect(state.draftedCardNumbers).toEqual([1]);
    expect(state.remainingCopiesByCard).toEqual({
      "1": 1,
      "5": 1,
      "6": 1,
      "7": 1,
      "8": 1,
    });
    expect(new Set(state.currentOffer).size).toBe(state.currentOffer.length);
  });

  it("ends the site cleanly when fewer than 4 unique cards remain", () => {
    const cardDatabase = buildDB(
      Array.from({ length: 7 }, (_, index) => makeCard(index + 1)),
    );
    const state = makeDraftState({
      remainingCopiesByCard: {
        "1": 1,
        "2": 1,
        "3": 1,
        "4": 1,
        "5": 1,
        "6": 1,
        "7": 1,
      },
      currentOffer: [1, 2, 3, 4],
    });

    const isComplete = processPlayerPick(1, state, cardDatabase);

    expect(isComplete).toBe(true);
    expect(state.currentOffer).toEqual([]);
  });

  it("still completes after SITE_PICKS picks when offers remain", () => {
    const cardDatabase = buildDB(
      Array.from({ length: 24 }, (_, index) => makeCard(index + 1)),
    );
    const state = initializeDraftState(
      cardDatabase,
      buildResolvedPackage(
        Object.fromEntries(
          Array.from({ length: 24 }, (_, index) => [index + 1, 1]),
        ),
      ),
    );

    enterDraftSite(state, cardDatabase);
    for (let pickIndex = 0; pickIndex < SITE_PICKS; pickIndex += 1) {
      const currentOffer = getPlayerPack(state);
      const isComplete = processPlayerPick(
        currentOffer[0],
        state,
        cardDatabase,
      );
      expect(isComplete).toBe(pickIndex === SITE_PICKS - 1);
    }
  });
});

describe("completeDraftSite", () => {
  it("logs the drafted cards stored in draft state", () => {
    const state = makeDraftState({
      remainingCopiesByCard: { "9": 1, "10": 1 },
      draftedCardNumbers: [4, 7],
      sitePicksCompleted: 2,
    });

    completeDraftSite(state);

    const completionEvent = getLogEntries().find(
      (entry) => entry.event === "draft_site_completed",
    );
    expect(completionEvent).toBeDefined();
    expect(completionEvent?.cardsDrafted).toEqual([4, 7]);
    expect(completionEvent?.picksCompleted).toBe(2);
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

    expect(sortCardsByTide(cards).map((card) => card.tides[0])).toEqual([
      "Bloom",
      "Arc",
      "Surge",
      "Neutral",
    ]);
  });
});
