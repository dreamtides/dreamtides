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
import { BattleZoneBrowser } from "./BattleZoneBrowser";

function createState() {
  const battleInit = createBattleInit({
    battleEntryKey: "site-7::2::dreamscape-2",
    site: makeBattleTestSite(),
    state: makeBattleTestState(),
    cardDatabase: makeBattleTestCardDatabase(),
    dreamcallers: makeBattleTestDreamcallers(),
  });
  const state = createInitialBattleState(battleInit);
  return state;
}

function mount(browser: { side: "player" | "enemy"; zone: "deck" | "hand" | "void" | "banished" }): {
  container: HTMLDivElement;
  onCommand: ReturnType<typeof vi.fn>;
  onOpenForesee: ReturnType<typeof vi.fn>;
  onOpenReorderMultiple: ReturnType<typeof vi.fn>;
  onSelectBattleCard: ReturnType<typeof vi.fn>;
  root: Root;
  state: ReturnType<typeof createState>;
} {
  const state = createState();
  const onCommand = vi.fn();
  const onOpenForesee = vi.fn();
  const onOpenReorderMultiple = vi.fn();
  const onSelectBattleCard = vi.fn();
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <BattleZoneBrowser
        browser={browser}
        state={state}
        selectedBattleCardId={browser.zone === "hand" ? state.sides[browser.side].hand[0] ?? null : null}
        onClose={() => undefined}
        onCommand={onCommand}
        onOpenForesee={onOpenForesee}
        onOpenReorderMultiple={onOpenReorderMultiple}
        onSelectBattleCard={onSelectBattleCard}
      />,
    );
  });

  return { container, onCommand, onOpenForesee, onOpenReorderMultiple, onSelectBattleCard, root, state };
}

afterEach(() => {
  document.body.innerHTML = "";
});

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

describe("BattleZoneBrowser", () => {
  it("renders the exact mockup header and controls for the deck browser", () => {
    const { container, onOpenForesee, onOpenReorderMultiple, onSelectBattleCard, root } = mount({ side: "player", zone: "deck" });

    expect(container.textContent).toContain("Your Deck");
    expect(
      container.querySelector<HTMLInputElement>("[data-zone-browser-search]")?.placeholder,
    ).toBe("Search by name…");
    expect(
      [...container.querySelectorAll<HTMLSelectElement>("select")].flatMap((element) =>
        [...element.options].map((option) => option.text),
      ),
    ).toEqual([
      "Current order",
      "Cost",
      "Spark",
      "Name",
      "All types",
      "Characters",
      "Events",
    ]);

    act(() => {
      container.querySelector<HTMLElement>('[data-zone-browser-card-id]')?.click();
    });

    expect(onSelectBattleCard).toHaveBeenCalledTimes(1);
    expect(container.textContent).toContain("Selected:");
    expect(container.textContent).toContain("Reveal Top");
    expect(container.textContent).toContain("Play From Top");
    expect(container.textContent).toContain("Hide Top");
    expect(container.textContent).toContain("Foresee");
    expect(container.textContent).toContain("Reorder Full Deck");
    expect(container.textContent).toContain("→ Hand");
    expect(container.textContent).toContain("→ Battlefield");
    expect(container.textContent).toContain("→ Void");
    expect(container.textContent).toContain("→ Banished");
    expect(container.textContent).toContain("→ Deck top");
    expect(container.textContent).toContain("→ Deck bot.");

    act(() => {
      container.querySelector<HTMLElement>('[data-zone-browser-action="foresee"]')?.click();
      container.querySelector<HTMLElement>('[data-zone-browser-action="reorder-full"]')?.click();
    });

    expect(onOpenForesee).toHaveBeenCalledWith("player", 1);
    expect(onOpenReorderMultiple).toHaveBeenCalledWith("player");

    act(() => {
      root.unmount();
    });
  });

  it("exposes per-card reveal toggles for enemy hand cards", () => {
    const { container, onCommand, root } = mount({ side: "enemy", zone: "hand" });

    act(() => {
      container.querySelector<HTMLElement>('[data-zone-browser-card-id]')?.click();
    });

    expect(container.textContent).toContain("Hide");

    act(() => {
      const reveal = [...container.querySelectorAll<HTMLElement>(".chip")].find(
        (element) => element.textContent?.trim() === "Hide",
      );
      reveal?.click();
    });

    expect(onCommand.mock.calls[0]?.[0]).toMatchObject({
      id: "DEBUG_EDIT",
      edit: {
        kind: "SET_CARD_VISIBILITY",
        isRevealedToPlayer: false,
      },
    });

    act(() => {
      container.querySelector<HTMLElement>('[data-zone-browser-action="reveal-all"]')?.click();
      container.querySelector<HTMLElement>('[data-zone-browser-action="hide-all"]')?.click();
    });

    expect(onCommand.mock.calls[1]?.[0]).toMatchObject({
      id: "DEBUG_EDIT",
      edit: {
        kind: "SET_SIDE_HAND_VISIBILITY",
        side: "enemy",
        isRevealedToPlayer: true,
      },
    });
    expect(onCommand.mock.calls[2]?.[0]).toMatchObject({
      id: "DEBUG_EDIT",
      edit: {
        kind: "SET_SIDE_HAND_VISIBILITY",
        side: "enemy",
        isRevealedToPlayer: false,
      },
    });

    act(() => {
      root.unmount();
    });
  });
});
