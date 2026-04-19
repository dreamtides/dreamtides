import { useEffect, useRef } from "react";
import type { Dispatch } from "react";
import type { BattleControllerAction } from "./controller";
import type { BattleReducerState } from "../types";

/**
 * Fires a single `RUN_AI_TURN` dispatch when the initial battle state starts
 * with the enemy on the main phase and no history has yet been recorded. Once
 * satisfied (or once the precondition is invalidated) the driver latches
 * permanently for the lifetime of the component, matching Phase 1's
 * "drain the opening enemy turn" contract without re-introducing string-key
 * dedupe against the redo history.
 *
 * bug-070 / spec §H-15: RUN_AI_TURN normally fires as part of an END_TURN
 * composite (see `endTurnWithAiFollowup` in `reducer.ts`), which keeps the
 * player's end-turn and the ensuing AI turn on a single history entry. The
 * standalone dispatch below is only legal because it is the *first* entry in
 * the history (the precondition above requires `history.past.length === 0`).
 * Any future code that dispatches RUN_AI_TURN mid-battle would violate H-15;
 * fold such a dispatch into its triggering composite instead.
 */
export function useAiTurnDriver(
  reducerState: BattleReducerState,
  dispatch: Dispatch<BattleControllerAction>,
): void {
  const hasDrainedRef = useRef(false);

  useEffect(() => {
    if (hasDrainedRef.current) {
      return;
    }

    if (
      reducerState.mutable.result !== null ||
      reducerState.mutable.activeSide !== "enemy" ||
      reducerState.mutable.phase !== "main" ||
      reducerState.history.past.length > 0
    ) {
      hasDrainedRef.current = true;
      return;
    }

    hasDrainedRef.current = true;
    dispatch({ type: "RUN_AI_TURN" });
  }, [
    dispatch,
    reducerState.history.past.length,
    reducerState.mutable.activeSide,
    reducerState.mutable.phase,
    reducerState.mutable.result,
  ]);
}
