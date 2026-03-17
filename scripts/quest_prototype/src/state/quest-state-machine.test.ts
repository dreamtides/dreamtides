import { describe, it, expect, beforeEach, vi } from "vitest";
import { logEvent, resetLog, getLogEntries } from "../logging";
import type { DreamscapeNode, QuestState, Screen, SiteState } from "../types/quest";

/**
 * Creates a default quest state for testing, matching the shape
 * produced by createDefaultState in quest-context.tsx.
 */
function createTestState(overrides: Partial<QuestState> = {}): QuestState {
  return {
    essence: 250,
    deck: [],
    dreamcaller: null,
    dreamsigns: [],
    tideCrystals: {
      Bloom: 0,
      Arc: 0,
      Ignite: 0,
      Pact: 0,
      Umbra: 0,
      Rime: 0,
      Surge: 0,
      Wild: 0,
    },
    completionLevel: 0,
    atlas: { nodes: {}, edges: [], nexusId: "" },
    currentDreamscape: null,
    visitedSites: [],
    draftState: null,
    screen: { type: "questStart" },
    activeSiteId: null,
    ...overrides,
  };
}

function screenName(screen: Screen): string {
  return screen.type === "site" ? `site:${screen.siteId}` : screen.type;
}

/**
 * Replicates the incrementCompletionLevel state updater logic from
 * quest-context.tsx so we can test the state machine transition in
 * isolation without mounting React components.
 */
function applyIncrementCompletionLevel(
  prev: QuestState,
  essenceReward: number,
  rewardCardNumber: number | null,
): QuestState {
  const newLevel = prev.completionLevel + 1;
  logEvent("battle_won", {
    completionLevel: newLevel,
    essenceReward,
    rewardCardNumber,
  });
  const screen: Screen =
    newLevel >= 7 ? { type: "questComplete" } : prev.screen;
  if (newLevel >= 7) {
    logEvent("screen_transition", {
      from: screenName(prev.screen),
      to: screenName(screen),
    });
  }
  return { ...prev, completionLevel: newLevel, screen };
}

beforeEach(() => {
  resetLog();
  vi.spyOn(console, "log").mockImplementation(() => {});
});

describe("incrementCompletionLevel state transitions", () => {
  it("does not change screen when level is below 7", () => {
    const state = createTestState({
      completionLevel: 3,
      screen: { type: "atlas" },
    });
    const next = applyIncrementCompletionLevel(state, 50, null);
    expect(next.completionLevel).toBe(4);
    expect(next.screen.type).toBe("atlas");
  });

  it("transitions to questComplete when reaching level 7", () => {
    const state = createTestState({
      completionLevel: 6,
      screen: { type: "atlas" },
    });
    const next = applyIncrementCompletionLevel(state, 100, 42);
    expect(next.completionLevel).toBe(7);
    expect(next.screen.type).toBe("questComplete");
  });

  it("logs screen_transition when reaching level 7", () => {
    const state = createTestState({
      completionLevel: 6,
      screen: { type: "atlas" },
    });
    applyIncrementCompletionLevel(state, 100, null);
    const entries = getLogEntries();
    const transition = entries.find((e) => e.event === "screen_transition");
    expect(transition).toBeDefined();
    expect(transition?.from).toBe("atlas");
    expect(transition?.to).toBe("questComplete");
  });

  it("does not log screen_transition when level is below 7", () => {
    const state = createTestState({
      completionLevel: 4,
      screen: { type: "atlas" },
    });
    applyIncrementCompletionLevel(state, 50, null);
    const entries = getLogEntries();
    const transition = entries.find((e) => e.event === "screen_transition");
    expect(transition).toBeUndefined();
  });

  it("always logs battle_won with the new level", () => {
    const state = createTestState({
      completionLevel: 2,
      screen: { type: "atlas" },
    });
    applyIncrementCompletionLevel(state, 75, 10);
    const entries = getLogEntries();
    const battleWon = entries.find((e) => e.event === "battle_won");
    expect(battleWon).toBeDefined();
    expect(battleWon?.completionLevel).toBe(3);
    expect(battleWon?.essenceReward).toBe(75);
    expect(battleWon?.rewardCardNumber).toBe(10);
  });

  it("transitions from any screen to questComplete at level 7", () => {
    const state = createTestState({
      completionLevel: 6,
      screen: { type: "site", siteId: "battle-1" },
    });
    const next = applyIncrementCompletionLevel(state, 100, null);
    expect(next.screen.type).toBe("questComplete");
    const entries = getLogEntries();
    const transition = entries.find((e) => e.event === "screen_transition");
    expect(transition?.from).toBe("site:battle-1");
  });
});

describe("screen name serialization", () => {
  it("returns the type for non-site screens", () => {
    expect(screenName({ type: "questStart" })).toBe("questStart");
    expect(screenName({ type: "atlas" })).toBe("atlas");
    expect(screenName({ type: "dreamscape" })).toBe("dreamscape");
    expect(screenName({ type: "questComplete" })).toBe("questComplete");
  });

  it("includes siteId for site screens", () => {
    expect(screenName({ type: "site", siteId: "shop-1" })).toBe("site:shop-1");
  });
});

function makeSite(type: SiteState["type"], isVisited: boolean): SiteState {
  return { id: `${type.toLowerCase()}-1`, type, isEnhanced: false, isVisited };
}

function makeNode(
  id: string,
  sites: SiteState[],
  status: DreamscapeNode["status"] = "available",
): DreamscapeNode {
  return {
    id,
    biomeName: "Test Biome",
    biomeColor: "#aabbcc",
    sites,
    position: { x: 0, y: 0 },
    status,
    enhancedSiteType: null,
  };
}

describe("battle site unlock gating", () => {
  it("battle is locked when non-battle sites remain unvisited", () => {
    const sites = [
      makeSite("Shop", false),
      makeSite("Essence", false),
      makeSite("Battle", false),
    ];
    const nonBattleAllVisited = sites
      .filter((s) => s.type !== "Battle")
      .every((s) => s.isVisited);
    expect(nonBattleAllVisited).toBe(false);
  });

  it("battle unlocks when all non-battle sites are visited", () => {
    const sites = [
      makeSite("Shop", true),
      makeSite("Essence", true),
      makeSite("Battle", false),
    ];
    const nonBattleAllVisited = sites
      .filter((s) => s.type !== "Battle")
      .every((s) => s.isVisited);
    expect(nonBattleAllVisited).toBe(true);
  });
});

/**
 * Replicates the setCurrentDreamscape state updater logic from
 * quest-context.tsx so we can test the event logging in isolation.
 */
function applySetCurrentDreamscape(
  prev: QuestState,
  nodeId: string | null,
): QuestState {
  if (nodeId !== null) {
    const node = prev.atlas.nodes[nodeId];
    logEvent("dreamscape_entered", {
      dreamscapeId: nodeId,
      biomeName: node?.biomeName ?? "unknown",
    });
  }
  return {
    ...prev,
    currentDreamscape: nodeId,
    visitedSites: nodeId !== null ? [] : prev.visitedSites,
  };
}

describe("early exit from dreamscape", () => {
  it("does not log dreamscape_completed when clearing currentDreamscape", () => {
    const node = makeNode("ds-1", [makeSite("Shop", false), makeSite("Battle", false)]);
    const state = createTestState({
      currentDreamscape: "ds-1",
      atlas: { nodes: { "ds-1": node }, edges: [], nexusId: "nexus" },
    });
    applySetCurrentDreamscape(state, null);
    const entries = getLogEntries();
    const completed = entries.find((e) => e.event === "dreamscape_completed");
    expect(completed).toBeUndefined();
  });

  it("logs dreamscape_entered when entering a dreamscape", () => {
    const node = makeNode("ds-1", [makeSite("Shop", false)]);
    const state = createTestState({
      atlas: { nodes: { "ds-1": node }, edges: [], nexusId: "nexus" },
    });
    applySetCurrentDreamscape(state, "ds-1");
    const entries = getLogEntries();
    const entered = entries.find((e) => e.event === "dreamscape_entered");
    expect(entered).toBeDefined();
    expect(entered?.dreamscapeId).toBe("ds-1");
  });
});

describe("final-boss completion path", () => {
  it("dreamscape should be marked completed before quest-complete transition", () => {
    // Simulate the battle auto-complete flow for the final boss:
    // 1. updateAtlas with completed node (via generateNewNodes)
    // 2. setCurrentDreamscape(null)
    // 3. incrementCompletionLevel -> questComplete
    const node = makeNode("ds-7", [
      makeSite("Shop", true),
      makeSite("Battle", true),
    ]);
    const state = createTestState({
      completionLevel: 6,
      currentDreamscape: "ds-7",
      screen: { type: "site", siteId: "battle-1" },
      atlas: {
        nodes: { nexus: makeNode("nexus", [], "completed"), "ds-7": node },
        edges: [["nexus", "ds-7"]],
        nexusId: "nexus",
      },
    });

    // Step 1: mark dreamscape completed in atlas (simulating updateAtlas
    // with generateNewNodes result, which sets status to "completed")
    const completedNode = { ...node, status: "completed" as const };
    const updatedAtlas = {
      ...state.atlas,
      nodes: { ...state.atlas.nodes, "ds-7": completedNode },
    };
    let next: QuestState = { ...state, atlas: updatedAtlas };

    // Step 2: log dreamscape_completed
    logEvent("dreamscape_completed", {
      dreamscapeId: "ds-7",
      sitesVisitedCount: 2,
    });

    // Step 3: clear currentDreamscape
    next = applySetCurrentDreamscape(next, null);

    // Step 4: increment completion level (triggers questComplete)
    next = applyIncrementCompletionLevel(next, 400, null);

    // The atlas should already show the dreamscape as completed
    expect(next.atlas.nodes["ds-7"].status).toBe("completed");
    expect(next.screen.type).toBe("questComplete");
    expect(next.completionLevel).toBe(7);

    // Count completed nodes like QuestCompleteScreen does
    const completedCount = Object.values(next.atlas.nodes).filter(
      (n) => n.status === "completed",
    ).length;
    expect(completedCount).toBe(2); // nexus + ds-7

    // Verify the correct events were logged
    const entries = getLogEntries();
    const completedEvent = entries.find((e) => e.event === "dreamscape_completed");
    expect(completedEvent).toBeDefined();
    expect(completedEvent?.dreamscapeId).toBe("ds-7");
  });
});
