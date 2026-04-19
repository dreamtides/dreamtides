import type { SiteState } from "../../types/quest";
import { formatPhaseLabel, formatSideLabel } from "../ui/format";
import type { BattleMutableState, BattleResult, BattleSide } from "../types";

export function BattleStatusBar({
  activeSide,
  battleId,
  enemyName,
  enemyScore,
  futureCount,
  hasAiOpponent = true,
  historyCount,
  phase,
  playerScore,
  result,
  roundNumber,
  siteType,
}: {
  activeSide: BattleSide;
  battleId: string;
  enemyName: string;
  enemyScore: number;
  futureCount: number;
  hasAiOpponent?: boolean;
  historyCount: number;
  phase: BattleMutableState["phase"];
  playerScore: number;
  result: BattleResult | null;
  roundNumber: number;
  siteType: SiteState["type"];
}) {
  const phaseLabel = formatPhaseLabel(phase);
  const sideLabel = result === null ? formatSideLabel(activeSide) : "Battle Over";
  const phaseSteps = [
    { id: "startOfTurn", label: "Start" },
    { id: "judgment", label: "Judgment" },
    { id: "draw", label: "Draw" },
    { id: "main", label: "Main" },
    { id: "endOfTurn", label: "End" },
  ] as const;

  return (
    <section data-battle-region="status-bar" className="topbar">
      <div className="left">
        <span data-battle-stat="round-number" className="turn-badge">
          Turn {String(roundNumber)}
        </span>
        <span className={`turn-owner-pill ${activeSide === "player" ? "you" : "them"}`}>
          {sideLabel}
        </span>
      </div>
      <div className="status-focus">
        <div className="status-focus-copy">
          <span className="status-focus-label">Active Phase</span>
          <strong
            data-battle-stat="phase"
            data-battle-phase={phase}
            className={`status-focus-phase ${activeSide === "player" ? "you" : "them"}`}
          >
            {phaseLabel}
          </strong>
        </div>
        <div className="phase-track" aria-hidden="true">
          {phaseSteps.map((step) => (
            <span
              key={step.id}
              className={`phase-track-step ${phase === step.id ? "active" : ""}`}
            >
              {step.label}
            </span>
          ))}
        </div>
      </div>
      <div className="right">
        <div
          data-battle-stat="score-summary"
          data-battle-score-summary={`${String(playerScore)}:${String(enemyScore)}`}
          className="score-summary"
        >
          <span
            data-battle-score-side="player"
            className={activeSide === "player" ? "active" : ""}
          >
            {String(playerScore)}
          </span>
          <span className="vs">—</span>
          <span
            data-battle-score-side="enemy"
            className={activeSide === "enemy" ? "active" : ""}
          >
            {String(enemyScore)}
          </span>
        </div>
      </div>
      <div hidden>
        <div data-battle-status-meta="battle-id">{battleId}</div>
        <div data-battle-status-meta="enemy-name">{enemyName}</div>
        <div data-battle-status-meta="site-type">{siteType}</div>
        <div data-battle-status-meta="has-ai">{String(hasAiOpponent)}</div>
        <div data-battle-status-meta="history">{String(historyCount)}</div>
        <div data-battle-status-meta="future">{String(futureCount)}</div>
        <div data-battle-status-meta="result">{result ?? "none"}</div>
      </div>
    </section>
  );
}
