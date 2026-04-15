// @vitest-environment jsdom

import { act } from "react";
import type { HTMLAttributes, ReactElement, ReactNode } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { QuestMutations } from "../state/quest-context";
import type { CardData } from "../types/cards";
import type { DreamcallerContent } from "../types/content";
import type { QuestState } from "../types/quest";
import { QuestStartScreen } from "./QuestStartScreen";
import { useQuest } from "../state/quest-context";
import { selectDreamcallerOffer } from "../data/dreamcaller-selection";
import { bootstrapQuestStart } from "./quest-start-bootstrap";

vi.mock("framer-motion", () => ({
  motion: {
    div: ({
      animate: _animate,
      children,
      initial: _initial,
      transition: _transition,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      initial?: unknown;
      transition?: unknown;
    } & HTMLAttributes<HTMLDivElement>) => <div {...props}>{children}</div>,
    h1: ({
      animate: _animate,
      children,
      initial: _initial,
      transition: _transition,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      initial?: unknown;
      transition?: unknown;
    } & HTMLAttributes<HTMLHeadingElement>) => <h1 {...props}>{children}</h1>,
    p: ({
      animate: _animate,
      children,
      initial: _initial,
      transition: _transition,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      initial?: unknown;
      transition?: unknown;
    } & HTMLAttributes<HTMLParagraphElement>) => <p {...props}>{children}</p>,
    button: ({
      animate: _animate,
      children,
      initial: _initial,
      transition: _transition,
      whileHover: _whileHover,
      whileTap: _whileTap,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      initial?: unknown;
      transition?: unknown;
      whileHover?: unknown;
      whileTap?: unknown;
    } & HTMLAttributes<HTMLButtonElement>) => (
      <button {...props}>{children}</button>
    ),
  },
}));

vi.mock("../state/quest-context", () => ({
  useQuest: vi.fn(),
}));

vi.mock("../data/dreamcaller-selection", () => ({
  selectDreamcallerOffer: vi.fn(),
}));

vi.mock("./quest-start-bootstrap", () => ({
  bootstrapQuestStart: vi.fn(),
}));

const OFFERED_DREAMCALLERS: readonly DreamcallerContent[] = [
  {
    id: "caller-1",
    name: "Mira of Lanterns",
    awakening: 4,
    renderedText: "First dreamcaller.",
    mandatoryTides: ["core"],
    optionalTides: ["support-a", "support-b", "support-c"],
  },
  {
    id: "caller-2",
    name: "Vey of Embers",
    awakening: 6,
    renderedText: "Second dreamcaller.",
    mandatoryTides: ["core"],
    optionalTides: ["support-d", "support-e", "support-f"],
  },
] as const;

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

function makeState(): QuestState {
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
  };
}

function setQuestContext(): void {
  vi.mocked(useQuest).mockReturnValue({
    state: makeState(),
    mutations: makeMutations(),
    cardDatabase: new Map<number, CardData>(),
    questContent: {
      cardDatabase: new Map(),
      cardsByPackageTide: new Map(),
      dreamcallers: [...OFFERED_DREAMCALLERS],
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
  setQuestContext();
  vi.mocked(selectDreamcallerOffer).mockReturnValue([...OFFERED_DREAMCALLERS]);
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("QuestStartScreen", () => {
  it("keeps dreamcaller selection cards free of tide labels and icons", () => {
    const { container, root } = mount(<QuestStartScreen />);

    expect(container.textContent).toContain("Mira of Lanterns");
    expect(container.textContent).toContain("Vey of Embers");
    expect(container.textContent).not.toContain("Bloom");
    expect(container.textContent).not.toContain("Ignite");
    expect(container.querySelector('img[alt="Bloom"]')).toBeNull();
    expect(container.querySelector('img[alt="Ignite"]')).toBeNull();

    const secondDreamcallerButton = Array.from(
      container.querySelectorAll("button"),
    ).find((candidate) => candidate.textContent?.includes("Vey of Embers"));
    if (!secondDreamcallerButton) {
      throw new Error("Missing dreamcaller selection button");
    }

    act(() => {
      secondDreamcallerButton.dispatchEvent(
        new MouseEvent("click", { bubbles: true }),
      );
    });

    expect(bootstrapQuestStart).toHaveBeenCalledWith(
      expect.objectContaining({
        dreamcaller: OFFERED_DREAMCALLERS[1],
      }),
    );

    act(() => {
      root.unmount();
    });
  });
});
