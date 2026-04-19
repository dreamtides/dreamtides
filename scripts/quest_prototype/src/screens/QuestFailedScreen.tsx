import { useCallback, useEffect } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { logEvent, logEventOnce } from "../logging";
import type { QuestFailureSummary } from "../types/quest";

/**
 * Terminal screen shown after a playable battle ends in defeat or draw and the
 * user explicitly leaves the battle. Renders a frozen {@link QuestFailureSummary}
 * captured by the battle-failure route and owns the single `resetQuest()` call
 * in the playable-battle flow.
 */
export function QuestFailedScreen() {
  const { state, mutations } = useQuest();
  const summary = state.failureSummary;

  useEffect(() => {
    if (summary === null) {
      return;
    }

    // Keyed logEventOnce survives StrictMode double-mount and router remount
    // churn because `onceKeys` is module-scoped (see `src/logging.ts`). The
    // previous hasLoggedRef approach reset on every mount.
    logEventOnce(`quest_failed_screen_shown:${summary.battleId}`, "quest_failed_screen_shown", {
      battleId: summary.battleId,
      battleMode: summary.battleMode,
      result: summary.result,
      reason: summary.reason,
      siteId: summary.siteId,
      dreamscapeIdOrNone: summary.dreamscapeIdOrNone,
      turnNumber: summary.turnNumber,
      playerScore: summary.playerScore,
      enemyScore: summary.enemyScore,
    });
  }, [summary]);

  const handleStartNewRun = useCallback(() => {
    if (summary === null) {
      return;
    }

    logEvent("quest_failed_start_new_run", {
      battleId: summary.battleId,
      result: summary.result,
    });
    mutations.resetQuest();
  }, [mutations, summary]);

  if (summary === null) {
    return (
      <div className="flex min-h-screen items-center justify-center px-6 py-12">
        <p className="text-center text-sm text-slate-400">
          Quest failure summary not found. Returning to the quest start is the next step.
        </p>
      </div>
    );
  }

  return (
    <div
      className="flex min-h-screen flex-col items-center px-4 py-12"
      data-quest-failed-screen={summary.result}
      // bug-113 / spec §K-17: surface the reason at the root so
      // automation + QA hooks can read failure reason alongside result.
      data-quest-failed-reason={summary.reason}
    >
      <motion.h1
        className="mb-4 text-center text-5xl font-extrabold tracking-wide md:text-7xl"
        style={{
          background:
            "linear-gradient(135deg, #f87171 0%, #b91c1c 50%, #7f1d1d 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          filter: "drop-shadow(0 0 32px rgba(239, 68, 68, 0.35))",
        }}
        initial={{ opacity: 0, scale: 0.82 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.55, ease: "easeOut" }}
      >
        {summary.result === "defeat" ? "Quest Ended" : "Stalemate"}
      </motion.h1>
      <motion.p
        className="mb-8 text-center text-base text-slate-300 md:text-lg"
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.45, delay: 0.15 }}
      >
        {summary.result === "defeat"
          ? "The run concluded with a defeat you chose to accept."
          : "The run ended in a draw you chose to accept."}
      </motion.p>

      <motion.p
        className="mb-6 text-center text-sm uppercase tracking-[0.18em] text-rose-200"
        data-quest-failed-reason={summary.reason}
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.45, delay: 0.2 }}
      >
        Reason: {formatReason(summary.reason)}
      </motion.p>

      <motion.dl
        className="mb-8 grid w-full max-w-xl grid-cols-2 gap-4 md:grid-cols-3 md:gap-6"
        data-quest-failed-summary={summary.battleId}
        initial={{ opacity: 0, y: 18 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.25 }}
      >
        <SummaryStat label="Result" value={formatResult(summary.result)} />
        <SummaryStat label="Reason" value={formatReason(summary.reason)} />
        <SummaryStat label="Battle" value={summary.battleId} />
        <SummaryStat label="Site" value={summary.siteLabel} />
        <SummaryStat label="Site Id" value={summary.siteId} />
        <SummaryStat
          label="Dreamscape"
          value={summary.dreamscapeIdOrNone ?? "None"}
        />
        <SummaryStat label="Round" value={String(summary.turnNumber)} />
        <SummaryStat label="Player Score" value={String(summary.playerScore)} />
        <SummaryStat label="Enemy Score" value={String(summary.enemyScore)} />
        <SummaryStat label="Mode" value={formatBattleMode(summary.battleMode)} />
      </motion.dl>

      <motion.div
        className="flex gap-4"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.45, delay: 0.4 }}
      >
        <motion.button
          type="button"
          data-quest-failed-action="start-new-run"
          className="cursor-pointer rounded-lg px-8 py-3 text-lg font-bold tracking-wide text-white"
          style={{
            background: "linear-gradient(135deg, #b91c1c 0%, #7f1d1d 100%)",
            border: "2px solid rgba(248, 113, 113, 0.6)",
            boxShadow: "0 0 20px rgba(185, 28, 28, 0.3)",
          }}
          whileHover={{
            boxShadow: "0 0 30px rgba(185, 28, 28, 0.5)",
            scale: 1.05,
          }}
          whileTap={{ scale: 0.97 }}
          onClick={handleStartNewRun}
        >
          Start New Run
        </motion.button>
      </motion.div>
    </div>
  );
}

function SummaryStat({
  label,
  value,
}: {
  label: string;
  value: string;
}) {
  return (
    <div
      className="flex flex-col items-center rounded-lg px-4 py-3"
      style={{
        background: "rgba(255, 255, 255, 0.05)",
        border: "1px solid rgba(248, 113, 113, 0.3)",
      }}
      data-quest-failed-stat={label}
    >
      <span
        className="text-xl font-bold md:text-2xl"
        style={{ color: "#fecaca" }}
      >
        {value}
      </span>
      <span className="mt-1 text-xs uppercase tracking-[0.22em] text-slate-400">
        {label}
      </span>
    </div>
  );
}

function formatResult(result: QuestFailureSummary["result"]): string {
  return result === "defeat" ? "Defeat" : "Draw";
}

function formatBattleMode(mode: QuestFailureSummary["battleMode"]): string {
  return mode === "playable" ? "Playable" : "Auto";
}

function formatReason(reason: QuestFailureSummary["reason"]): string {
  switch (reason) {
    case "score_target_reached":
      return "Score threshold reached";
    case "turn_limit_reached":
      return "Turn limit reached";
    case "forced_result":
      return "Forced result";
  }
}
