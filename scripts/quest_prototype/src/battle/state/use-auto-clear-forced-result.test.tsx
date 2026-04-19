// @vitest-environment jsdom

import { act, type Dispatch, useEffect } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { resetLog } from "../../logging";
import { createBattleInit } from "../integration/create-battle-init";
import {
  makeBattleTestCardDatabase,
  makeBattleTestDreamcallers,
  makeBattleTestSite,
  makeBattleTestState,
} from "../test-support";
import type { BattleInit, BattleMutableState, BattleReducerState } from "../types";
import {
  type BattleControllerAction,
  useBattleController,
} from "./controller";
import { createInitialBattleState } from "./create-initial-state";
import { useAutoClearForcedResult } from "./use-auto-clear-forced-result";

interface HarnessSnapshot {
  dispatch: Dispatch<BattleControllerAction>;
  state: BattleReducerState;
}

function mountHarness({
  battleInit,
  initialState,
  onStateChange,
}: {
  battleInit: BattleInit;
  initialState: BattleMutableState;
  onStateChange: (value: HarnessSnapshot) => void;
}) {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <Harness
        battleInit={battleInit}
        initialState={initialState}
        onStateChange={onStateChange}
      />,
    );
  });

  return { container, root };
}

function Harness({
  battleInit,
  initialState,
  onStateChange,
}: {
  battleInit: BattleInit;
  initialState: BattleMutableState;
  onStateChange: (value: HarnessSnapshot) => void;
}) {
  const [state, dispatch] = useBattleController(initialState, battleInit);
  useAutoClearForcedResult(state, battleInit, dispatch);

  useEffect(() => {
    onStateChange({ dispatch, state });
  }, [dispatch, onStateChange, state]);

  return null;
}

function readLatestSnapshot(
  snapshot: HarnessSnapshot | null,
): HarnessSnapshot {
  if (snapshot === null) {
    throw new Error("Harness did not publish a snapshot");
  }

  return snapshot;
}

beforeEach(() => {
  resetLog();
  vi.spyOn(console, "log").mockImplementation(() => {});
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  document.body.innerHTML = "";
  vi.restoreAllMocks();
});

describe("useAutoClearForcedResult", () => {
  it("clears the forced defeat when a follow-up edit leaves defeat no longer natural", () => {
    const { battleInit, initialState } = createTestBattle();
    let latest: HarnessSnapshot | null = null;
    const { root } = mountHarness({
      battleInit,
      initialState,
      onStateChange: (value) => {
        latest = value;
      },
    });

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: { id: "FORCE_RESULT", result: "defeat" },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBe("defeat");
    expect(readLatestSnapshot(latest).state.mutable.result).toBe("defeat");

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: {
          id: "DEBUG_EDIT",
          edit: {
            kind: "ADJUST_SCORE",
            side: "player",
            amount: 1,
          },
        },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBeNull();
    expect(readLatestSnapshot(latest).state.mutable.result).toBeNull();

    act(() => {
      root.unmount();
    });
  });

  it("keeps a forced defeat that is still naturally supported (naturalResult === result)", () => {
    const { battleInit, initialState } = createTestBattle();
    initialState.sides.enemy.score = battleInit.scoreToWin;
    let latest: HarnessSnapshot | null = null;
    const { root } = mountHarness({
      battleInit,
      initialState,
      onStateChange: (value) => {
        latest = value;
      },
    });

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: { id: "FORCE_RESULT", result: "defeat" },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBe("defeat");

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: {
          id: "DEBUG_EDIT",
          edit: {
            kind: "ADJUST_SCORE",
            side: "enemy",
            amount: 1,
          },
        },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBe("defeat");
    expect(readLatestSnapshot(latest).state.mutable.result).toBe("defeat");

    act(() => {
      root.unmount();
    });
  });

  it("folds auto-clear into the triggering edit (bug-019: one history entry per user action)", () => {
    // Bug 019 / H-7 / K-13: a gameplay edit that invalidates the forced
    // defeat must clear the flag inline inside the same history commit, so
    // one user gesture = one undo step. Previously the reducer committed the
    // ADJUST_SCORE entry and then `useAutoClearForcedResult` dispatched a
    // separate `CLEAR_FORCED_RESULT` entry, which doubled the undo depth
    // (and could land the first Undo on a half-undone state).
    const { battleInit, initialState } = createTestBattle();
    let latest: HarnessSnapshot | null = null;
    const { root } = mountHarness({
      battleInit,
      initialState,
      onStateChange: (value) => {
        latest = value;
      },
    });

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: { id: "FORCE_RESULT", result: "defeat" },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBe("defeat");
    expect(readLatestSnapshot(latest).state.mutable.result).toBe("defeat");

    const pastLengthBefore = readLatestSnapshot(latest).state.history.past.length;

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: {
          id: "DEBUG_EDIT",
          edit: {
            kind: "ADJUST_SCORE",
            side: "player",
            amount: 1,
          },
        },
      });
    });

    // Inline auto-clear has run: forced flag cleared as part of the
    // ADJUST_SCORE entry, *not* a separate CLEAR_FORCED_RESULT entry.
    const afterClear = readLatestSnapshot(latest).state;
    expect(afterClear.mutable.forcedResult).toBeNull();
    expect(afterClear.mutable.result).toBeNull();
    expect(afterClear.history.past.length).toBe(pastLengthBefore + 1);
    expect(
      afterClear.history.past[afterClear.history.past.length - 1]?.metadata
        .commandId,
    ).toBe("ADJUST_SCORE");

    // A single undo returns to the forced-defeat state.
    act(() => {
      readLatestSnapshot(latest).dispatch({ type: "UNDO" });
    });

    const undone = readLatestSnapshot(latest).state;
    expect(undone.mutable.forcedResult).toBe("defeat");
    expect(undone.mutable.result).toBe("defeat");
    expect(undone.history.future.length).toBe(1);
    expect(undone.history.future[0]?.metadata.commandId).toBe("ADJUST_SCORE");

    // Redo reapplies ADJUST_SCORE, which re-clears the forced flag inline.
    act(() => {
      readLatestSnapshot(latest).dispatch({ type: "REDO" });
    });

    const redone = readLatestSnapshot(latest).state;
    expect(redone.mutable.forcedResult).toBeNull();
    expect(redone.mutable.result).toBeNull();
    expect(redone.history.future.length).toBe(0);
    expect(
      redone.history.past[redone.history.past.length - 1]?.metadata.commandId,
    ).toBe("ADJUST_SCORE");

    act(() => {
      root.unmount();
    });
  });

  it("never clears a forced victory (only defeat and draw auto-clear)", () => {
    const { battleInit, initialState } = createTestBattle();
    let latest: HarnessSnapshot | null = null;
    const { root } = mountHarness({
      battleInit,
      initialState,
      onStateChange: (value) => {
        latest = value;
      },
    });

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: { id: "FORCE_RESULT", result: "victory" },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBe("victory");

    act(() => {
      readLatestSnapshot(latest).dispatch({
        type: "APPLY_COMMAND",
        command: {
          id: "DEBUG_EDIT",
          edit: {
            kind: "ADJUST_SCORE",
            side: "enemy",
            amount: -1,
          },
        },
      });
    });

    expect(readLatestSnapshot(latest).state.mutable.forcedResult).toBe("victory");
    expect(readLatestSnapshot(latest).state.mutable.result).toBe("victory");

    act(() => {
      root.unmount();
    });
  });
});

function createTestBattle() {
  const battleInit = createBattleInit({
    battleEntryKey: "site-7::2::dreamscape-2",
    site: makeBattleTestSite(),
    state: makeBattleTestState(),
    cardDatabase: makeBattleTestCardDatabase(),
    dreamcallers: makeBattleTestDreamcallers(),
  });

  return {
    battleInit,
    initialState: createInitialBattleState(battleInit),
  };
}
