import { useEffect } from "react";
import type { BattleCommand } from "../debug/commands";
import type { BattleMutableState, BattleSide } from "../types";

export function BattleActionBar({
  canEndTurn,
  futureCount,
  hideDebugSection: _hideDebugSection = false,
  historyCount,
  isInteractionLocked = false,
  isBattleLogOpen: _isBattleLogOpen,
  isDesktopInspectorLayout: _isDesktopInspectorLayout,
  isInspectorDrawerOpen: _isInspectorDrawerOpen,
  state: _state,
  onCommand,
  onOpenForesee: _onOpenForesee,
  onRedo,
  onToggleBattleLog,
  onToggleInspector: _onToggleInspector,
  onUndo,
}: {
  canEndTurn: boolean;
  futureCount: number;
  hideDebugSection?: boolean;
  historyCount: number;
  isInteractionLocked?: boolean;
  isBattleLogOpen: boolean;
  isDesktopInspectorLayout: boolean;
  isInspectorDrawerOpen: boolean;
  state: BattleMutableState;
  onCommand: (command: BattleCommand) => void;
  onOpenForesee: (side: BattleSide, count: number) => void;
  onRedo: () => void;
  onToggleBattleLog: () => void;
  onToggleInspector: () => void;
  onUndo: () => void;
}) {
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent): void {
      const target = event.target as HTMLElement | null;
      if (
        isInteractionLocked ||
        target !== null &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.tagName === "SELECT" || target.isContentEditable)
      ) {
        return;
      }
      const mod = event.metaKey || event.ctrlKey;
      if (!mod && event.key.toLowerCase() === "e") {
        if (!canEndTurn) {
          return;
        }
        event.preventDefault();
        onCommand({ id: "END_TURN", sourceSurface: "action-bar" });
        return;
      }
      if (mod && !event.shiftKey && event.key.toLowerCase() === "z") {
        if (historyCount === 0) {
          return;
        }
        event.preventDefault();
        onUndo();
        return;
      }
      if (mod && event.shiftKey && event.key.toLowerCase() === "z") {
        if (futureCount === 0) {
          return;
        }
        event.preventDefault();
        onRedo();
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [canEndTurn, futureCount, historyCount, isInteractionLocked, onCommand, onRedo, onUndo]);

  return (
    <section data-battle-region="action-bar" className="actionbar">
      <div className="group">
        <button
          type="button"
          data-battle-action="undo"
          className="btn ghost sm"
          onClick={onUndo}
          disabled={historyCount === 0}
        >
          Undo
        </button>
        <button
          type="button"
          data-battle-action="redo"
          className="btn ghost sm"
          onClick={onRedo}
          disabled={futureCount === 0}
        >
          Redo
        </button>
        <button
          type="button"
          data-battle-action="toggle-log"
          className="btn ghost sm"
          onClick={onToggleBattleLog}
        >
          Log
        </button>
      </div>
      <div className="group">
        <button
          type="button"
          data-battle-action="skip-to-rewards"
          className="btn ghost sm"
          onClick={() => onCommand({ id: "SKIP_TO_REWARDS", sourceSurface: "action-bar" })}
        >
          Skip to rewards
        </button>
        <button
          type="button"
          data-battle-action="end-turn"
          className="btn primary"
          onClick={() => onCommand({ id: "END_TURN", sourceSurface: "action-bar" })}
          disabled={!canEndTurn}
        >
          End turn
        </button>
      </div>
    </section>
  );
}
