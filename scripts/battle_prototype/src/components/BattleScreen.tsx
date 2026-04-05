import { useState } from "react";
import type { BattleView, CardView, DisplayPlayer, GameAction, TestDeckName } from "../types/battle";
import { PlayerStatus } from "./PlayerStatus";
import { BattlefieldZone } from "./BattlefieldZone";
import { StackZone } from "./StackZone";
import { HandZone } from "./HandZone";
import { ActionBar } from "./ActionBar";
import { OverlayPrompt } from "./OverlayPrompt";
import { DebugPanel } from "./DebugPanel";
import { CardDisplay } from "./CardDisplay";

interface BattleScreenProps {
  battle: BattleView;
  onAction: (action: GameAction) => void;
  onDebugAction: (action: GameAction) => void;
  onReconnect: (deck: TestDeckName) => void;
  disabled: boolean;
}

function cardsByPosition(cards: CardView[], position: string, player?: DisplayPlayer): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    if (typeof pos === "string") return false;
    if (position in pos) {
      if (player === undefined) return true;
      return (pos as Record<string, unknown>)[position] === player;
    }
    return false;
  });
}

function countCards(cards: CardView[], position: string, player: DisplayPlayer): number {
  return cardsByPosition(cards, position, player).length;
}

function stackCards(cards: CardView[]): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    return typeof pos !== "string" && "OnStack" in pos;
  });
}

function browserCards(cards: CardView[]): CardView[] {
  return cards.filter((c) => c.position.position === "Browser");
}

function cardOrderCards(cards: CardView[]): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    return typeof pos !== "string" && "CardOrderSelector" in pos;
  });
}

export function BattleScreen({ battle, onAction, onDebugAction, onReconnect, disabled }: BattleScreenProps) {
  const [showDebug, setShowDebug] = useState(false);
  const ui = battle.interface;

  return (
    <div className="flex flex-col min-h-screen">
      {/* Turn info */}
      <div
        className="text-center py-1 text-sm"
        style={{
          background: "var(--color-surface)",
          borderBottom: "1px solid var(--color-border)",
          color: "var(--color-text-dim)",
        }}
      >
        Turn {battle.turn_number}
        {disabled && " — waiting..."}
      </div>

      {/* Enemy status */}
      <PlayerStatus
        player={battle.enemy}
        label="Enemy"
        deckCount={countCards(battle.cards, "InDeck", "Enemy")}
        voidCount={countCards(battle.cards, "InVoid", "Enemy")}
        banishedCount={countCards(battle.cards, "InBanished", "Enemy")}
      />

      {/* Enemy battlefield */}
      <BattlefieldZone
        cards={cardsByPosition(battle.cards, "OnBattlefield", "Enemy")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Stack */}
      <StackZone
        cards={stackCards(battle.cards)}
        onAction={onAction}
        disabled={disabled}
      />

      {/* User battlefield */}
      <BattlefieldZone
        cards={cardsByPosition(battle.cards, "OnBattlefield", "User")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* User status */}
      <PlayerStatus
        player={battle.user}
        label="You"
        deckCount={countCards(battle.cards, "InDeck", "User")}
        voidCount={countCards(battle.cards, "InVoid", "User")}
        banishedCount={countCards(battle.cards, "InBanished", "User")}
      />

      {/* Hand */}
      <HandZone
        cards={cardsByPosition(battle.cards, "InHand", "User")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Action buttons */}
      <ActionBar
        primaryButton={ui.primary_action_button ?? undefined}
        secondaryButton={ui.secondary_action_button ?? undefined}
        undoButton={ui.undo_button ?? undefined}
        incrementButton={ui.increment_button ?? undefined}
        decrementButton={ui.decrement_button ?? undefined}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Dev toggle button */}
      <div className="flex justify-center py-1">
        <button
          onClick={() => setShowDebug((prev) => !prev)}
          className="px-3 py-1 rounded text-xs"
          style={{
            background: "var(--color-surface-light)",
            color: "var(--color-text-dim)",
            border: "1px solid var(--color-border)",
          }}
        >
          {showDebug ? "Hide Debug" : "Show Debug"}
        </button>
      </div>

      {/* Debug panel */}
      {showDebug && (
        <DebugPanel
          onAction={onDebugAction}
          onReconnect={onReconnect}
        />
      )}

      {/* Card browser (void browsing) */}
      {ui.browser && browserCards(battle.cards).length > 0 && (
        <div
          className="fixed inset-0 flex items-center justify-center z-40"
          style={{ background: "rgba(0, 0, 0, 0.7)" }}
        >
          <div
            className="rounded-lg p-4 max-w-2xl w-full mx-4"
            style={{
              background: "var(--color-surface)",
              border: "1px solid var(--color-border)",
              maxHeight: "80vh",
              overflowY: "auto",
            }}
          >
            <div className="flex justify-between items-center mb-3">
              <h3 className="font-bold">Card Browser</h3>
              {ui.browser.close_button != null && (
                <button
                  onClick={() => onAction(ui.browser!.close_button!)}
                  className="px-3 py-1 rounded text-sm"
                  style={{
                    background: "var(--color-surface-light)",
                    border: "1px solid var(--color-border)",
                    cursor: "pointer",
                  }}
                >
                  Close
                </button>
              )}
            </div>
            <div className="flex gap-2 flex-wrap justify-center">
              {browserCards(battle.cards).map((card) => (
                <CardDisplay
                  key={card.id}
                  card={card}
                  onAction={onAction}
                  disabled={disabled}
                />
              ))}
            </div>
          </div>
        </div>
      )}

      {/* Card order selector (Foresee) */}
      {ui.card_order_selector && cardOrderCards(battle.cards).length > 0 && (
        <div
          className="fixed inset-0 flex items-center justify-center z-40"
          style={{ background: "rgba(0, 0, 0, 0.7)" }}
        >
          <div
            className="rounded-lg p-4 max-w-2xl w-full mx-4"
            style={{
              background: "var(--color-surface)",
              border: "1px solid var(--color-border)",
            }}
          >
            <h3 className="font-bold mb-3">Reorder Cards (click to select position)</h3>
            <div className="flex gap-2 flex-wrap justify-center">
              {cardOrderCards(battle.cards)
                .sort((a, b) => a.position.sorting_key - b.position.sorting_key)
                .map((card) => (
                  <CardDisplay
                    key={card.id}
                    card={card}
                    onAction={onAction}
                    disabled={disabled}
                  />
                ))}
            </div>
          </div>
        </div>
      )}

      {/* Overlay */}
      {ui.screen_overlay && (
        <OverlayPrompt
          overlay={ui.screen_overlay}
          onAction={onAction}
          disabled={disabled}
        />
      )}
    </div>
  );
}
