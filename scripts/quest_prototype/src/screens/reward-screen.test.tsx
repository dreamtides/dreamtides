// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { HTMLAttributes, ReactElement, ReactNode } from "react";
import type { QuestMutations } from "../state/quest-context";
import type { QuestState } from "../types/quest";
import type { CardData } from "../types/cards";
import type { DreamsignTemplate } from "../types/content";
import { RewardSiteScreen } from "./RewardSiteScreen";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";

vi.mock("framer-motion", () => ({
  motion: {
    div: ({
      animate: _animate,
      children,
      exit: _exit,
      initial: _initial,
      transition: _transition,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      exit?: unknown;
      initial?: unknown;
      transition?: unknown;
    } & HTMLAttributes<HTMLDivElement>) => <div {...props}>{children}</div>,
    button: ({
      animate: _animate,
      children,
      exit: _exit,
      initial: _initial,
      transition: _transition,
      whileHover: _whileHover,
      whileTap: _whileTap,
      ...props
    }: {
      animate?: unknown;
      children: ReactNode;
      exit?: unknown;
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

vi.mock("../logging", () => ({
  logEvent: vi.fn(),
}));

vi.mock("../components/CardDisplay", () => ({
  CardDisplay: ({ card }: { card: CardData }) => <div>{card.name}</div>,
}));

vi.mock("../data/card-database", () => ({
  TIDE_COLORS: {
    Bloom: "#34d399",
    Arc: "#a78bfa",
  },
  isStarterCard: (card: { isStarter: boolean }) => card.isStarter,
  tideIconUrl: (tide: string) => `/mock/${tide}.png`,
}));

const DREAMSIGN_TEMPLATES: DreamsignTemplate[] = [
  {
    id: "dreamsign-1",
    name: "Dreamsign One",
    displayTide: "Bloom",
    packageTides: ["alpha"],
    effectDescription: "First effect.",
  },
];

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
    screen: { type: "site", siteId: "site-1" },
    activeSiteId: null,
    ...overrides,
  };
}

function makeCardDatabase(): Map<number, CardData> {
  return new Map([
    [
      1,
      {
        name: "Card Reward",
        id: "card-reward",
        cardNumber: 1,
        cardType: "Character",
        subtype: "",
        isStarter: false,
        energyCost: 1,
        spark: 1,
        isFast: false,
        tides: ["alpha"],
        renderedText: "Reward text.",
        imageNumber: 1,
        artOwned: false,
      },
    ],
  ]);
}

function setQuestContext(
  state: QuestState,
  mutations: QuestMutations,
  cardDatabase: Map<number, CardData> = makeCardDatabase(),
): void {
  vi.mocked(useQuest).mockReturnValue({
    state,
    mutations,
    cardDatabase,
    questContent: {
      cardDatabase,
      cardsByPackageTide: new Map(),
      dreamcallers: [],
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
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

function clickButton(container: HTMLElement, label: string): void {
  const button = Array.from(container.querySelectorAll("button")).find(
    (candidate) => candidate.textContent?.trim() === label,
  );
  if (!button) {
    throw new Error(`Could not find button with label: ${label}`);
  }
  act(() => {
    button.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("RewardSiteScreen", () => {
  it("spends a shown Dreamsign on reveal and keeps it spent when declined", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const mutations = makeMutations();
    setQuestContext(
      makeState({
        remainingDreamsignPool: ["dreamsign-1"],
      }),
      mutations,
      new Map(),
    );

    const { container, root } = mount(
      <RewardSiteScreen
        site={{ id: "site-1", type: "Reward", isEnhanced: false, isVisited: false }}
      />,
    );

    expect(container.textContent).toContain("Dreamsign One");
    expect(container.textContent).not.toContain("Bloom");
    expect(container.querySelector('img[alt="Bloom"]')).toBeNull();
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      [],
      "reward_site_revealed",
    );

    clickButton(container, "Decline");

    expect(mutations.addDreamsign).not.toHaveBeenCalled();
    expect(mutations.markSiteVisited).toHaveBeenCalledWith("site-1");
    expect(mutations.setScreen).toHaveBeenCalledWith({ type: "dreamscape" });
    expect(logEvent).toHaveBeenCalledWith(
      "reward_declined",
      expect.objectContaining({ rewardType: "dreamsign" }),
    );

    act(() => {
      root.unmount();
    });
  });

  it("keeps the revealed reward stable across rerenders after spending the shared Dreamsign pool", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const mutations = makeMutations();
    const initialState = makeState({
      remainingDreamsignPool: ["dreamsign-1"],
    });

    setQuestContext(
      initialState,
      mutations,
      new Map(),
    );

    const element = (
      <RewardSiteScreen
        site={{ id: "site-1", type: "Reward", isEnhanced: false, isVisited: false }}
      />
    );
    const { container, root } = mount(element);

    expect(container.textContent).toContain("Dreamsign One");
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledTimes(1);
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      [],
      "reward_site_revealed",
    );

    setQuestContext(
      makeState({
        remainingDreamsignPool: [],
      }),
      mutations,
      new Map(),
    );

    act(() => {
      root.render(element);
    });

    expect(container.textContent).toContain("Dreamsign One");
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledTimes(1);

    act(() => {
      root.unmount();
    });
  });

  it("reveals a card reward without mutating the Dreamsign pool", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);
    const mutations = makeMutations();
    setQuestContext(makeState(), mutations);

    const { container, root } = mount(
      <RewardSiteScreen
        site={{ id: "site-1", type: "Reward", isEnhanced: false, isVisited: false }}
      />,
    );

    expect(container.textContent).toContain("Card Reward");
    expect(mutations.setRemainingDreamsignPool).not.toHaveBeenCalled();

    clickButton(container, "Accept");

    expect(mutations.addCard).toHaveBeenCalledWith(1, "reward_site");
    expect(mutations.markSiteVisited).toHaveBeenCalledWith("site-1");

    act(() => {
      root.unmount();
    });
  });
});
