import { describe, it, expect } from "vitest";
import { extractBotSummaries } from "./debug-helpers";
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

function makeDraftState(agents: AgentState[]): DraftState {
  return {
    pool: [],
    packs: agents.map(() => []),
    agents,
    currentRound: 0,
    currentPick: 0,
    totalPicks: 0,
    isActive: true,
    sitePicksCompleted: 0,
  };
}

describe("extractBotSummaries", () => {
  it("returns empty array when draft state is null", () => {
    const result = extractBotSummaries(null, new Map());
    expect(result).toEqual([]);
  });

  it("excludes seat 0 (the player) from results", () => {
    const agents = [
      makeAgent([10, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 5, 0, 0, 0, 0, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractBotSummaries(state, new Map());
    expect(result).toHaveLength(1);
    expect(result[0].seatIndex).toBe(1);
  });

  it("identifies primary and secondary tides from preference vector", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([3, 9, 0, 0, 0, 6, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractBotSummaries(state, new Map());
    expect(result[0].primaryTide).toBe("Arc");
    expect(result[0].secondaryTide).toBe("Rime");
  });

  it("returns null tides when preference is all zeros", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractBotSummaries(state, new Map());
    expect(result[0].primaryTide).toBeNull();
    expect(result[0].secondaryTide).toBeNull();
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
    const result = extractBotSummaries(state, db);

    expect(result[0].draftedCards).toHaveLength(2);
    expect(result[0].draftedCards[0].name).toBe("Rose Golem");
    expect(result[0].draftedCards[1].name).toBe("Lightning Sprite");
  });

  it("sorts bots by primary tide name then by seat index", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([0, 0, 10, 0, 0, 0, 0], []),  // Ignite
      makeAgent([10, 0, 0, 0, 0, 0, 0], []),   // Bloom
      makeAgent([0, 0, 0, 0, 0, 0, 10], []),   // Surge
    ];
    const state = makeDraftState(agents);
    const result = extractBotSummaries(state, new Map());

    expect(result[0].primaryTide).toBe("Bloom");
    expect(result[1].primaryTide).toBe("Ignite");
    expect(result[2].primaryTide).toBe("Surge");
  });

  it("computes normalized preference weights", () => {
    const agents = [
      makeAgent([0, 0, 0, 0, 0, 0, 0], []),
      makeAgent([3, 0, 0, 0, 0, 0, 0], []),
    ];
    const state = makeDraftState(agents);
    const result = extractBotSummaries(state, new Map());

    const weights = result[0].preferenceWeights;
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
    const result = extractBotSummaries(state, db);

    expect(result[0].cardsByTide.Bloom).toBe(2);
    expect(result[0].cardsByTide.Arc).toBe(1);
  });
});
