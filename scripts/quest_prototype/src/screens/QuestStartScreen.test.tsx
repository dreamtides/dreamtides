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
import { structuralTidesForPackageTides } from "../data/structural-tides";
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
    } & HTMLAttributes<HTMLDivElement>) => (
      <div data-transition={JSON.stringify(_transition)} {...props}>
        {children}
      </div>
    ),
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
      whileHover,
      whileTap,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      initial?: unknown;
      transition?: unknown;
      whileHover?: unknown;
      whileTap?: unknown;
    } & HTMLAttributes<HTMLButtonElement>) => (
      <button
        data-transition={JSON.stringify(_transition)}
        data-while-hover={JSON.stringify(whileHover)}
        data-while-tap={JSON.stringify(whileTap)}
        {...props}
      >
        {children}
      </button>
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
    title: "Keeper of the Threshold Flame",
    awakening: 4,
    renderedText: "First dreamcaller.",
    imageNumber: "0009",
    mandatoryTides: ["materialize_value", "ally_formation", "support-a"],
    optionalTides: ["support-a", "support-b", "support-c"],
  },
  {
    id: "caller-2",
    name: "Vey of Embers",
    title: "The Ashen Cartographer",
    awakening: 6,
    renderedText: "Second dreamcaller.",
    imageNumber: "0010",
    mandatoryTides: ["warrior_pressure", "ally_wide", "support-d"],
    optionalTides: ["support-d", "support-e", "support-f"],
  },
  {
    id: "caller-3",
    name: "Noctis of Tides",
    title: "Harbinger of the Ninth Current",
    awakening: 5,
    renderedText: "Third dreamcaller.",
    imageNumber: "0011",
    mandatoryTides: ["void_recursion", "spark_tall", "support-g"],
    optionalTides: ["support-g", "support-h", "support-i"],
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
  it("shows exactly 3 Dreamcaller choices without legacy tide-step UI", () => {
    const { container, root } = mount(<QuestStartScreen />);
    const displayedStructuralTides = OFFERED_DREAMCALLERS.flatMap((dreamcaller) =>
      structuralTidesForPackageTides(dreamcaller.mandatoryTides),
    );

    expect(container.textContent).toContain("Mira of Lanterns");
    expect(container.textContent).toContain("Vey of Embers");
    expect(container.textContent).toContain("Noctis of Tides");
    expect(container.textContent).toContain("Choose Your Dreamcaller");
    expect(container.textContent).not.toContain("Structural Tides");
    expect(container.textContent).not.toContain("Bloom");
    expect(container.textContent).not.toContain("Ignite");
    expect(container.textContent).not.toContain("DreamcallerDraft");
    expect(container.textContent).not.toContain("Choose Your Tide");
    expect(container.querySelector('img[alt="Bloom"]')).toBeNull();
    expect(container.querySelector('img[alt="Ignite"]')).toBeNull();
    expect(container.querySelectorAll("button")).toHaveLength(3);
    expect(
      container.querySelectorAll("[data-structural-tide-chip]"),
    ).toHaveLength(displayedStructuralTides.length);

    for (const tide of displayedStructuralTides) {
      expect(container.textContent).toContain(tide.displayName);
      expect(container.textContent).toContain(tide.hoverBlurb);
      const chip = container.querySelector(
        `[data-structural-tide-chip="${tide.id}"]`,
      );
      expect(chip?.getAttribute("title")).toBe(tide.hoverBlurb);
      const icon = container.querySelector(
        `[data-structural-tide-icon="${tide.id}"]`,
      );
      expect(icon?.className).toContain("bx");
      expect(icon?.className).toContain(tide.iconClass);
    }

    expect(container.textContent).not.toContain("support-a");
    expect(container.textContent).not.toContain("support-d");
    expect(container.textContent).not.toContain("support-g");

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

  it("applies instant hover and tap transitions even with staggered entrance animation", () => {
    const { container, root } = mount(<QuestStartScreen />);

    const firstDreamcallerButton = container.querySelector("button");
    if (!firstDreamcallerButton) {
      throw new Error("Missing dreamcaller selection button");
    }
    const firstDreamcallerWrapper = firstDreamcallerButton.parentElement;
    if (!firstDreamcallerWrapper) {
      throw new Error("Missing dreamcaller wrapper");
    }

    const transition = JSON.parse(
      firstDreamcallerWrapper.getAttribute("data-transition") ?? "null",
    ) as { delay?: number } | null;
    const whileHover = JSON.parse(
      firstDreamcallerButton.getAttribute("data-while-hover") ?? "null",
    ) as { transition?: { delay?: number; duration?: number } } | null;
    const whileTap = JSON.parse(
      firstDreamcallerButton.getAttribute("data-while-tap") ?? "null",
    ) as { transition?: { delay?: number; duration?: number } } | null;

    expect(transition?.delay).toBeGreaterThan(0);
    expect(whileHover?.transition).toEqual({ delay: 0, duration: 0.12 });
    expect(whileTap?.transition).toEqual({ delay: 0, duration: 0.08 });

    act(() => {
      root.unmount();
    });
  });
});
