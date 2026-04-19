import type { QuestContent } from "../data/quest-content";
import { bootstrapQuestStart } from "../screens/quest-start-bootstrap";
import type { CardData } from "../types/cards";
import type { QuestMutations } from "../state/quest-context";
import type { QuestState } from "../types/quest";

const MAX_BOOTSTRAP_STEPS = 16;

export interface PlayableBattleBootstrapArgs {
  state: QuestState;
  mutations: QuestMutations;
  questContent: QuestContent;
  cardDatabase: Map<number, CardData>;
}

export type PlayableBattleBootstrapTerminalStep =
  | { stage: "skipped"; reason: "no-content" }
  | { stage: "skipped"; reason: "no-battle-site" }
  | { stage: "stalled"; reason: "max-steps-exceeded"; lastAction: PlayableBattleBootstrapAction | null }
  | { stage: "complete" };

export type PlayableBattleBootstrapAction =
  | "quest-start"
  | "select-dreamscape"
  | "enter-battle";

export type PlayableBattleBootstrapInProgressStep =
  | { stage: "in-progress"; action: PlayableBattleBootstrapAction };

export type PlayableBattleBootstrapStep =
  | PlayableBattleBootstrapTerminalStep
  | PlayableBattleBootstrapInProgressStep;

/**
 * Returns true when the bootstrap step indicates no further iterations are
 * required. Covers the successful-completion, no-content skip, no-battle-site
 * skip, and stalled-after-max-steps paths (bug-027).
 */
export function isPlayableBattleBootstrapTerminal(
  step: PlayableBattleBootstrapStep,
): step is PlayableBattleBootstrapTerminalStep {
  return (
    step.stage === "complete" ||
    step.stage === "skipped" ||
    step.stage === "stalled"
  );
}

export interface PlayableBattleBootstrapController {
  advance: (args: PlayableBattleBootstrapArgs) => PlayableBattleBootstrapStep;
  isDone: () => boolean;
}

/**
 * Creates a stateful bootstrap controller. Tracks a step counter and the last
 * action so callers (App.tsx) don't need ad-hoc fire-key strings to dedup
 * effect reruns. After `MAX_BOOTSTRAP_STEPS` attempts without reaching
 * `complete`/`skipped`, the controller latches `stalled` so a stuck mutation
 * pipeline surfaces explicitly rather than looping silently (bug-027).
 */
export function createPlayableBattleBootstrapController(): PlayableBattleBootstrapController {
  let steps = 0;
  let lastStep: PlayableBattleBootstrapStep | null = null;
  let done = false;

  function advance(args: PlayableBattleBootstrapArgs): PlayableBattleBootstrapStep {
    if (done && lastStep !== null) {
      return lastStep;
    }

    steps += 1;

    if (steps > MAX_BOOTSTRAP_STEPS) {
      const lastAction =
        lastStep !== null && lastStep.stage === "in-progress"
          ? lastStep.action
          : null;
      lastStep = {
        stage: "stalled",
        reason: "max-steps-exceeded",
        lastAction,
      };
      done = true;
      return lastStep;
    }

    const step = runPlayableBattleBootstrapStep(args);
    lastStep = step;
    if (isPlayableBattleBootstrapTerminal(step)) {
      done = true;
    }
    return step;
  }

  return {
    advance,
    isDone: () => done,
  };
}

export function runPlayableBattleBootstrapStep({
  state,
  mutations,
  questContent,
  cardDatabase,
}: PlayableBattleBootstrapArgs): PlayableBattleBootstrapStep {
  if (questContent.dreamcallers.length === 0) {
    return { stage: "skipped", reason: "no-content" };
  }

  if (state.dreamcaller === null) {
    const firstDreamcaller = questContent.dreamcallers[0];
    bootstrapQuestStart({
      dreamcaller: firstDreamcaller,
      state: {
        completionLevel: state.completionLevel,
        deck: state.deck,
        dreamsigns: state.dreamsigns,
        essence: state.essence,
      },
      mutations,
      cardDatabase,
      questContent,
    });
    return { stage: "in-progress", action: "quest-start" };
  }

  if (state.currentDreamscape === null) {
    const firstAvailable = Object.values(state.atlas.nodes).find(
      (node) => node.status === "available",
    );

    if (firstAvailable === undefined) {
      return { stage: "skipped", reason: "no-battle-site" };
    }

    mutations.setCurrentDreamscape(firstAvailable.id);
    return { stage: "in-progress", action: "select-dreamscape" };
  }

  const currentNode = state.atlas.nodes[state.currentDreamscape];
  if (currentNode === undefined) {
    return { stage: "skipped", reason: "no-battle-site" };
  }

  const battleSite = currentNode.sites.find((site) => site.type === "Battle");
  if (battleSite === undefined) {
    return { stage: "skipped", reason: "no-battle-site" };
  }

  if (state.screen.type !== "site" || state.screen.siteId !== battleSite.id) {
    mutations.markSiteVisited(battleSite.id);
    mutations.setScreen({ type: "site", siteId: battleSite.id });
    return { stage: "in-progress", action: "enter-battle" };
  }

  return { stage: "complete" };
}
