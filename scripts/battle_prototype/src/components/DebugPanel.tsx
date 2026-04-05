import type { GameAction } from "../types/battle";

interface DebugPanelProps {
  onAction: (action: GameAction) => void;
  onReconnect: () => void;
}

interface DebugButtonConfig {
  label: string;
  action: GameAction;
}

const DEBUG_BUTTONS: DebugButtonConfig[] = [
  {
    label: "99 Energy",
    action: {
      BattleAction: {
        Debug: { SetEnergy: { player: "One", energy: 99 } },
      },
    },
  },
  {
    label: "Draw Card",
    action: {
      BattleAction: {
        Debug: { DrawCard: { player: "One" } },
      },
    },
  },
  {
    label: "Enemy Character",
    action: {
      BattleAction: {
        Debug: {
          AddCardToBattlefield: {
            player: "Two",
            card: "00000000-0000-0000-0000-000000000000",
          },
        },
      },
    },
  },
  {
    label: "Opponent Continue",
    action: {
      BattleAction: {
        Debug: "OpponentContinue",
      },
    },
  },
  {
    label: "Deck \u2192 1",
    action: {
      BattleAction: {
        Debug: { SetCardsRemainingInDeck: { player: "One", cards: 1 } },
      },
    },
  },
  {
    label: "0 Energy",
    action: {
      BattleAction: {
        Debug: { SetEnergy: { player: "One", energy: 0 } },
      },
    },
  },
];

export function DebugPanel({
  onAction,
  onReconnect,
}: DebugPanelProps) {
  return (
    <div
      className="p-4 flex flex-col gap-3"
      style={{
        background: "var(--color-surface)",
        borderTop: "2px solid var(--color-primary)",
      }}
    >
      <h3 className="font-bold text-sm" style={{ color: "var(--color-primary-light)" }}>
        Debug Panel
      </h3>

      {/* Restart */}
      <div className="flex gap-2 items-center">
        <button
          onClick={() => onReconnect()}
          className="px-3 py-1 rounded text-sm"
          style={{
            background: "var(--color-primary)",
            color: "var(--color-text)",
            cursor: "pointer",
          }}
        >
          Restart Battle (Core11)
        </button>
      </div>

      {/* Debug action buttons */}
      <div className="flex gap-2 flex-wrap">
        {DEBUG_BUTTONS.map((btn) => (
          <button
            key={btn.label}
            onClick={() => onAction(btn.action)}
            className="px-3 py-1 rounded text-xs"
            style={{
              background: "var(--color-surface-light)",
              color: "var(--color-text)",
              border: "1px solid var(--color-border)",
              cursor: "pointer",
            }}
          >
            {btn.label}
          </button>
        ))}
      </div>
    </div>
  );
}
