import { useEffect, type Dispatch } from "react";
import type {
  BattleControllerAction,
} from "./controller";
import {
  selectNaturalBattleResult,
  shouldAutoClearForcedResult,
} from "./selectors";
import type { BattleInit, BattleReducerState } from "../types";

/**
 * Watches the controller's reducer state for a live forced defeat/draw whose
 * underlying natural result no longer matches after a non-force, non-skip edit.
 * When that happens, dispatches `CLEAR_FORCED_RESULT` so the overlay follows
 * the live scores instead of the stale forced flag.
 */
export function useAutoClearForcedResult(
  reducerState: BattleReducerState,
  battleInit: Pick<BattleInit, "scoreToWin" | "turnLimit">,
  dispatch: Dispatch<BattleControllerAction>,
): void {
  const isResultForced = reducerState.mutable.forcedResult !== null;
  const result = reducerState.mutable.result;
  const naturalResult = selectNaturalBattleResult(
    reducerState.mutable,
    battleInit,
  );
  const lastActivityKind = reducerState.lastActivity?.kind ?? null;

  useEffect(() => {
    if (
      !isResultForced ||
      result === null ||
      result === "victory"
    ) {
      return;
    }

    if (naturalResult === result) {
      return;
    }

    // Skip when the most recent controller activity was an undo/redo. In that
    // case the forced-result state was produced by replaying a history snapshot
    // rather than by a fresh user command, and re-dispatching CLEAR_FORCED_RESULT
    // here would wipe `history.future` and make the undo un-redoable (H-20/H-21).
    if (lastActivityKind === "undo" || lastActivityKind === "redo") {
      return;
    }

    if (!shouldAutoClearForcedResult(reducerState.history.past)) {
      return;
    }

    dispatch({ type: "CLEAR_FORCED_RESULT" });
  }, [
    dispatch,
    isResultForced,
    lastActivityKind,
    naturalResult,
    reducerState.history.past,
    result,
  ]);
}
