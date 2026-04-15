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
  HUD: () => <div>HUD</div>,
}));

const deckViewerMock = vi.fn<
  (props: { isOpen: boolean }) => ReactNode
>(({ isOpen }) => <div data-deck-open={String(isOpen)}>Deck Viewer</div>);

vi.mock("./components/DeckViewer", () => ({
  DeckViewer: (props: { isOpen: boolean }) => deckViewerMock(props),
}));

vi.mock("./screens/DebugScreen", () => ({
  DebugScreen: () => <div>Debug Screen</div>,
}));

function makeMutations(): QuestMutations {
  return {
    changeEssence: vi.fn(),
    addCard: vi.fn(),
    addBaneCard: vi.fn(),
    removeCard: vi.fn(),
    transfigureCard: vi.fn(),
    setDreamcallerSelection: vi.fn(),
    addDreamsign: vi.fn(),
    removeDreamsign: vi.fn(),
    setRemainingDreamsignPool: vi.fn(),
    incrementCompletionLevel: vi.fn(),
    setScreen: vi.fn(),
    markSiteVisited: vi.fn(),
    setCurrentDreamscape: vi.fn(),
    updateAtlas: vi.fn(),
    setDraftState: vi.fn(),
    resetQuest: vi.fn(),
  };
}

function makeState(overrides: Partial<QuestState> = {}): QuestState {
  return {
    essence: 250,
    deck: [],
    dreamcaller: null,
    resolvedPackage: null,
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
  it("automatically opens the deck viewer after leaving quest start with the starter deck", () => {
    setQuestState(makeState());

    const { root } = mount(<QuestApp cardDatabase={new Map()} />);

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
          awakening: 4,
          renderedText: "Pick your path.",
          accentTide: "Bloom",
        },
        screen: { type: "dreamscape" },
      }),
    );

    act(() => {
      root.render(<QuestApp cardDatabase={new Map()} />);
    });

    expect(deckViewerMock).toHaveBeenLastCalledWith(
      expect.objectContaining({ isOpen: true }),
    );

    act(() => {
      root.unmount();
    });
  });
});
