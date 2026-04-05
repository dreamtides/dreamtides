import { useEffect, useState } from "react";
import type { BattleView, CardView, DisplayPlayer, GameAction } from "../types/battle";
import { PlayerStatus } from "./PlayerStatus";
import { RankZone } from "./BattlefieldZone";
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
  onReconnect: () => void;
  events: string[];
  disabled: boolean;
  yourTurnCounter: number;
}

function cardsByPosition(cards: CardView[], position: string, player?: DisplayPlayer): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    if (typeof pos === "string") return false;
    if (position in pos) {
      if (player === undefined) return true;
      const val = (pos as Record<string, unknown>)[position];
      // OnBattlefield is now a tuple [player, rank, slot]
      if (Array.isArray(val)) return val[0] === player;
      return val === player;
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

export function BattleScreen({ battle, onAction, onDebugAction, onReconnect, events, disabled, yourTurnCounter }: BattleScreenProps) {
  const [showDebug, setShowDebug] = useState(false);
  const [showVoid, setShowVoid] = useState<DisplayPlayer | null>(null);
  const [showLog, setShowLog] = useState(false);
  const [yourTurnVisible, setYourTurnVisible] = useState(false);
  const ui = battle.interface;

  useEffect(() => {
    if (yourTurnCounter === 0) return;
    setYourTurnVisible(true);
    const timer = setTimeout(() => setYourTurnVisible(false), 2000);
    return () => clearTimeout(timer);
  }, [yourTurnCounter]);

  const isGameOver =
    !disabled &&
    !battle.user.can_act &&
    !ui.primary_action_button &&
    !ui.secondary_action_button &&
    (battle.user.score >= 12 || battle.enemy.score >= 12);

  return (
    <div className="flex flex-col min-h-screen">
      {/* Game over banner */}
      {isGameOver && (
        <div
          className="text-center py-3 text-lg font-bold"
          style={{
            background: battle.user.score >= 12 ? "#065f46" : "#7f1d1d",
            color: "white",
          }}
        >
          {battle.user.score >= 12
            ? `Victory! You won ${battle.user.score} - ${battle.enemy.score}`
            : `Defeat. You lost ${battle.user.score} - ${battle.enemy.score}`}
        </div>
      )}

      {/* Turn info */}
      <div
        className="text-center py-1 text-sm"
        style={{
          background: disabled ? "var(--color-primary)" : "var(--color-surface)",
          borderBottom: "1px solid var(--color-border)",
          color: disabled ? "var(--color-text)" : "var(--color-text-dim)",
          transition: "background 0.2s",
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
        handCount={countCards(battle.cards, "InHand", "Enemy")}
        voidCount={countCards(battle.cards, "InVoid", "Enemy")}
        banishedCount={countCards(battle.cards, "InBanished", "Enemy")}
        onVoidClick={() => setShowVoid("Enemy")}
      />

      {/* Enemy back rank */}
      <RankZone
        label="ENEMY BACK RANK"
        cards={battle.cards}
        player="Enemy"
        rank="Back"
        onAction={onAction}
        disabled={disabled}
      />

      {/* Enemy front rank */}
      <RankZone
        label="ENEMY FRONT RANK"
        cards={battle.cards}
        player="Enemy"
        rank="Front"
        onAction={onAction}
        disabled={disabled}
      />

      {/* Judgment line */}
      <div className="flex items-center justify-center gap-2 py-1">
        <div className="flex-1 border-t border-yellow-600/50" />
        <span className="text-xs text-yellow-600">{"\u26A1"} JUDGMENT LINE {"\u26A1"}</span>
        <div className="flex-1 border-t border-yellow-600/50" />
      </div>

      {/* Your front rank */}
      <RankZone
        label="YOUR FRONT RANK"
        cards={battle.cards}
        player="User"
        rank="Front"
        onAction={onAction}
        disabled={disabled}
      />

      {/* Your back rank */}
      <RankZone
        label="YOUR BACK RANK"
        cards={battle.cards}
        player="User"
        rank="Back"
        onAction={onAction}
        disabled={disabled}
      />

      {/* Stack */}
      <StackZone
        cards={stackCards(battle.cards)}
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
        onVoidClick={() => setShowVoid("User")}
      />

      {/* Hand */}
      <HandZone
        cards={cardsByPosition(battle.cards, "InHand", "User")}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Battle log toggle + Action buttons */}
      <ActionBar
        primaryButton={ui.primary_action_button ?? undefined}
        secondaryButton={ui.secondary_action_button ?? undefined}
        undoButton={ui.undo_button ?? undefined}
        incrementButton={ui.increment_button ?? undefined}
        decrementButton={ui.decrement_button ?? undefined}
        onAction={onAction}
        disabled={disabled}
      />

      {/* Battle log + Dev toggle */}
      <div className="flex justify-center gap-2 py-1">
        {events.length > 0 && (
          <button
            onClick={() => setShowLog((prev) => !prev)}
            className="px-3 py-1 rounded text-xs"
            style={{
              background: "var(--color-surface-light)",
              color: "var(--color-primary-light)",
              border: "1px solid var(--color-border)",
            }}
          >
            Battle Log ({events.length})
          </button>
        )}
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

      {/* Battle log panel */}
      {showLog && events.length > 0 && (
        <div
          className="px-4 py-2"
          style={{
            background: "var(--color-surface-light)",
            borderTop: "1px solid var(--color-border)",
            maxHeight: 200,
            overflowY: "auto",
          }}
        >
          <div
            className="text-xs font-bold mb-1"
            style={{ color: "var(--color-primary-light)" }}
          >
            Battle Log
          </div>
          {events.map((event, i) => (
            <div
              key={i}
              className="text-xs"
              style={{ color: "var(--color-text-dim)" }}
            >
              {"\u2022"} {event}
            </div>
          ))}
        </div>
      )}

      {/* Debug panel */}
      {showDebug && (
        <DebugPanel
          onAction={onDebugAction}
          onReconnect={onReconnect}
        />
      )}

      {/* Card browser (void browsing) */}
      {browserCards(battle.cards).length > 0 && (
        <div
          className="fixed inset-0 flex items-center justify-center z-50"
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
              {ui.browser?.close_button != null && (
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
            <ActionBar
              primaryButton={ui.primary_action_button ?? undefined}
              secondaryButton={ui.secondary_action_button ?? undefined}
              undoButton={ui.undo_button ?? undefined}
              onAction={onAction}
              disabled={disabled}
            />
          </div>
        </div>
      )}

      {/* Void viewer */}
      {showVoid && (
        <div
          className="fixed inset-0 flex items-center justify-center z-50"
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
              <h3 className="font-bold">
                {showVoid === "Enemy" ? "Enemy" : "Your"} Void
              </h3>
              <button
                onClick={() => setShowVoid(null)}
                className="px-3 py-1 rounded text-sm"
                style={{
                  background: "var(--color-surface-light)",
                  border: "1px solid var(--color-border)",
                  cursor: "pointer",
                  color: "var(--color-text)",
                }}
              >
                Close
              </button>
            </div>
            <div className="flex gap-2 flex-wrap justify-center">
              {cardsByPosition(battle.cards, "InVoid", showVoid).length === 0 ? (
                <p style={{ color: "var(--color-text-dim)" }}>No cards in void</p>
              ) : (
                cardsByPosition(battle.cards, "InVoid", showVoid).map((card) => (
                  <CardDisplay key={card.id} card={card} disabled />
                ))
              )}
            </div>
          </div>
        </div>
      )}

      {/* Card order selector (Foresee) */}
      {ui.card_order_selector && cardOrderCards(battle.cards).length > 0 && (
        <div
          className="fixed inset-0 flex items-center justify-center z-50"
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
            <h3 className="font-bold mb-1">Foresee</h3>
            <p className="text-xs mb-3" style={{ color: "var(--color-text-dim)" }}>
              Choose a destination for each card.
            </p>
            <div className="flex flex-col gap-3">
              {cardOrderCards(battle.cards)
                .sort((a, b) => a.position.sorting_key - b.position.sorting_key)
                .map((card) => {
                  const cardId = card.revealed?.actions.can_select_order;
                  return (
                    <div
                      key={card.id}
                      className="flex items-center gap-3 p-2 rounded"
                      style={{ background: "var(--color-surface-light)" }}
                    >
                      <CardDisplay card={card} compact disabled />
                      <div className="flex gap-2">
                        {ui.card_order_selector!.include_deck && (
                          <button
                            onClick={() =>
                              onAction({
                                BattleAction: {
                                  SelectOrderForDeckCard: {
                                    card_id: cardId,
                                    target: { Deck: 0 },
                                  },
                                },
                              })
                            }
                            disabled={disabled || cardId == null}
                            className="px-3 py-1 rounded text-xs font-bold"
                            style={{
                              background: "var(--color-primary)",
                              color: "var(--color-text)",
                              cursor: cardId != null ? "pointer" : "not-allowed",
                              opacity: cardId != null ? 1 : 0.5,
                            }}
                          >
                            {"\u2192"} Top of Deck
                          </button>
                        )}
                        {ui.card_order_selector!.include_void && (
                          <button
                            onClick={() =>
                              onAction({
                                BattleAction: {
                                  SelectOrderForDeckCard: {
                                    card_id: cardId,
                                    target: "Void",
                                  },
                                },
                              })
                            }
                            disabled={disabled || cardId == null}
                            className="px-3 py-1 rounded text-xs font-bold"
                            style={{
                              background: "var(--color-surface)",
                              color: "var(--color-text)",
                              border: "1px solid var(--color-border)",
                              cursor: cardId != null ? "pointer" : "not-allowed",
                              opacity: cardId != null ? 1 : 0.5,
                            }}
                          >
                            {"\u2192"} Void
                          </button>
                        )}
                      </div>
                    </div>
                  );
                })}
            </div>
            <ActionBar
              primaryButton={ui.primary_action_button ?? undefined}
              secondaryButton={ui.secondary_action_button ?? undefined}
              undoButton={ui.undo_button ?? undefined}
              onAction={onAction}
              disabled={disabled}
            />
          </div>
        </div>
      )}

      {/* Your Turn popup */}
      {yourTurnVisible && !ui.card_order_selector && !ui.screen_overlay && browserCards(battle.cards).length === 0 && !showVoid && (
        <div
          className="fixed left-4 top-1/2 -translate-y-1/2 pointer-events-none"
          style={{ zIndex: 45 }}
        >
          <div
            className="text-lg font-bold px-4 py-2 rounded-lg"
            style={{
              background: "rgba(0, 0, 0, 0.85)",
              border: "2px solid var(--color-primary)",
              color: "var(--color-primary-light)",
              animation: "fadeInOut 2s ease-in-out",
            }}
          >
            Your Turn
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
