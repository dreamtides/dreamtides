import { describe, it, expect } from "vitest";
import { extractDraftDebugInfo } from "./debug-helpers";
import type { DraftState, AgentState } from "../types/draft";
import type { CardData } from "../types/cards";

function makeAgent(preference: number[], picks: number[]): AgentState {
  return { preference, opennessHistory: [], picks };
}

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
  agents: AgentState[],
  currentRound = 0,
): DraftState {
  return {
    pool: [],
    packs: agents.map(() => []),
    agents,
    currentRound,
    currentPick: 0,
    totalPicks: 0,
    isActive: true,
    sitePicksCompleted: 0,
  };
}

describe("extractDraftDebugInfo", () => {
  it("returns null when draft state is null", () => {
    const result = extractDraftDebugInfo(null, new Map());
    expect(result).toBeNull();
  });

  it("includes seat 0 (the player) marked as isPlayer", () => {
    const agents = [
      makeAgent([10, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 5, 0, 0, 0, 0, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.seats).toHaveLength(2);
    expect(result.seats[0].seatIndex).toBe(0);
    expect(result.seats[0].isPlayer).toBe(true);
    expect(result.seats[1].seatIndex).toBe(1);
    expect(result.seats[1].isPlayer).toBe(false);
  });

  it("identifies primary and secondary tides from preference vector", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([3, 9, 0, 0, 0, 6, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.seats[1].primaryTide).toBe("Arc");
    expect(result.seats[1].secondaryTide).toBe("Rime");
  });

  it("returns null tides when preference is all zeros", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, new Map())!;
    expect(result.seats[1].primaryTide).toBeNull();
    expect(result.seats[1].secondaryTide).toBeNull();
  });

  it("resolves card data for drafted cards", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "Rose Golem"));
    db.set(2, makeCard(2, "Arc", "Lightning Sprite"));

    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([3, 6, 0, 0, 0, 0, 0], [1, 2]),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, db)!;

    expect(result.seats[1].draftedCards).toHaveLength(2);
    expect(result.seats[1].draftedCards[0].name).toBe("Rose Golem");
    expect(result.seats[1].draftedCards[1].name).toBe("Lightning Sprite");
  });

  it("sorts seats by seat index", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 0, 10, 0, 0, 0, 0], []),
      makeAgent([10, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 0, 0, 0, 0, 0, 10], []),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, new Map())!;

    expect(result.seats[0].seatIndex).toBe(0);
    expect(result.seats[1].seatIndex).toBe(1);
    expect(result.seats[2].seatIndex).toBe(2);
    expect(result.seats[3].seatIndex).toBe(3);
  });

  it("computes normalized preference weights", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([3, 0, 0, 0, 0, 0, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, new Map())!;

    const weights = result.seats[1].preferenceWeights;
    expect(weights.Bloom).toBeCloseTo(1.0);
    expect(weights.Arc).toBeCloseTo(0.0);
  });

  it("groups drafted cards by tide", () => {
    const db = new Map<number, CardData>();
    db.set(1, makeCard(1, "Bloom", "Rose Golem"));
    db.set(2, makeCard(2, "Bloom", "Vine Crawler"));
    db.set(3, makeCard(3, "Arc", "Spark Mage"));

    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([6, 3, 0, 0, 0, 0, 0], [1, 2, 3]),
    ];
    const state = makeDraftState(agents);
    const result = extractDraftDebugInfo(state, db)!;

    expect(result.seats[1].cardsByTide.Bloom).toBe(2);
    expect(result.seats[1].cardsByTide.Arc).toBe(1);
  });

  it("always passes left: seat 9 passes to player", () => {
    const agents = Array.from({ length: 10 }, () =>
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
    );
    const state = makeDraftState(agents, 0);
    const result = extractDraftDebugInfo(state, new Map())!;

    expect(result.seatPassingToPlayer).toBe(9);
    expect(result.displayRound).toBe(1);
    expect(result.seats[0].receivesFromSeat).toBe(9);
  });

  it("passing direction is the same regardless of round", () => {
    const agents = Array.from({ length: 10 }, () =>
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
    );
    const round0 = extractDraftDebugInfo(makeDraftState(agents, 0), new Map())!;
    const round1 = extractDraftDebugInfo(makeDraftState(agents, 1), new Map())!;

    expect(round0.seatPassingToPlayer).toBe(9);
    expect(round1.seatPassingToPlayer).toBe(9);
    expect(round0.seats[0].receivesFromSeat).toBe(9);
    expect(round1.seats[0].receivesFromSeat).toBe(9);
  });

  it("computes receivesFromSeat correctly for all seats", () => {
    const agents = Array.from({ length: 4 }, () =>
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
    );
    const state = makeDraftState(agents, 0);
    const result = extractDraftDebugInfo(state, new Map())!;

    // Packs always pass left: seat i receives from seat i-1
    expect(result.seats[0].receivesFromSeat).toBe(3);
    expect(result.seats[1].receivesFromSeat).toBe(0);
    expect(result.seats[2].receivesFromSeat).toBe(1);
    expect(result.seats[3].receivesFromSeat).toBe(2);
  });
});
