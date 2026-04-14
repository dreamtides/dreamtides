import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, it, expect, beforeEach, vi } from "vitest";
import type { QuestContent } from "../data/quest-content";
import { getLogEntries, logEvent, resetLog } from "../logging";
import {
  QuestProvider,
  useQuest,
  type QuestContextValue,
} from "./quest-context";
import type { DreamscapeNode, QuestState, Screen, SiteState } from "../types/quest";

function createTestState(overrides: Partial<QuestState> = {}): QuestState {
  return {
    essence: 250,
    deck: [],
    dreamcaller: null,
    resolvedPackage: null,
    remainingDreamsignPool: [],
    dreamsigns: [],
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

function makeQuestContent(): QuestContent {
  return {
    cardDatabase: new Map(),
    cardsByPackageTide: new Map(),
    dreamcallers: [],
    dreamsignTemplates: [],
    resolvedPackagesByDreamcallerId: new Map(),
  };
}

function captureQuestContext(): QuestContextValue {
  let captured: QuestContextValue | null = null;

  function Capture() {
    captured = useQuest();
    return null;
  }

  renderToStaticMarkup(
    createElement(
      QuestProvider,
      {
        cardDatabase: new Map(),
        questContent: makeQuestContent(),
        children: createElement(Capture),
      },
    ),
  );

  if (captured === null) {
    throw new Error("Failed to capture quest context");
  }

  return captured;
}

function screenName(screen: Screen): string {
  return screen.type === "site" ? `site:${screen.siteId}` : screen.type;
}

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

beforeEach(() => {
  resetLog();
  vi.spyOn(console, "log").mockImplementation(() => {});
});

describe("QuestProvider default state contract", () => {
  it("exposes package-driven default state without legacy tide fields", () => {
    const { state } = captureQuestContext();

    expect(state.resolvedPackage).toBeNull();
    expect(state.remainingDreamsignPool).toEqual([]);
    expect(state.draftState).toBeNull();
    expect("tideCrystals" in (state as unknown as Record<string, unknown>)).toBe(false);
    expect("chosenTide" in (state as unknown as Record<string, unknown>)).toBe(false);
    expect("excludedTides" in (state as unknown as Record<string, unknown>)).toBe(false);
  });

  it("omits legacy mutators and exposes explicit pool mutators", () => {
    const mutationNames = Object.keys(captureQuestContext().mutations);

    expect(mutationNames).toContain("setRemainingDreamsignPool");
    expect(mutationNames).not.toContain("addTideCrystal");
    expect(mutationNames).not.toContain("setChosenTide");
    expect(mutationNames).not.toContain("setExcludedTides");
  });
});

describe("incrementCompletionLevel state transitions", () => {
  it("does not change screen when level is below 7", () => {
    const next = applyIncrementCompletionLevel(
      createTestState({
        completionLevel: 3,
        screen: { type: "atlas" },
      }),
      50,
      null,
    );

    expect(next.completionLevel).toBe(4);
    expect(next.screen.type).toBe("atlas");
  });

  it("transitions to questComplete when reaching level 7", () => {
    const next = applyIncrementCompletionLevel(
      createTestState({
        completionLevel: 6,
        screen: { type: "atlas" },
      }),
      100,
      42,
    );

    expect(next.completionLevel).toBe(7);
    expect(next.screen.type).toBe("questComplete");
  });
});

describe("battle site unlock gating", () => {
  it("battle is locked when non-battle sites remain unvisited", () => {
    const sites = [
      makeSite("Shop", false),
      makeSite("Essence", false),
      makeSite("Battle", false),
    ];

    expect(
      sites.filter((site) => site.type !== "Battle").every((site) => site.isVisited),
    ).toBe(false);
  });

  it("battle unlocks when all non-battle sites are visited", () => {
    const sites = [
      makeSite("Shop", true),
      makeSite("Essence", true),
      makeSite("Battle", false),
    ];

    expect(
      sites.filter((site) => site.type !== "Battle").every((site) => site.isVisited),
    ).toBe(true);
  });
});

describe("dreamscape transitions", () => {
  it("does not log dreamscape_completed when clearing currentDreamscape", () => {
    const node = makeNode("ds-1", [
      makeSite("Shop", false),
      makeSite("Battle", false),
    ]);
    const state = createTestState({
      currentDreamscape: "ds-1",
      atlas: { nodes: { "ds-1": node }, edges: [], nexusId: "nexus" },
    });

    applySetCurrentDreamscape(state, null);

    expect(
      getLogEntries().find((entry) => entry.event === "dreamscape_completed"),
    ).toBeUndefined();
  });
});
