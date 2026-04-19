// @vitest-environment jsdom

import { act } from "react";
import type { ReactElement, ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CardData } from "./types/cards";
import type { QuestMutations } from "./state/quest-context";
import type { QuestState } from "./types/quest";
import { QuestApp } from "./App";
import { useQuest } from "./state/quest-context";

vi.mock("./state/quest-context", () => ({
  useQuest: vi.fn(),
}));

vi.mock("./components/ScreenRouter", () => ({
  ScreenRouter: () => <div>Screen Router</div>,
}));

vi.mock("./components/HUD", () => ({
  HUD: () => <div data-testid="hud">HUD</div>,
}));

const deckViewerMock = vi.fn<
  (props: { introMode?: boolean; isOpen: boolean }) => ReactNode
>(({ isOpen, introMode }) => (
  <div
    data-deck-intro={String(Boolean(introMode))}
    data-deck-open={String(isOpen)}
  >
    Deck Viewer
  </div>
));

vi.mock("./components/DeckViewer", () => ({
  DeckViewer: (props: { isOpen: boolean }) => deckViewerMock(props),
}));

vi.mock("./screens/DebugScreen", () => ({
  DebugScreen: () => <div>Debug Screen</div>,
}));

vi.mock("./screens/CardSourceOverlay", () => ({
  CardSourceOverlay: () => <div>Card Source Overlay</div>,
}));

function makeMutations(): QuestMutations {
  return {
    changeEssence: vi.fn(),
    addCard: vi.fn(),
    addBaneCard: vi.fn(),
    removeCard: vi.fn(),
    transfigureCard: vi.fn(),
    setDreamcallerSelection: vi.fn(),
    setCardSourceDebug: vi.fn(),
    addDreamsign: vi.fn(),
    removeDreamsign: vi.fn(),
    setRemainingDreamsignPool: vi.fn(),
    incrementCompletionLevel: vi.fn(),
    setScreen: vi.fn(),
    markSiteVisited: vi.fn(),
    setCurrentDreamscape: vi.fn(),
    updateAtlas: vi.fn(),
    setDraftState: vi.fn(),
    setFailureSummary: vi.fn(),
    resetQuest: vi.fn(),
  };
}

function makeState(overrides: Partial<QuestState> = {}): QuestState {
  return {
    essence: 250,
    deck: [],
    dreamcaller: null,
    resolvedPackage: null,
    cardSourceDebug: null,
    remainingDreamsignPool: [],
    dreamsigns: [],
    completionLevel: 0,
    atlas: {
      nodes: {},
      edges: [],
      nexusId: "",
    },
    currentDreamscape: null,
    visitedSites: [],
    draftState: null,
    screen: { type: "questStart" },
    activeSiteId: null,
    failureSummary: null,
    ...overrides,
  };
}

function setQuestState(state: QuestState): void {
  vi.mocked(useQuest).mockReturnValue({
    state,
    mutations: makeMutations(),
    cardDatabase: new Map<number, CardData>(),
    questContent: {
      cardDatabase: new Map(),
      cardsByPackageTide: new Map(),
      dreamcallers: [],
      dreamsignTemplates: [],
      resolvedPackagesByDreamcallerId: new Map(),
    },
  });
}

function mount(element: ReactElement): {
  container: HTMLDivElement;
  root: Root;
} {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  act(() => {
    root.render(element);
  });
  return { container, root };
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response(null));
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("QuestApp", () => {
  it("does not auto-open the deck viewer after leaving quest start (FIND-01-6)", () => {
    setQuestState(makeState());

    const { root } = mount(
      <QuestApp
        cardDatabase={new Map()}
        runtimeConfig={{ battleMode: "auto", seedOverride: null, startInBattle: false }}
      />,
    );

    expect(deckViewerMock).toHaveBeenLastCalledWith(
      expect.objectContaining({ isOpen: false }),
    );

    setQuestState(
      makeState({
        deck: Array.from({ length: 10 }, (_, index) => ({
          entryId: `deck-${String(index + 1)}`,
          cardNumber: 711 + index,
          transfiguration: null,
          isBane: false,
        })),
        dreamcaller: {
          id: "caller-1",
          name: "Starter Caller",
          title: "Of the First Hand",
          awakening: 4,
          renderedText: "Pick your path.",
          imageNumber: "0004",
          accentTide: "Bloom",
        },
        screen: { type: "dreamscape" },
      }),
    );

    act(() => {
      root.render(
        <QuestApp
          cardDatabase={new Map()}
          runtimeConfig={{ battleMode: "auto", seedOverride: null, startInBattle: false }}
        />,
      );
    });

    // FIND-01-6 (Stage 4): player lands on the site screen unobstructed.
    // The deck viewer stays closed; View Deck on the HUD opens it on demand.
    expect(deckViewerMock).toHaveBeenLastCalledWith(
      expect.objectContaining({ isOpen: false }),
    );

    act(() => {
      root.unmount();
    });
  });

  it("invokes the playable-battle bootstrap when startInBattle is set on playable mode", () => {
    const mutations = makeMutations();
    const dreamcaller = {
      id: "caller-1",
      name: "Test Caller",
      title: "Of Tests",
      awakening: 4,
      renderedText: "Pick.",
      imageNumber: "0001",
      mandatoryTides: ["Bloom"],
      optionalTides: ["Arc", "Ignite", "Pact", "Rime"],
    };
    const resolvedPackage = {
      dreamcaller,
      mandatoryTides: ["Bloom"],
      optionalSubset: ["Arc", "Ignite", "Pact"],
      selectedTides: ["Bloom", "Arc", "Ignite", "Pact"],
      draftPoolCopiesByCard: { "101": 2 },
      dreamsignPoolIds: ["dreamsign-1"],
      mandatoryOnlyPoolSize: 12,
      draftPoolSize: 24,
      doubledCardCount: 1,
      legalSubsetCount: 4,
      preferredSubsetCount: 2,
    };
    vi.mocked(useQuest).mockReturnValue({
      state: makeState(),
      mutations,
      cardDatabase: new Map<number, CardData>(),
      questContent: {
        cardDatabase: new Map(),
        cardsByPackageTide: new Map(),
        dreamcallers: [dreamcaller],
        dreamsignTemplates: [
          {
            id: "dreamsign-1",
            name: "Bloom Echo",
            effectDescription: "Test.",
            displayTide: "Bloom",
            packageTides: ["Bloom"],
          },
        ],
        resolvedPackagesByDreamcallerId: new Map([
          [dreamcaller.id, resolvedPackage],
        ]),
      },
    });

    const { root } = mount(
      <QuestApp
        cardDatabase={new Map()}
        runtimeConfig={{
          battleMode: "playable",
          seedOverride: null,
          startInBattle: true,
        }}
      />,
    );

    expect(mutations.setDreamcallerSelection).toHaveBeenCalledOnce();

    act(() => {
      root.unmount();
    });
  });

  it("does not invoke the playable-battle bootstrap when startInBattle is false", () => {
    const mutations = makeMutations();
    vi.mocked(useQuest).mockReturnValue({
      state: makeState(),
      mutations,
      cardDatabase: new Map<number, CardData>(),
      questContent: {
        cardDatabase: new Map(),
        cardsByPackageTide: new Map(),
        dreamcallers: [],
        dreamsignTemplates: [],
        resolvedPackagesByDreamcallerId: new Map(),
      },
    });

    const { root } = mount(
      <QuestApp
        cardDatabase={new Map()}
        runtimeConfig={{
          battleMode: "playable",
          seedOverride: null,
          startInBattle: false,
        }}
      />,
    );

    expect(mutations.setDreamcallerSelection).not.toHaveBeenCalled();
    expect(mutations.setScreen).not.toHaveBeenCalled();

    act(() => {
      root.unmount();
    });
  });

  it("does not invoke the playable-battle bootstrap in auto mode regardless of startInBattle", () => {
    const mutations = makeMutations();
    const dreamcaller = {
      id: "caller-1",
      name: "Test Caller",
      title: "Of Tests",
      awakening: 4,
      renderedText: "Pick.",
      imageNumber: "0001",
      mandatoryTides: ["Bloom"],
      optionalTides: ["Arc", "Ignite", "Pact", "Rime"],
    };
    const resolvedPackage = {
      dreamcaller,
      mandatoryTides: ["Bloom"],
      optionalSubset: ["Arc", "Ignite", "Pact"],
      selectedTides: ["Bloom", "Arc", "Ignite", "Pact"],
      draftPoolCopiesByCard: { "101": 2 },
      dreamsignPoolIds: ["dreamsign-1"],
      mandatoryOnlyPoolSize: 12,
      draftPoolSize: 24,
      doubledCardCount: 1,
      legalSubsetCount: 4,
      preferredSubsetCount: 2,
    };
    vi.mocked(useQuest).mockReturnValue({
      state: makeState(),
      mutations,
      cardDatabase: new Map<number, CardData>(),
      questContent: {
        cardDatabase: new Map(),
        cardsByPackageTide: new Map(),
        dreamcallers: [dreamcaller],
        dreamsignTemplates: [
          {
            id: "dreamsign-1",
            name: "Bloom Echo",
            effectDescription: "Test.",
            displayTide: "Bloom",
            packageTides: ["Bloom"],
          },
        ],
        resolvedPackagesByDreamcallerId: new Map([
          [dreamcaller.id, resolvedPackage],
        ]),
      },
    });

    const { root } = mount(
      <QuestApp
        cardDatabase={new Map()}
        runtimeConfig={{
          battleMode: "auto",
          seedOverride: null,
          startInBattle: true,
        }}
      />,
    );

    expect(mutations.setDreamcallerSelection).not.toHaveBeenCalled();
    expect(mutations.setScreen).not.toHaveBeenCalled();

    act(() => {
      root.unmount();
    });
  });

  it("hides the shared HUD on playable battle sites so the battle dock stays usable", () => {
    setQuestState(
      makeState({
        atlas: {
          nodes: {
            "dreamscape-1": {
              id: "dreamscape-1",
              biomeName: "Test Dreamscape",
              biomeColor: "#112233",
              sites: [
                {
                  id: "site-1",
                  type: "Battle",
                  isEnhanced: false,
                  isVisited: false,
                },
              ],
              position: { x: 0, y: 0 },
              status: "available",
              enhancedSiteType: null,
            },
          },
          edges: [],
          nexusId: "dreamscape-1",
        },
        currentDreamscape: "dreamscape-1",
        screen: { type: "site", siteId: "site-1" },
        activeSiteId: "site-1",
      }),
    );

    const { container, root } = mount(
      <QuestApp
        cardDatabase={new Map()}
        runtimeConfig={{ battleMode: "playable", seedOverride: null, startInBattle: false }}
      />,
    );

    expect(container.querySelector('[data-testid="hud"]')).toBeNull();

    act(() => {
      root.unmount();
    });
  });
});
