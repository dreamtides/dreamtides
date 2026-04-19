// @vitest-environment jsdom

import { act, type ComponentPropsWithoutRef, type ReactNode } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CompleteBattleSiteVictoryInput } from "../battle/integration/battle-completion-bridge";
import { createBattleInit } from "../battle/integration/create-battle-init";
import { resetLog } from "../logging";
import { useQuest } from "../state/quest-context";
import {
  makeBattleTestCardDatabase,
  makeBattleTestDreamcallers,
  makeBattleTestSite,
  makeBattleTestState,
} from "../battle/test-support";
import { AutoBattleScreen } from "./AutoBattleScreen";

const mockedBridge = vi.hoisted(() => ({
  completeBattleSiteVictory: vi.fn<(input: CompleteBattleSiteVictoryInput) => void>(),
}));

let nextAnimationFrameId = 1;
const animationFrameCallbacks = new Map<number, FrameRequestCallback>();

vi.mock("../state/quest-context", () => ({
  useQuest: vi.fn(),
}));

vi.mock("../battle/integration/battle-completion-bridge", () => ({
  completeBattleSiteVictory: mockedBridge.completeBattleSiteVictory,
}));

vi.mock("framer-motion", async () => {
  const reactModule = await import("react");

  return {
    AnimatePresence: ({ children }: { children: ReactNode }) => children,
    motion: {
      button: createMockMotionComponent("button", reactModule.createElement),
      div: createMockMotionComponent("div", reactModule.createElement),
      h1: createMockMotionComponent("h1", reactModule.createElement),
      h2: createMockMotionComponent("h2", reactModule.createElement),
      img: createMockMotionComponent("img", reactModule.createElement),
      p: createMockMotionComponent("p", reactModule.createElement),
      span: createMockMotionComponent("span", reactModule.createElement),
    },
  };
});

function createMockMotionComponent<Tag extends keyof HTMLElementTagNameMap>(
  tag: Tag,
  createElement: typeof import("react").createElement,
) {
  return function MockMotionComponent({
    children,
    ...props
  }: ComponentPropsWithoutRef<Tag> & {
    animate?: unknown;
    exit?: unknown;
    initial?: unknown;
    transition?: unknown;
    whileHover?: unknown;
    whileTap?: unknown;
  }) {
    return createElement(tag, props, children);
  };
}

function mountScreen() {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  const cardDatabase = makeBattleTestCardDatabase();
  const battleInit = createBattleInit({
    battleEntryKey: "site-7::2::dreamscape-2",
    site: makeBattleTestSite(),
    state: makeBattleTestState(),
    cardDatabase,
    dreamcallers: makeBattleTestDreamcallers(),
  });

  act(() => {
    root.render(
      <AutoBattleScreen
        battleInit={battleInit}
        site={makeBattleTestSite()}
      />,
    );
  });

  return {
    cardDatabase,
    container,
    root,
  };
}

beforeEach(() => {
  resetLog();
  vi.useFakeTimers();
  vi.spyOn(console, "log").mockImplementation(() => {});
  animationFrameCallbacks.clear();
  nextAnimationFrameId = 1;
  vi.spyOn(globalThis, "requestAnimationFrame").mockImplementation((callback) => {
    const frameId = nextAnimationFrameId;
    nextAnimationFrameId += 1;
    animationFrameCallbacks.set(frameId, callback);
    return frameId;
  });
  vi.spyOn(globalThis, "cancelAnimationFrame").mockImplementation((frameId) => {
    animationFrameCallbacks.delete(frameId);
  });
  mockedBridge.completeBattleSiteVictory.mockReset();
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
  const questValue: ReturnType<typeof useQuest> = {
    state: {
      ...makeBattleTestState(),
      screen: { type: "site", siteId: "site-7" },
      activeSiteId: "site-7",
      essence: 250,
      visitedSites: [],
      cardSourceDebug: null,
      remainingDreamsignPool: [],
      draftState: null,
      failureSummary: null,
    },
    mutations: {
      addCard: vi.fn(),
      addBaneCard: vi.fn(),
      addDreamsign: vi.fn(),
      changeEssence: vi.fn(),
      incrementCompletionLevel: vi.fn(),
      markSiteVisited: vi.fn(),
      removeCard: vi.fn(),
      removeDreamsign: vi.fn(),
      resetQuest: vi.fn(),
      setCardSourceDebug: vi.fn(),
      setCurrentDreamscape: vi.fn(),
      setDraftState: vi.fn(),
      setDreamcallerSelection: vi.fn(),
      setFailureSummary: vi.fn(),
      setRemainingDreamsignPool: vi.fn(),
      setScreen: vi.fn(),
      transfigureCard: vi.fn(),
      updateAtlas: vi.fn(),
    },
    cardDatabase: makeBattleTestCardDatabase(),
    questContent: {
      cardDatabase: makeBattleTestCardDatabase(),
      cardsByPackageTide: new Map(),
      dreamcallers: makeBattleTestDreamcallers(),
      dreamsignTemplates: [],
      resolvedPackagesByDreamcallerId: new Map(),
    },
  };
  vi.mocked(useQuest).mockReturnValue(questValue);
});

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
  document.body.innerHTML = "";
});

describe("AutoBattleScreen", () => {
  it("keeps the auto victory route on the shared completion bridge", () => {
    const { container, root } = mountScreen();

    clickButtonByText(container, "Start Battle");

    act(() => {
      vi.advanceTimersByTime(1500);
    });

    expect(container.textContent).toContain("Choose a Card Reward");
    expect(mockedBridge.completeBattleSiteVictory).not.toHaveBeenCalled();

    const rewardCardButton = container.querySelector<HTMLElement>('[role="button"]');

    if (rewardCardButton === null) {
      throw new Error("Missing reward card button");
    }

    act(() => {
      rewardCardButton.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
      rewardCardButton.click();
    });

    // FIND-08-2: armed but not committed yet; press Confirm Reward to commit.
    const confirmButton = container.querySelector<HTMLButtonElement>(
      '[data-battle-reward-action="confirm"]',
    );
    if (confirmButton === null) {
      throw new Error("Missing reward confirm button");
    }
    act(() => {
      confirmButton.click();
    });

    const bridgeInput = mockedBridge.completeBattleSiteVictory.mock.calls[0]?.[0];

    expect(bridgeInput).toBeDefined();
    expect(bridgeInput?.battleId).toBe("battle:site-7::2::dreamscape-2");
    expect(bridgeInput?.siteId).toBe("site-7");
    expect(bridgeInput?.postVictoryHandoffDelayMs).toBe(800);
    expect(bridgeInput?.selectedRewardCard.cardNumber).toEqual(expect.any(Number));
    expect(bridgeInput?.selectedRewardCard.name).toEqual(expect.any(String));

    act(() => {
      root.unmount();
    });
  });

  it("clears pending battle-animation timers when unmounted mid-animation (bug-095)", () => {
    const { container, root } = mountScreen();

    clickButtonByText(container, "Start Battle");

    // Unmount before the 1500ms animation elapses to prove the cleanup
    // removes the setTimeout id from `timersRef.current`.
    const clearSpy = vi.spyOn(globalThis, "clearTimeout");
    act(() => {
      root.unmount();
    });
    expect(clearSpy).toHaveBeenCalled();
  });

  it("guards against double-click on Start Battle (bug-095)", () => {
    const { container, root } = mountScreen();

    // Two rapid Start Battle presses must not queue two victory transitions.
    clickButtonByText(container, "Start Battle");
    expect(() => clickButtonByText(container, "Start Battle")).toThrowError(
      /Missing button with text Start Battle/,
    );

    act(() => {
      vi.advanceTimersByTime(1500);
    });

    expect(container.textContent).toContain("Choose a Card Reward");

    act(() => {
      root.unmount();
    });
  });

  it("bridges with the exact BattleInit fields that playable mode uses (bug-025 / §M-15 parity)", () => {
    // Deterministic seed + fixtures: both auto and playable share this
    // `createBattleInit(...)` bootstrap (bug-007). We mount auto here and
    // assert bridge input keys that playable mode's own test expects.
    const { container, root } = mountScreen();

    clickButtonByText(container, "Start Battle");
    act(() => {
      vi.advanceTimersByTime(1500);
    });
    const rewardCardButton = container.querySelector<HTMLElement>('[role="button"]');
    if (rewardCardButton === null) {
      throw new Error("Missing reward card button");
    }
    act(() => {
      rewardCardButton.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
      rewardCardButton.click();
    });
    // FIND-08-2: confirm the armed selection.
    const confirmButton = container.querySelector<HTMLButtonElement>(
      '[data-battle-reward-action="confirm"]',
    );
    if (confirmButton === null) {
      throw new Error("Missing reward confirm button");
    }
    act(() => {
      confirmButton.click();
    });

    const bridgeInput = mockedBridge.completeBattleSiteVictory.mock.calls[0]?.[0];
    expect(bridgeInput).toBeDefined();
    // Mirrors the playable-screen assertion in PlayableBattleScreen.test.tsx:
    // same keys, same postVictoryHandoffDelayMs, same battleId/siteId pairing.
    expect(Object.keys(bridgeInput ?? {}).sort()).toEqual(
      [
        "atlasSnapshot",
        "battleId",
        "completionLevelAtBattleStart",
        "dreamscapeId",
        "essenceReward",
        "isFinalBoss",
        "isMiniboss",
        "mutations",
        "playerHasBanes",
        "postVictoryHandoffDelayMs",
        "selectedRewardCard",
        "siteId",
      ].sort(),
    );

    act(() => {
      root.unmount();
    });
  });

  it("keeps reward options stable across the animation → victory transition (bug-095 / bug-096)", () => {
    const { container, root } = mountScreen();

    clickButtonByText(container, "Start Battle");

    // Snapshot the reward buttons' names immediately after victory opens.
    act(() => {
      vi.advanceTimersByTime(1500);
    });
    const firstSnapshot = extractRewardCardNames(container);
    expect(firstSnapshot.length).toBeGreaterThanOrEqual(1);

    // Advance more time to force a re-render cycle; the reward options should
    // stay frozen because they are drawn from the `BattleInit` snapshot.
    act(() => {
      vi.advanceTimersByTime(500);
    });
    const secondSnapshot = extractRewardCardNames(container);
    expect(secondSnapshot).toEqual(firstSnapshot);

    act(() => {
      root.unmount();
    });
  });
});

function extractRewardCardNames(container: HTMLElement): string[] {
  return Array.from(container.querySelectorAll<HTMLElement>('[role="button"]'))
    .map((node) => node.getAttribute("aria-label") ?? node.textContent ?? "")
    .filter((value) => value.length > 0);
}

function clickButtonByText(
  container: HTMLDivElement,
  text: string,
): void {
  const button = Array.from(container.querySelectorAll<HTMLButtonElement>("button"))
    .find((element) => element.textContent?.includes(text));

  if (button === undefined) {
    throw new Error(`Missing button with text ${text}`);
  }

  act(() => {
    button.click();
  });
}
