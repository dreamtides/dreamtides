import type {
  BattleHistoryEntryMetadata,
  BattleReducerTransition,
  BattleTransitionData,
} from "../types";

/**
 * Deep-clones a `BattleReducerTransition` for snapshot/history storage.
 * `reducer.ts` (forward committing) and `history.ts` (undo/redo cloning)
 * both need this operation and previously duplicated near-identical
 * implementations (bug-016). Any future field added to
 * `BattleTransitionData` now has exactly one place to update.
 */
export function cloneBattleReducerTransition(
  transition: BattleReducerTransition,
): BattleReducerTransition;
export function cloneBattleReducerTransition(
  transition: BattleReducerTransition | null,
): BattleReducerTransition | null;
export function cloneBattleReducerTransition(
  transition: BattleReducerTransition | null,
): BattleReducerTransition | null {
  if (transition === null) {
    return null;
  }

  return {
    metadata: { ...transition.metadata },
    ...cloneBattleTransitionBody(transition),
  };
}

/**
 * Wraps a plain `BattleTransitionData` with an outer metadata envelope to
 * produce a `BattleReducerTransition`. Used by the reducer when it commits
 * a freshly-resolved action. Deep-copies nested arrays/records so the
 * stored transition is independent of the input source.
 */
export function createReducerTransition(
  metadata: BattleHistoryEntryMetadata,
  transition: BattleTransitionData,
): BattleReducerTransition {
  return {
    metadata: { ...metadata },
    ...cloneBattleTransitionBody(transition),
  };
}

function cloneBattleTransitionBody(
  transition: BattleTransitionData,
): BattleTransitionData {
  return {
    steps: transition.steps.map((step) => ({ ...step })),
    energyChanges: transition.energyChanges.map((change) => ({
      ...change,
      at: { ...change.at },
    })),
    judgment: transition.judgment === null
      ? null
      : {
        ...transition.judgment,
        lanes: transition.judgment.lanes.map((lane) => ({ ...lane })),
      },
    scoreChanges: transition.scoreChanges.map((change) => ({
      ...change,
      at: { ...change.at },
    })),
    resultChange: transition.resultChange === null
      ? null
      : {
        ...transition.resultChange,
        at: { ...transition.resultChange.at },
      },
    aiChoices: transition.aiChoices.map((choice) => ({ ...choice })),
    logEvents: transition.logEvents.map((event) => ({
      ...event,
      fields: { ...event.fields },
    })),
  };
}
