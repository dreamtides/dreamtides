import { describe, it, expect, beforeEach, vi } from "vitest";
import { resetLog, getLogEntries } from "../logging";
import type { CardData, Tide } from "../types/cards";
import {
  DEFAULT_DRAFT_CONFIG,
  initializeDraftState,
  refreshPool,
  dealRound,
  computeFitness,
  normalize,
  scoreCard,
  botPick,
  rotatePacks,
  playerPick,
  advancePick,
  enterDraftSite,
  getPlayerPack,
  processPlayerPick,
  completeDraftSite,
  sortCardsByTide,
} from "./draft-engine";

/** Helper to build a minimal CardData for testing. */
function makeCard(
  cardNumber: number,
  tide: Tide = "Bloom",
  rarity: "Common" | "Uncommon" | "Rare" | "Legendary" = "Common",
): CardData {
  return {
    name: `TestCard${String(cardNumber)}`,
    id: `id-${String(cardNumber)}`,
    cardNumber,
    cardType: "Character",
    subtype: "",
    rarity,
    energyCost: 3,
    spark: 2,
    isFast: false,
    tide,
    tideCost: 1,
    renderedText: "Test text",
    imageNumber: 100000 + cardNumber,
    artOwned: false,
  };
}

/** Build a card database with the given number of cards, distributed across tides. */
function makeDatabase(count: number): Map<number, CardData> {
  const tides: Tide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
  const db = new Map<number, CardData>();
  for (let i = 1; i <= count; i++) {
    db.set(i, makeCard(i, tides[(i - 1) % tides.length]));
  }
  return db;
}

beforeEach(() => {
  resetLog();
  vi.restoreAllMocks();
  vi.spyOn(console, "log").mockImplementation(() => {});
});

describe("computeFitness", () => {
  it("returns 1.0 at the correct index for a named tide card", () => {
    const card = makeCard(1, "Arc");
    const fitness = computeFitness(card);
    expect(fitness).toHaveLength(7);
    // Arc is index 1
    expect(fitness[1]).toBe(1.0);
    expect(fitness[0]).toBe(0.0);
    expect(fitness[2]).toBe(0.0);
  });

  it("returns 0.15 uniform for a Neutral card", () => {
    const card = makeCard(1, "Neutral");
    const fitness = computeFitness(card);
    expect(fitness).toHaveLength(7);
    for (const v of fitness) {
      expect(v).toBeCloseTo(0.15);
    }
  });

  it("maps each named tide to the correct index", () => {
    const tides: Tide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
    for (let i = 0; i < tides.length; i++) {
      const card = makeCard(1, tides[i]);
      const fitness = computeFitness(card);
      expect(fitness[i]).toBe(1.0);
      const otherSum = fitness.reduce((s, v) => s + v, 0) - 1.0;
      expect(otherSum).toBeCloseTo(0.0);
    }
  });
});

describe("normalize", () => {
  it("normalizes a vector to sum to 1", () => {
    const result = normalize([3, 1, 1, 0, 0, 0, 2]);
    const sum = result.reduce((s, v) => s + v, 0);
    expect(sum).toBeCloseTo(1.0);
    expect(result[0]).toBeCloseTo(3 / 7);
  });

  it("returns uniform distribution for all-zero vector", () => {
    const result = normalize([0, 0, 0, 0, 0, 0, 0]);
    expect(result).toHaveLength(7);
    for (const v of result) {
      expect(v).toBeCloseTo(1 / 7);
    }
  });

  it("handles a single non-zero element", () => {
    const result = normalize([0, 0, 5, 0, 0, 0, 0]);
    expect(result[2]).toBeCloseTo(1.0);
    for (let i = 0; i < 7; i++) {
      if (i !== 2) expect(result[i]).toBeCloseTo(0.0);
    }
  });
});

describe("scoreCard", () => {
  it("computes weighted sum of preference, openness, and rarity", () => {
    const card = makeCard(1, "Bloom", "Rare");
    // Bloom fitness: [1, 0, 0, 0, 0, 0, 0]
    // preference = [7, 0, 0, 0, 0, 0, 0] => normalized = [1, 0, 0, 0, 0, 0, 0]
    // dot(fitness, pref) = 1.0
    // openness with empty history => uniform [1/7, ...]
    // dot(fitness, uniform) = 1/7
    // rarity: Rare = 0.67
    // score = 0.6 * 1.0 + 0.2 * (1/7) + 0.2 * 0.67
    const agent = {
      preference: [7, 0, 0, 0, 0, 0, 0],
      opennessHistory: [],
      picks: [],
    };
    const score = scoreCard(card, agent, DEFAULT_DRAFT_CONFIG);
    const expected = 0.6 * 1.0 + 0.2 * (1 / 7) + 0.2 * 0.67;
    expect(score).toBeCloseTo(expected, 4);
  });

  it("uses uniform preference when all zeros", () => {
    const card = makeCard(1, "Bloom", "Common");
    const agent = {
      preference: [0, 0, 0, 0, 0, 0, 0],
      opennessHistory: [],
      picks: [],
    };
    const score = scoreCard(card, agent, DEFAULT_DRAFT_CONFIG);
    // preference normalized to uniform, dot with bloom fitness = 1/7
    // openness uniform too, dot = 1/7
    // rarity: Common = 0.0
    const expected = 0.6 * (1 / 7) + 0.2 * (1 / 7) + 0.2 * 0.0;
    expect(score).toBeCloseTo(expected, 4);
  });
});

describe("initializeDraftState", () => {
  it("creates a pool of all card numbers from the database", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    expect(state.pool).toHaveLength(483);
    expect(state.agents).toHaveLength(10);
    expect(state.isActive).toBe(false);
    expect(state.currentRound).toBe(0);
    expect(state.totalPicks).toBe(0);
  });

  it("initializes 10 agents with zero preference vectors", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    for (const agent of state.agents) {
      expect(agent.preference).toHaveLength(7);
      expect(agent.preference.every((v) => v === 0)).toBe(true);
      expect(agent.opennessHistory).toHaveLength(0);
      expect(agent.picks).toHaveLength(0);
    }
  });

  it("logs draft_pool_initialized event", () => {
    const db = makeDatabase(483);
    initializeDraftState(db);
    const entries = getLogEntries();
    const initEvent = entries.find((e) => e.event === "draft_pool_initialized");
    expect(initEvent).toBeDefined();
    expect(initEvent?.poolSize).toBe(483);
  });
});

describe("dealRound", () => {
  it("creates 10 packs of 15 cards from the pool", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    expect(state.packs).toHaveLength(10);
    for (const pack of state.packs) {
      expect(pack).toHaveLength(15);
    }
    // Pool should shrink by 150
    expect(state.pool).toHaveLength(483 - 150);
    expect(state.isActive).toBe(true);
  });

  it("draws cards without replacement (no duplicates across packs)", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const allDealt = state.packs.flat();
    const uniqueDealt = new Set(allDealt);
    expect(uniqueDealt.size).toBe(150);
  });

  it("logs draft_round_started event", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const entries = getLogEntries();
    const roundEvent = entries.find((e) => e.event === "draft_round_started");
    expect(roundEvent).toBeDefined();
    expect(roundEvent?.roundNumber).toBe(0);
  });

  it("handles pool smaller than 150 by creating smaller packs", () => {
    const db = makeDatabase(50);
    const state = initializeDraftState(db);
    dealRound(state);
    const totalDealt = state.packs.flat().length;
    expect(totalDealt).toBe(50);
    expect(state.pool).toHaveLength(0);
  });
});

describe("rotatePacks", () => {
  it("always rotates left: seat N's pack goes to seat N+1", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const packsBefore = state.packs.map((p) => [...p]);
    rotatePacks(state);
    // Seat 1 should now have what seat 0 had
    expect(state.packs[1]).toEqual(packsBefore[0]);
    // Seat 0 should have what seat 9 had (wrap)
    expect(state.packs[0]).toEqual(packsBefore[9]);
  });

  it("rotates the same direction regardless of round number", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const packsBefore = state.packs.map((p) => [...p]);

    state.currentRound = 2;
    rotatePacks(state);
    // Still left: seat 1 gets seat 0's pack
    expect(state.packs[1]).toEqual(packsBefore[0]);
    expect(state.packs[0]).toEqual(packsBefore[9]);
  });

  it("emits draft_packs_rotated event", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    rotatePacks(state);
    const entries = getLogEntries();
    const event = entries.find((e) => e.event === "draft_packs_rotated");
    expect(event).toBeDefined();
    expect(event?.roundNumber).toBe(0);
  });
});

describe("playerPick", () => {
  it("removes the picked card from seat 0 pack and adds to picks", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const cardToPick = state.packs[0][0];
    playerPick(cardToPick, state, db);
    expect(state.packs[0]).not.toContain(cardToPick);
    expect(state.agents[0].picks).toContain(cardToPick);
  });

  it("logs draft_pick_player event with pack contents", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const cardToPick = state.packs[0][0];
    playerPick(cardToPick, state, db);
    const entries = getLogEntries();
    const pickEvent = entries.find((e) => e.event === "draft_pick_player");
    expect(pickEvent).toBeDefined();
    expect(pickEvent?.cardNumber).toBe(cardToPick);
    expect(pickEvent?.packContents).toBeDefined();
  });

  it("returns false when the card is not in the pack", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const result = playerPick(999999, state, db);
    expect(result).toBe(false);
    expect(state.agents[0].picks).toHaveLength(0);
  });
});

describe("botPick", () => {
  it("removes a card from the bot seat pack and adds to picks", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    const packSizeBefore = state.packs[1].length;
    botPick(1, state, db, DEFAULT_DRAFT_CONFIG);
    expect(state.packs[1]).toHaveLength(packSizeBefore - 1);
    expect(state.agents[1].picks).toHaveLength(1);
  });

  it("updates the bot preference vector after picking", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    botPick(1, state, db, DEFAULT_DRAFT_CONFIG);
    // At least one element of preference should be non-zero now
    const hasNonZero = state.agents[1].preference.some((v) => v > 0);
    expect(hasNonZero).toBe(true);
  });

  it("logs draft_pick_bot event", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    botPick(1, state, db, DEFAULT_DRAFT_CONFIG);
    const entries = getLogEntries();
    const botEvent = entries.find((e) => e.event === "draft_pick_bot");
    expect(botEvent).toBeDefined();
    expect(botEvent?.seatNumber).toBe(1);
  });
});

describe("advancePick", () => {
  it("increments currentPick and totalPicks", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    advancePick(state, db);
    expect(state.currentPick).toBe(1);
    expect(state.totalPicks).toBe(1);
  });
});

describe("refreshPool", () => {
  it("creates a fresh pool of 483 cards and resets counters", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state);
    state.totalPicks = 30;
    state.currentRound = 2;
    state.currentPick = 9;
    // Give bots some preferences and stale openness history
    state.agents[1].preference = [1, 2, 3, 4, 5, 6, 7];
    state.agents[1].opennessHistory = [[0.1, 0.2, 0.3, 0.1, 0.1, 0.1, 0.1]];
    refreshPool(state, db);
    expect(state.pool).toHaveLength(483);
    expect(state.currentRound).toBe(0);
    expect(state.currentPick).toBe(0);
    expect(state.totalPicks).toBe(0);
    expect(state.isActive).toBe(false);
    // Bot preferences should persist
    expect(state.agents[1].preference).toEqual([1, 2, 3, 4, 5, 6, 7]);
    // Openness history should be cleared (stale signals from previous pool)
    expect(state.agents[1].opennessHistory).toHaveLength(0);
  });

  it("logs draft_pool_refreshed event", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    refreshPool(state, db);
    const entries = getLogEntries();
    const event = entries.find((e) => e.event === "draft_pool_refreshed");
    expect(event).toBeDefined();
    expect(event?.poolSize).toBe(483);
  });
});

describe("enterDraftSite", () => {
  it("deals a round if no active packs exist", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    expect(state.isActive).toBe(true);
    expect(state.packs.every((p) => p.length > 0)).toBe(true);
    expect(state.sitePicksCompleted).toBe(0);
  });

  it("logs draft_site_entered event", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    const entries = getLogEntries();
    const event = entries.find((e) => e.event === "draft_site_entered");
    expect(event).toBeDefined();
  });
});

describe("getPlayerPack", () => {
  it("returns the current pack for seat 0", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    const pack = getPlayerPack(state);
    expect(pack).toEqual(state.packs[0]);
  });
});

describe("processPlayerPick", () => {
  it("completes a full pick cycle: player pick, bot picks, rotation, advancement", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    const cardToPick = state.packs[0][0];
    const done = processPlayerPick(cardToPick, state, db);
    expect(done).toBe(false); // Only 1 of 5 site picks done
    expect(state.agents[0].picks).toContain(cardToPick);
    expect(state.sitePicksCompleted).toBe(1);
    // All 9 bots should have picked
    for (let i = 1; i < 10; i++) {
      expect(state.agents[i].picks).toHaveLength(1);
    }
  });

  it("returns true after 5 picks (site batch complete)", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    for (let i = 0; i < 5; i++) {
      const pack = getPlayerPack(state);
      const card = pack[0];
      const done = processPlayerPick(card, state, db);
      if (i < 4) {
        expect(done).toBe(false);
      } else {
        expect(done).toBe(true);
      }
    }
    expect(state.sitePicksCompleted).toBe(5);
  });

  it("throws an error when the card is not in the player's pack", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    expect(() => processPlayerPick(999999, state, db)).toThrow(
      "Card 999999 is not in seat 0's current pack",
    );
    // State should not be corrupted: no bot picks, no rotation, no advancement
    expect(state.sitePicksCompleted).toBe(0);
    expect(state.currentPick).toBe(0);
    for (let i = 1; i < 10; i++) {
      expect(state.agents[i].picks).toHaveLength(0);
    }
  });
});

describe("completeDraftSite", () => {
  it("logs draft_site_completed event with cards drafted", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);
    for (let i = 0; i < 5; i++) {
      const pack = getPlayerPack(state);
      processPlayerPick(pack[0], state, db);
    }
    completeDraftSite(state);
    const entries = getLogEntries();
    const event = entries.find((e) => e.event === "draft_site_completed");
    expect(event).toBeDefined();
    expect(event?.cardsDrafted).toBeDefined();
  });
});

describe("DEFAULT_DRAFT_CONFIG", () => {
  it("has the correct configuration values", () => {
    expect(DEFAULT_DRAFT_CONFIG.seatCount).toBe(10);
    expect(DEFAULT_DRAFT_CONFIG.packSize).toBe(15);
    expect(DEFAULT_DRAFT_CONFIG.roundsPerPool).toBe(3);
    expect(DEFAULT_DRAFT_CONFIG.picksPerRound).toBe(10);
    expect(DEFAULT_DRAFT_CONFIG.tideCount).toBe(7);
    expect(DEFAULT_DRAFT_CONFIG.preferenceWeight).toBe(0.6);
    expect(DEFAULT_DRAFT_CONFIG.signalWeight).toBe(0.2);
    expect(DEFAULT_DRAFT_CONFIG.rarityWeight).toBe(0.2);
    expect(DEFAULT_DRAFT_CONFIG.aiOptimality).toBe(0.8);
    expect(DEFAULT_DRAFT_CONFIG.learningRate).toBe(3.0);
    expect(DEFAULT_DRAFT_CONFIG.opennessWindow).toBe(3);
  });
});

describe("full draft flow integration", () => {
  it("processes 10 picks (a full round) correctly", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    enterDraftSite(state, db);

    // First 5 picks (site 1)
    for (let i = 0; i < 5; i++) {
      const pack = getPlayerPack(state);
      processPlayerPick(pack[0], state, db);
    }
    completeDraftSite(state);

    // Second 5 picks (site 2)
    enterDraftSite(state, db);
    for (let i = 0; i < 5; i++) {
      const pack = getPlayerPack(state);
      processPlayerPick(pack[0], state, db);
    }
    completeDraftSite(state);

    expect(state.agents[0].picks).toHaveLength(10);
    // After 10 picks, round should have advanced
    expect(state.totalPicks).toBe(10);
  });

  it("triggers pool refresh after 30 total picks", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);

    // Process 30 picks (3 rounds of 10)
    for (let round = 0; round < 3; round++) {
      enterDraftSite(state, db);
      for (let i = 0; i < 5; i++) {
        const pack = getPlayerPack(state);
        processPlayerPick(pack[0], state, db);
      }
      completeDraftSite(state);

      enterDraftSite(state, db);
      for (let i = 0; i < 5; i++) {
        const pack = getPlayerPack(state);
        processPlayerPick(pack[0], state, db);
      }
      completeDraftSite(state);
    }

    // After 30 picks, pool should have been refreshed
    expect(state.agents[0].picks).toHaveLength(30);
    // Pool should be fresh (483 cards)
    expect(state.pool).toHaveLength(483);

    const entries = getLogEntries();
    const refreshEvents = entries.filter((e) => e.event === "draft_pool_refreshed");
    expect(refreshEvents.length).toBeGreaterThanOrEqual(1);
  });
});

describe("sortCardsByTide", () => {
  it("sorts cards by tide order: Bloom, Arc, Ignite, Pact, Umbra, Rime, Surge, Neutral", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Surge"));
    db.set(2, makeCard(2, "Bloom"));
    db.set(3, makeCard(3, "Neutral"));
    db.set(4, makeCard(4, "Ignite"));
    db.set(5, makeCard(5, "Arc"));

    const cards = [db.get(1)!, db.get(2)!, db.get(3)!, db.get(4)!, db.get(5)!];
    const sorted = sortCardsByTide(cards);

    expect(sorted.map((c) => c.tide)).toEqual([
      "Bloom",
      "Arc",
      "Ignite",
      "Surge",
      "Neutral",
    ]);
  });

  it("preserves original order for cards of the same tide", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom"));
    db.set(2, makeCard(2, "Bloom"));
    db.set(3, makeCard(3, "Bloom"));

    const cards = [db.get(1)!, db.get(2)!, db.get(3)!];
    const sorted = sortCardsByTide(cards);

    expect(sorted.map((c) => c.cardNumber)).toEqual([1, 2, 3]);
  });

  it("returns an empty array for empty input", () => {
    expect(sortCardsByTide([])).toEqual([]);
  });

  it("does not mutate the original array", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Surge"));
    db.set(2, makeCard(2, "Bloom"));

    const cards = [db.get(1)!, db.get(2)!];
    const original = [...cards];
    sortCardsByTide(cards);

    expect(cards.map((c) => c.cardNumber)).toEqual(original.map((c) => c.cardNumber));
  });
});

describe("balanced seeding", () => {
  it("distributes cards evenly across tides when using balanced algorithm", () => {
    const db = makeDatabase(483);
    const config = {
      ...DEFAULT_DRAFT_CONFIG,
      seedingAlgorithm: "balanced" as const,
    };
    const state = initializeDraftState(db, config);
    dealRound(state, db, config);

    // Each pack should have approximately 2 cards per tide (15/7 ~ 2.14)
    for (const pack of state.packs) {
      const tideCounts: Record<string, number> = {};
      for (const cardNum of pack) {
        const card = db.get(cardNum);
        if (card) {
          tideCounts[card.tide] = (tideCounts[card.tide] ?? 0) + 1;
        }
      }
      // Each tide should have at least 1 card and at most 3 in a 15-card pack
      for (const count of Object.values(tideCounts)) {
        expect(count).toBeGreaterThanOrEqual(1);
        expect(count).toBeLessThanOrEqual(3);
      }
    }
  });

  it("creates packs of the correct size with balanced seeding", () => {
    const db = makeDatabase(483);
    const config = {
      ...DEFAULT_DRAFT_CONFIG,
      seedingAlgorithm: "balanced" as const,
    };
    const state = initializeDraftState(db, config);
    dealRound(state, db, config);

    expect(state.packs).toHaveLength(10);
    for (const pack of state.packs) {
      expect(pack).toHaveLength(15);
    }
  });

  it("draws cards without replacement using balanced seeding", () => {
    const db = makeDatabase(483);
    const config = {
      ...DEFAULT_DRAFT_CONFIG,
      seedingAlgorithm: "balanced" as const,
    };
    const state = initializeDraftState(db, config);
    dealRound(state, db, config);

    const allDealt = state.packs.flat();
    const uniqueDealt = new Set(allDealt);
    expect(uniqueDealt.size).toBe(150);
  });

  it("defaults to balanced seeding by default", () => {
    const db = makeDatabase(483);
    const state = initializeDraftState(db);
    dealRound(state, db);

    expect(state.packs).toHaveLength(10);
    for (const pack of state.packs) {
      expect(pack).toHaveLength(15);
    }
  });

  it("handles a pool with missing tides gracefully", () => {
    // Create a database with only 2 tides
    const db = new Map<number, CardData>();
    for (let i = 1; i <= 200; i++) {
      db.set(i, makeCard(i, i % 2 === 0 ? "Bloom" : "Arc"));
    }

    const config = {
      ...DEFAULT_DRAFT_CONFIG,
      seedingAlgorithm: "balanced" as const,
    };
    const state = initializeDraftState(db, config);
    dealRound(state, db, config);

    expect(state.packs).toHaveLength(10);
    for (const pack of state.packs) {
      expect(pack).toHaveLength(15);
    }
  });
});
