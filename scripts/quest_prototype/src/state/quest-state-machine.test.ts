import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { QuestContent } from "../data/quest-content";
import { toQuestDreamcaller } from "../data/dreamcaller-selection";
import type { ResolvedDreamcallerPackage } from "../types/content";
import type { DraftState } from "../types/draft";
import type {
  CardSourceDebugState,
  Dreamcaller,
  QuestState,
} from "../types/quest";
import {
  applyCardSourceDebug,
  QuestProvider,
  applyDraftState,
  applyDreamcallerSelection,
  applyRemainingDreamsignPool,
  createDefaultState,
  useQuest,
  type QuestContextValue,
} from "./quest-context";

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
    createElement(QuestProvider, {
      cardDatabase: new Map(),
      questContent: makeQuestContent(),
      children: createElement(Capture),
    }),
  );

  if (captured === null) {
    throw new Error("Failed to capture quest context");
  }

  return captured;
}

function makeDreamcaller(): Dreamcaller {
  return {
    id: "dreamcaller-1",
    name: "Test Dreamcaller",
    title: "State Witness",
    awakening: 4,
    renderedText: "Test ability.",
    imageNumber: "0006",
    accentTide: "Bloom",
  };
}

function makeResolvedPackage(): ResolvedDreamcallerPackage {
  return {
    dreamcaller: {
      id: "dreamcaller-1",
      name: "Test Dreamcaller",
      title: "State Witness",
      awakening: 4,
      renderedText: "Test rules text.",
      imageNumber: "0006",
      mandatoryTides: ["core"],
      optionalTides: ["support-a", "support-b", "support-c", "support-d"],
    },
    mandatoryTides: ["core"],
    optionalSubset: ["support-a", "support-b", "support-c"],
    selectedTides: ["core", "support-a", "support-b", "support-c"],
    draftPoolCopiesByCard: {
      "101": 2,
      "202": 1,
    },
    dreamsignPoolIds: ["embers-whisper", "glacial-insight", "ashbloom-mantle"],
    mandatoryOnlyPoolSize: 120,
    draftPoolSize: 210,
    doubledCardCount: 1,
    legalSubsetCount: 2,
    preferredSubsetCount: 1,
  };
}

function makeDraftState(): DraftState {
  return {
    remainingCopiesByCard: {
      "101": 1,
      "202": 2,
    },
    currentOffer: [101, 202, 303, 404],
    activeSiteId: "site-1",
    pickNumber: 3,
    sitePicksCompleted: 2,
  };
}

function makeCardSourceDebugState(): CardSourceDebugState {
  return {
    screenLabel: "Draft Picks",
    surface: "Draft",
    entries: [
      {
        cardNumber: 101,
        cardName: "Lantern Witness",
        cardTides: ["core", "support-a"],
        matchedMandatoryTides: ["core"],
        matchedOptionalTides: ["support-a"],
        isFallback: false,
      },
    ],
  };
}

beforeEach(() => {
  vi.spyOn(console, "log").mockImplementation(() => {});
});

describe("QuestProvider default state contract", () => {
  it("exposes package-driven default state without legacy tide fields", () => {
    const state = createDefaultState();

    expect(state.resolvedPackage).toBeNull();
    expect(state.cardSourceDebug).toBeNull();
    expect(state.remainingDreamsignPool).toEqual([]);
    expect(state.draftState).toBeNull();
    expect("tideCrystals" in (state as unknown as Record<string, unknown>)).toBe(false);
    expect("chosenTide" in (state as unknown as Record<string, unknown>)).toBe(false);
    expect("excludedTides" in (state as unknown as Record<string, unknown>)).toBe(false);
  });

  it("omits legacy mutators and exposes explicit pool mutators", () => {
    const mutationNames = Object.keys(captureQuestContext().mutations);

    expect(mutationNames).toContain("setDreamcallerSelection");
    expect(mutationNames).toContain("setCardSourceDebug");
    expect(mutationNames).toContain("setRemainingDreamsignPool");
    expect(mutationNames).toContain("setDraftState");
    expect(mutationNames).not.toContain("setDreamcaller");
    expect(mutationNames).not.toContain("addTideCrystal");
    expect(mutationNames).not.toContain("setChosenTide");
    expect(mutationNames).not.toContain("setExcludedTides");
  });
});

describe("Task 02 state transitions", () => {
  it("stores the resolved package and initializes the shared Dreamsign pool from it", () => {
    const resolvedPackage = makeResolvedPackage();
    const next = applyDreamcallerSelection(
      createDefaultState(),
      resolvedPackage,
    );

    expect(next.dreamcaller).toEqual(toQuestDreamcaller(resolvedPackage.dreamcaller));
    expect(next.resolvedPackage).toEqual(resolvedPackage);
    expect(next.remainingDreamsignPool).toEqual(resolvedPackage.dreamsignPoolIds);

    resolvedPackage.dreamsignPoolIds.push("verdant-accord");
    expect(next.remainingDreamsignPool).toEqual([
      "embers-whisper",
      "glacial-insight",
      "ashbloom-mantle",
    ]);
  });

  it("updates the remaining Dreamsign pool without mutating prior state", () => {
    const initial = applyDreamcallerSelection(
      createDefaultState(),
      makeResolvedPackage(),
    );
    const nextPool = ["glacial-insight"];
    const next = applyRemainingDreamsignPool(initial, nextPool);

    expect(initial.remainingDreamsignPool).toEqual([
      "embers-whisper",
      "glacial-insight",
      "ashbloom-mantle",
    ]);
    expect(next.remainingDreamsignPool).toEqual(["glacial-insight"]);

    nextPool.push("ashbloom-mantle");
    expect(next.remainingDreamsignPool).toEqual(["glacial-insight"]);
  });

  it("persists fixed-pool draft progress, including drafted cards for site completion", () => {
    const next = applyDraftState(createDefaultState(), makeDraftState());

    expect(next.draftState).toEqual({
      remainingCopiesByCard: {
        "101": 1,
        "202": 2,
      },
      currentOffer: [101, 202, 303, 404],
      activeSiteId: "site-1",
      pickNumber: 3,
      sitePicksCompleted: 2,
    });
    expect("seenCards" in (next.draftState as unknown as Record<string, unknown>)).toBe(false);
  });

  it("stores and clears card source debug data without mutating prior state", () => {
    const initial = createDefaultState();
    const overlay = makeCardSourceDebugState();
    const next = applyCardSourceDebug(initial, overlay);

    expect(initial.cardSourceDebug).toBeNull();
    expect(next.cardSourceDebug).toEqual(overlay);

    overlay.entries.push({
      cardNumber: 202,
      cardName: "Late Mutation",
      cardTides: ["support-b"],
      matchedMandatoryTides: [],
      matchedOptionalTides: ["support-b"],
      isFallback: false,
    });

    expect(next.cardSourceDebug?.entries).toHaveLength(1);
    expect(applyCardSourceDebug(next, null).cardSourceDebug).toBeNull();
  });

  it("resets package-driven run state back to an empty quest shell", () => {
    const populated: QuestState = {
      ...createDefaultState(),
      dreamcaller: makeDreamcaller(),
      resolvedPackage: makeResolvedPackage(),
      cardSourceDebug: makeCardSourceDebugState(),
      remainingDreamsignPool: ["embers-whisper"],
      draftState: makeDraftState(),
      visitedSites: ["site-1"],
      currentDreamscape: "dreamscape-1",
    };

    const reset = createDefaultState();

    expect(reset).toEqual({
      ...createDefaultState(),
      essence: 250,
      deck: [],
      dreamcaller: null,
      resolvedPackage: null,
      cardSourceDebug: null,
      remainingDreamsignPool: [],
      dreamsigns: [],
      completionLevel: 0,
      atlas: { nodes: {}, edges: [], nexusId: "" },
      currentDreamscape: null,
      visitedSites: [],
      draftState: null,
      screen: { type: "questStart" },
      activeSiteId: null,
    });
    expect(populated.resolvedPackage).not.toBeNull();
  });
});
