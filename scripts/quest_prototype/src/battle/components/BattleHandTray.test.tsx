// @vitest-environment jsdom

import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createBattleInit } from "../integration/create-battle-init";
import { createInitialBattleState } from "../state/create-initial-state";
import {
  makeBattleTestCardDatabase,
  makeBattleTestDreamcallers,
  makeBattleTestSite,
  makeBattleTestState,
} from "../test-support";
import { BattleHandTray } from "./BattleHandTray";

function createState() {
  const battleInit = createBattleInit({
    battleEntryKey: "site-7::2::dreamscape-2",
    site: makeBattleTestSite(),
    state: makeBattleTestState(),
    cardDatabase: makeBattleTestCardDatabase(),
    dreamcallers: makeBattleTestDreamcallers(),
  });
  const state = createInitialBattleState(battleInit);
  state.sides.player.currentEnergy = 1;
  return state;
}

function mount(): {
  clickCard: ReturnType<typeof vi.fn>;
  container: HTMLDivElement;
  contextMenu: ReturnType<typeof vi.fn>;
  doubleClickCard: ReturnType<typeof vi.fn>;
  root: Root;
  state: ReturnType<typeof createState>;
} {
  const state = createState();
  const clickCard = vi.fn();
  const contextMenu = vi.fn();
  const doubleClickCard = vi.fn();
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <BattleHandTray
        canInteract
        currentEnergy={state.sides.player.currentEnergy}
        hand={state.sides.player.hand.slice(0, 2)}
        onHandCardAction={() => undefined}
        openingHandSize={5}
        playerDrawSkipsTurnOne
        selectedCardId={state.sides.player.hand[0] ?? null}
        state={state}
        onCardClick={clickCard}
        onCardContextMenu={contextMenu}
        onCardDoubleClick={doubleClickCard}
      />,
    );
  });

  return { clickCard, container, contextMenu, doubleClickCard, root, state };
}

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  document.body.innerHTML = "";
});

describe("BattleHandTray", () => {
  it("renders one mockup card per hand entry, including affordability state", () => {
    const { container, root, state } = mount();
    const cards = [...container.querySelectorAll<HTMLElement>("[data-battle-card-id]")];
    const firstCardId = state.sides.player.hand[0];
    const secondCardId = state.sides.player.hand[1];

    if (firstCardId === undefined || secondCardId === undefined) {
      throw new Error("expected opening hand cards");
    }

    expect(cards).toHaveLength(2);
    expect(cards[0]?.textContent).toContain(state.cardInstances[firstCardId]?.definition.name ?? "");
    expect(cards[1]?.textContent).toContain(state.cardInstances[secondCardId]?.definition.name ?? "");
    expect(cards[0]?.classList.contains("playable")).toBe(
      (state.cardInstances[firstCardId]?.definition.energyCost ?? 0) <= state.sides.player.currentEnergy,
    );
    expect(cards[1]?.classList.contains("unaffordable")).toBe(
      (state.cardInstances[secondCardId]?.definition.energyCost ?? 0) > state.sides.player.currentEnergy,
    );
    expect(cards[0]?.classList.contains("selected")).toBe(true);

    act(() => {
      root.unmount();
    });
  });

  it("forwards click, double-click, and context-menu events with the battle card id", () => {
    const { clickCard, container, contextMenu, doubleClickCard, root } = mount();
    const firstCard = container.querySelector<HTMLElement>("[data-battle-card-id]");
    const battleCardId = firstCard?.getAttribute("data-battle-card-id");

    if (firstCard === null || battleCardId === null) {
      throw new Error("expected first hand card");
    }

    act(() => {
      firstCard.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      firstCard.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
      firstCard.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true }));
    });

    expect(clickCard).toHaveBeenCalledWith(battleCardId);
    expect(doubleClickCard).toHaveBeenCalledWith(battleCardId);
    expect(contextMenu.mock.calls[0]?.[0]).toBe(battleCardId);

    act(() => {
      root.unmount();
    });
  });
});
