// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { HTMLAttributes, ReactElement, ReactNode } from "react";
import type { DreamsignTemplate } from "../types/content";
import type { QuestMutations } from "../state/quest-context";
import type { QuestState, SiteState } from "../types/quest";
import { DreamsignDraftScreen } from "./DreamsignDraftScreen";
import { DreamsignOfferingScreen } from "./DreamsignOfferingScreen";
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
  },
}));

vi.mock("../state/quest-context", () => ({
  useQuest: vi.fn(),
}));

vi.mock("../logging", () => ({
  logEvent: vi.fn(),
}));

vi.mock("../data/card-database", () => ({
  TIDE_COLORS: {
    Ignite: "#f97316",
    Rime: "#60a5fa",
    Bloom: "#34d399",
    Arc: "#a78bfa",
  },
  isStarterCard: (card: { isStarter: boolean }) => card.isStarter,
  tideIconUrl: (tide: string) => `/mock/${tide}.png`,
}));

const DREAMSIGN_TEMPLATES: DreamsignTemplate[] = [
  {
    id: "embers-whisper",
    name: "Ember's Whisper",
    effectDescription: "Fire.",
    imageName: "embers-whisper.png",
    packageTides: ["ember"],
  },
  {
    id: "glacial-insight",
    name: "Glacial Insight",
    effectDescription: "Ice.",
    imageName: "glacial-insight.png",
    packageTides: ["frost"],
  },
  {
    id: "verdant-accord",
    name: "Verdant Accord",
    effectDescription: "Growth.",
    imageName: "verdant-accord.png",
    packageTides: ["grove"],
  },
  {
    id: "stormthread-sigil",
    name: "Stormthread Sigil",
    effectDescription: "Storm.",
    imageName: "stormthread-sigil.png",
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
    screen: { type: "site", siteId: "site-1" },
    activeSiteId: null,
    ...overrides,
  };
}

function makeSite(overrides: Partial<SiteState> = {}): SiteState {
  return {
    id: "site-1",
    type: "DreamsignOffering",
    isEnhanced: false,
    isVisited: false,
    ...overrides,
  };
}

function makeResolvedPackage(
  selectedTides: string[],
  dreamsignPoolIds: string[],
) {
  return {
    dreamcaller: {
      id: "dc-1",
      name: "Dreamcaller",
      title: "Title",
      awakening: 1,
      renderedText: "",
      imageNumber: "0001",
      mandatoryTides: selectedTides,
      optionalTides: [],
    },
    mandatoryTides: [...selectedTides],
    optionalSubset: [],
    selectedTides: [...selectedTides],
    draftPoolCopiesByCard: {},
    dreamsignPoolIds: [...dreamsignPoolIds],
    mandatoryOnlyPoolSize: 0,
    draftPoolSize: 0,
    doubledCardCount: 0,
    legalSubsetCount: 0,
    preferredSubsetCount: 0,
  };
}

function setQuestContext(state: QuestState, mutations: QuestMutations): void {
  vi.mocked(useQuest).mockReturnValue({
    state,
    mutations,
    cardDatabase: new Map(),
    questContent: {
      cardDatabase: new Map(),
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
    (candidate) => candidate.textContent?.includes(label),
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
  vi.spyOn(Math, "random").mockReturnValue(0.999);
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("DreamsignOfferingScreen", () => {
  it("spends the shown dreamsign on reveal and keeps it spent when rejected", () => {
    const mutations = makeMutations();
    setQuestContext(
      makeState({
        remainingDreamsignPool: ["embers-whisper", "glacial-insight"],
        resolvedPackage: makeResolvedPackage(
          ["ember"],
          ["embers-whisper", "glacial-insight"],
        ),
      }),
      mutations,
    );

    const { container, root } = mount(<DreamsignOfferingScreen site={makeSite()} />);

    expect(container.textContent).toContain("Ember's Whisper");
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      ["glacial-insight"],
      "dreamsign_offering_revealed",
    );

    clickButton(container, "Reject");

    expect(mutations.addDreamsign).not.toHaveBeenCalled();
    expect(mutations.markSiteVisited).toHaveBeenCalledWith("site-1");
    expect(mutations.setScreen).toHaveBeenCalledWith({ type: "dreamscape" });
    expect(logEvent).toHaveBeenCalledWith(
      "site_completed",
      expect.objectContaining({ siteType: "DreamsignOffering" }),
    );

    act(() => {
      root.unmount();
    });
  });

  it("does not restore a shown dreamsign to the pool during full-capacity purge acceptance", () => {
    const mutations = makeMutations();
    setQuestContext(
      makeState({
        remainingDreamsignPool: ["embers-whisper", "glacial-insight"],
        dreamsigns: Array.from({ length: 12 }, (_, index) => ({
          name: `Held Sign ${String(index)}`,
          tide: index % 2 === 0 ? "Bloom" : "Rime",
          effectDescription: "Existing.",
          isBane: false,
        })),
        resolvedPackage: makeResolvedPackage(
          ["ember"],
          ["embers-whisper", "glacial-insight"],
        ),
      }),
      mutations,
    );

    const { container, root } = mount(<DreamsignOfferingScreen site={makeSite()} />);

    clickButton(container, "Accept");
    clickButton(container, "Held Sign 0");

    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledTimes(1);
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      ["glacial-insight"],
      "dreamsign_offering_revealed",
    );
    expect(mutations.removeDreamsign).toHaveBeenCalledWith(
      0,
      "purged_for_new_dreamsign",
    );
    expect(mutations.addDreamsign).toHaveBeenCalledWith(
      expect.objectContaining({ name: "Ember's Whisper" }),
      "DreamsignOffering",
    );

    act(() => {
      root.unmount();
    });
  });
});

describe("DreamsignDraftScreen", () => {
  it("spends revealed draft options on mount and keeps them spent when skipped", () => {
    const mutations = makeMutations();
    setQuestContext(
      makeState({
        remainingDreamsignPool: [
          "embers-whisper",
          "glacial-insight",
          "verdant-accord",
          "stormthread-sigil",
        ],
        resolvedPackage: makeResolvedPackage(
          ["ember", "frost", "grove"],
          [
            "embers-whisper",
            "glacial-insight",
            "verdant-accord",
            "stormthread-sigil",
          ],
        ),
      }),
      mutations,
    );

    const { container, root } = mount(
      <DreamsignDraftScreen
        site={makeSite({ type: "DreamsignDraft" })}
      />,
    );

    expect(container.textContent).toContain("Ember's Whisper");
    expect(container.textContent).toContain("Glacial Insight");
    expect(container.textContent).toContain("Verdant Accord");
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      ["stormthread-sigil"],
      "dreamsign_draft_revealed",
    );

    clickButton(container, "Skip (discard both)");

    expect(mutations.addDreamsign).not.toHaveBeenCalled();
    expect(mutations.markSiteVisited).toHaveBeenCalledWith("site-1");
    expect(mutations.setScreen).toHaveBeenCalledWith({ type: "dreamscape" });

    act(() => {
      root.unmount();
    });
  });

  it("degrades to an exhausted-pool fallback without rerolling from the global catalog", () => {
    const mutations = makeMutations();
    setQuestContext(
      makeState({
        remainingDreamsignPool: ["missing-id"],
        resolvedPackage: makeResolvedPackage(["ember"], ["missing-id"]),
      }),
      mutations,
    );

    const { container, root } = mount(
      <DreamsignDraftScreen
        site={makeSite({ type: "DreamsignDraft" })}
      />,
    );

    expect(container.textContent).toContain("The Dreamsign pool is exhausted.");
    expect(container.textContent).not.toContain("Ember's Whisper");
    expect(container.querySelectorAll("button")).toHaveLength(1);
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      [],
      "dreamsign_draft_revealed",
    );

    clickButton(container, "Skip");

    expect(mutations.addDreamsign).not.toHaveBeenCalled();
    expect(mutations.markSiteVisited).toHaveBeenCalledWith("site-1");

    act(() => {
      root.unmount();
    });
  });

  it("filters draft options to selected and neutral dreamsigns", () => {
    const mutations = makeMutations();
    setQuestContext(
      makeState({
        remainingDreamsignPool: [
          "embers-whisper",
          "glacial-insight",
          "stormthread-sigil",
        ],
        resolvedPackage: makeResolvedPackage(
          ["ember"],
          [
            "embers-whisper",
            "glacial-insight",
            "stormthread-sigil",
          ],
        ),
      }),
      mutations,
    );

    const { container, root } = mount(
      <DreamsignDraftScreen
        site={makeSite({ type: "DreamsignDraft" })}
      />,
    );

    expect(container.textContent).toContain("Ember's Whisper");
    expect(container.textContent).toContain("Stormthread Sigil");
    expect(container.textContent).not.toContain("Glacial Insight");
    expect(mutations.setRemainingDreamsignPool).toHaveBeenCalledWith(
      ["glacial-insight"],
      "dreamsign_draft_revealed",
    );

    act(() => {
      root.unmount();
    });
  });
});
