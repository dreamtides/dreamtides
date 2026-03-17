import { describe, it, expect, beforeEach, vi } from "vitest";
import { logEvent, resetLog, getLogEntries } from "../logging";
import type { QuestState, Screen } from "../types/quest";

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
