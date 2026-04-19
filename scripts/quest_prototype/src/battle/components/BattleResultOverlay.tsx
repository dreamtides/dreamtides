import type { CSSProperties } from "react";

import type { FrozenCardData } from "../../types/cards";
import { BattleCardView, battleCardVisualFromReward } from "./BattleCardView";
import type { BattleResult } from "../types";

export function BattleResultOverlay({
  result,
  rewardCards,
  selectedRewardIndex,
  rewardLocked = false,
  onChooseReward,
  onConfirmReward,
  onDismissInspect,
  onReset,
}: {
  result: BattleResult;
  rewardCards?: readonly FrozenCardData[];
  selectedRewardIndex?: number | null;
  rewardLocked?: boolean;
  onChooseReward?: (index: number) => void;
  onConfirmReward?: () => void;
  onDismissInspect: () => void;
  onReset?: () => void;
}) {
  const title = result === "victory" ? "Victory." : result === "defeat" ? "Defeat." : "Draw.";

  return (
    <div
      className="result-overlay"
      data-battle-overlay={result}
      data-battle-result-overlay={result}
    >
      <div className="panel">
        <h1>{title}</h1>
        {result === "victory" ? <p>Choose a reward card.</p> : null}
        {result === "victory" && rewardCards !== undefined ? (
          <div className="reward-picker">
            {rewardCards.map((card, index) => (
              <div
                key={`${card.cardNumber}-${String(index)}`}
                className={`reward ${selectedRewardIndex === index ? "chosen" : ""}`}
                onClick={() => onChooseReward?.(index)}
              >
                <BattleCardView
                  data={battleCardVisualFromReward(card)}
                  selected={selectedRewardIndex === index}
                  style={{ "--card-w": "108px", "--card-h": "150px" } as CSSProperties}
                />
              </div>
            ))}
          </div>
        ) : null}
        <div className="actions">
          <button
            type="button"
            data-battle-action="dismiss-result"
            data-battle-result-action="dismiss"
            className="btn ghost"
            onClick={onDismissInspect}
          >
            Keep inspecting
          </button>
          {result === "victory" ? (
            <button
              type="button"
              data-battle-action="confirm-reward"
              data-battle-result-action="confirm-reward"
              className="btn primary"
              disabled={selectedRewardIndex === null || selectedRewardIndex === undefined || rewardLocked}
              onClick={onConfirmReward}
            >
              Confirm reward
            </button>
          ) : (
            <button
              type="button"
              data-battle-action="reset-run"
              data-battle-result-action="reset-run"
              className="btn danger"
              onClick={onReset}
            >
              Reset run…
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
