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
  {
    label: "Sentry\u2192Enemy Front 0",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "Two",
            card: "a1b2c3d4-1111-4000-8000-000000000001",
            position: 0,
          },
        },
      },
    },
  },
  {
    label: "Knight\u2192Enemy Front 1",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "Two",
            card: "a1b2c3d4-3333-4000-8000-000000000003",
            position: 1,
          },
        },
      },
    },
  },
  {
    label: "Titan\u2192Enemy Front 2",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "Two",
            card: "a1b2c3d4-5555-4000-8000-000000000005",
            position: 2,
          },
        },
      },
    },
  },
  {
    label: "Colossus\u2192Enemy Front 3",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "Two",
            card: "a1b2c3d4-7777-4000-8000-000000000006",
            position: 3,
          },
        },
      },
    },
  },
  {
    label: "Scout\u2192Your Front 0",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "One",
            card: "a1b2c3d4-2222-4000-8000-000000000002",
            position: 0,
          },
        },
      },
    },
  },
  {
    label: "Warrior\u2192Your Front 1",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "One",
            card: "a1b2c3d4-4444-4000-8000-000000000004",
            position: 1,
          },
        },
      },
    },
  },
  {
    label: "Knight\u2192Your Front 0",
    action: {
      BattleAction: {
        Debug: {
          AddCardToFrontRank: {
            player: "One",
            card: "a1b2c3d4-3333-4000-8000-000000000003",
            position: 0,
          },
        },
      },
    },
  },
  {
    label: "Vanilla\u2192Hand",
    action: {
      BattleAction: {
        Debug: {
          AddCardToHand: {
            player: "One",
            card: "a1b2c3d4-3333-4000-8000-000000000003",
          },
        },
      },
    },
  },
  {
    label: "Skip to Judgment",
    action: {
      BattleAction: {
        Debug: "SkipToJudgment",
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
