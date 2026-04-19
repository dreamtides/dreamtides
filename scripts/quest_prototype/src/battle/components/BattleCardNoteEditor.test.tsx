// @vitest-environment jsdom

import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BattleDebugEdit } from "../debug/commands";
import { createBattleInit } from "../integration/create-battle-init";
import { createInitialBattleState } from "../state/create-initial-state";
import {
  makeBattleTestCardDatabase,
  makeBattleTestDreamcallers,
  makeBattleTestSite,
  makeBattleTestState,
} from "../test-support";
import type { BattleMutableState } from "../types";
import { BattleCardNoteEditor } from "./BattleCardNoteEditor";

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

function createTestState(): BattleMutableState {
  const battleInit = createBattleInit({
    battleEntryKey: "site-7::2::dreamscape-2",
    site: makeBattleTestSite(),
    state: makeBattleTestState(),
    cardDatabase: makeBattleTestCardDatabase(),
    dreamcallers: makeBattleTestDreamcallers(),
  });
  return createInitialBattleState(battleInit);
}

describe("BattleCardNoteEditor", () => {
  it("defaults to 'end of next turn' expiry and submits an atStartOfTurn expiry (FIND-09-5)", () => {
    const state = createTestState();
    const battleCardId = state.sides.player.hand[0];
    const onSubmit = vi.fn<(edit: BattleDebugEdit) => void>();
    const onClose = vi.fn();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <BattleCardNoteEditor
          battleCardId={battleCardId}
          generateNoteId={() => "note_test_1"}
          onClose={onClose}
          onSubmit={onSubmit}
          state={state}
        />,
      );
    });

    // Default radio is "end-of-next-turn", not "manual".
    const defaultRadio = document.querySelector<HTMLInputElement>(
      'input[type="radio"][value="end-of-next-turn"]',
    );
    expect(defaultRadio?.checked).toBe(true);
    const manualRadio = document.querySelector<HTMLInputElement>(
      'input[type="radio"][value="manual"]',
    );
    expect(manualRadio?.checked).toBe(false);

    const textarea = document.querySelector<HTMLTextAreaElement>(
      '[data-battle-note-field="text"]',
    );
    expect(textarea).not.toBeNull();
    act(() => {
      setReactTextareaValue(textarea!, "remember this");
    });

    const addButton = document.querySelector<HTMLButtonElement>(
      '[data-battle-note-action="add"]',
    );
    act(() => {
      addButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onSubmit).toHaveBeenCalledTimes(1);
    const edit = onSubmit.mock.calls[0][0];
    if (edit.kind !== "ADD_CARD_NOTE") {
      throw new Error("expected ADD_CARD_NOTE");
    }
    expect(edit.text).toBe("remember this");
    expect(edit.expiry.kind).toBe("atStartOfTurn");
    expect(onClose).toHaveBeenCalled();

    act(() => {
      root.unmount();
    });
  });

  it("submits a manual expiry when Manual is explicitly selected", () => {
    const state = createTestState();
    const battleCardId = state.sides.player.hand[0];
    const onSubmit = vi.fn<(edit: BattleDebugEdit) => void>();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <BattleCardNoteEditor
          battleCardId={battleCardId}
          generateNoteId={() => "note_test_2"}
          onClose={() => {}}
          onSubmit={onSubmit}
          state={state}
        />,
      );
    });

    const textarea = document.querySelector<HTMLTextAreaElement>(
      '[data-battle-note-field="text"]',
    );
    act(() => {
      setReactTextareaValue(textarea!, "persist forever");
    });

    const manualRadio = document.querySelector<HTMLInputElement>(
      'input[type="radio"][value="manual"]',
    );
    act(() => {
      manualRadio?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const addButton = document.querySelector<HTMLButtonElement>(
      '[data-battle-note-action="add"]',
    );
    act(() => {
      addButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "ADD_CARD_NOTE",
        expiry: { kind: "manual" },
      }),
    );

    act(() => {
      root.unmount();
    });
  });

  it("resolves 'Expire after N turns' to an atStartOfTurn expiry at the creator's Nth upcoming turn (FIND-09-5)", () => {
    const state = createTestState();
    const battleCardId = state.sides.player.hand[0];
    const onSubmit = vi.fn<(edit: BattleDebugEdit) => void>();
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <BattleCardNoteEditor
          battleCardId={battleCardId}
          generateNoteId={() => "note_test_3"}
          onClose={() => {}}
          onSubmit={onSubmit}
          state={state}
        />,
      );
    });

    const textarea = document.querySelector<HTMLTextAreaElement>(
      '[data-battle-note-field="text"]',
    );
    act(() => {
      setReactTextareaValue(textarea!, "expires in 2 turns");
    });

    const afterNRadio = document.querySelector<HTMLInputElement>(
      'input[type="radio"][value="after-n-turns"]',
    );
    act(() => {
      afterNRadio?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    const addButton = document.querySelector<HTMLButtonElement>(
      '[data-battle-note-action="add"]',
    );
    act(() => {
      addButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });

    expect(onSubmit).toHaveBeenCalledTimes(1);
    const edit = onSubmit.mock.calls[0][0];
    if (edit.kind !== "ADD_CARD_NOTE") {
      throw new Error("expected ADD_CARD_NOTE");
    }
    expect(edit.expiry.kind).toBe("atStartOfTurn");

    act(() => {
      root.unmount();
    });
  });

  it("disables Add Note while the textarea is empty", () => {
    const state = createTestState();
    const battleCardId = state.sides.player.hand[0];
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <BattleCardNoteEditor
          battleCardId={battleCardId}
          generateNoteId={() => "note_test_3"}
          onClose={() => {}}
          onSubmit={() => {}}
          state={state}
        />,
      );
    });

    const addButton = document.querySelector<HTMLButtonElement>(
      '[data-battle-note-action="add"]',
    );
    expect(addButton?.disabled).toBe(true);

    act(() => {
      root.unmount();
    });
  });

  it("enforces the 200-char maxLength on the textarea", () => {
    const state = createTestState();
    const battleCardId = state.sides.player.hand[0];
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <BattleCardNoteEditor
          battleCardId={battleCardId}
          generateNoteId={() => "note_test_4"}
          onClose={() => {}}
          onSubmit={() => {}}
          state={state}
        />,
      );
    });

    const textarea = document.querySelector<HTMLTextAreaElement>(
      '[data-battle-note-field="text"]',
    );
    expect(textarea?.getAttribute("maxlength")).toBe("200");

    act(() => {
      root.unmount();
    });
  });

  it("resolves the card name for the heading instead of showing the raw battleCardId (bug-099)", () => {
    const state = createTestState();
    const battleCardId = state.sides.player.hand[0];
    const expectedName = state.cardInstances[battleCardId].definition.name;
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <BattleCardNoteEditor
          battleCardId={battleCardId}
          generateNoteId={() => "note_test_5"}
          onClose={() => {}}
          onSubmit={() => {}}
          state={state}
        />,
      );
    });

    const title = document.getElementById("battle-note-editor-title");
    expect(title?.textContent).toContain(expectedName);
    expect(title?.textContent).not.toContain(battleCardId);

    act(() => {
      root.unmount();
    });
  });
});

function setReactTextareaValue(
  element: HTMLTextAreaElement,
  value: string,
): void {
  // Bypass React's synthetic-event value tracker by invoking the native
  // HTMLTextAreaElement value setter directly. The `{ set: ... }` shape
  // satisfies @typescript-eslint/unbound-method without `.call` scoping.
  const descriptor = Object.getOwnPropertyDescriptor(
    window.HTMLTextAreaElement.prototype,
    "value",
  ) as { set?: (this: HTMLTextAreaElement, value: string) => void } | undefined;
  if (descriptor?.set !== undefined) {
    descriptor.set.apply(element, [value]);
  }
  element.dispatchEvent(new Event("input", { bubbles: true }));
}
