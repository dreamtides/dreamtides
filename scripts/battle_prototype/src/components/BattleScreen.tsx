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

type CardOrderTarget = "Deck" | "Void";

interface BattleScreenProps {
  battle: BattleView;
  onAction: (action: GameAction) => void;
  onDebugAction: (action: GameAction) => void;
  onReconnect: (userGoesSecond?: boolean) => void;
  events: string[];
  disabled: boolean;
  yourTurnCounter: number;
  judgmentPause?: boolean;
  onContinueFromJudgment?: () => void;
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

function cardOrderTarget(card: CardView): CardOrderTarget {
  const pos = card.position.position;
  if (typeof pos !== "string" && "CardOrderSelector" in pos) {
    return (pos as { CardOrderSelector: string }).CardOrderSelector === "Void"
      ? "Void"
      : "Deck";
  }
  return "Deck";
}

export function BattleScreen({ battle, onAction, onDebugAction, onReconnect, events, disabled, yourTurnCounter, judgmentPause, onContinueFromJudgment }: BattleScreenProps) {
  const [showDebug, setShowDebug] = useState(false);
  const [showVoid, setShowVoid] = useState<DisplayPlayer | null>(null);
  const [showLog, setShowLog] = useState(false);
  const [yourTurnVisible, setYourTurnVisible] = useState(false);
  const [optimisticCardOrderTargets, setOptimisticCardOrderTargets] = useState<Record<string, CardOrderTarget>>({});
  const ui = battle.interface;

  useEffect(() => {
    if (yourTurnCounter === 0) return;
    setYourTurnVisible(true);
    const timer = setTimeout(() => setYourTurnVisible(false), 2000);
    return () => clearTimeout(timer);
  }, [yourTurnCounter]);

  // Auto-dismiss judgment pause when there's an active prompt so the user
  // can interact with the prompt (e.g. Foresee) immediately.
  useEffect(() => {
    if (judgmentPause && (ui.card_order_selector || ui.screen_overlay)) {
      onContinueFromJudgment?.();
    }
  }, [judgmentPause, ui.card_order_selector, ui.screen_overlay, onContinueFromJudgment]);

  useEffect(() => {
    if (!ui.card_order_selector) {
      setOptimisticCardOrderTargets({});
      return;
    }
    const selectorIds = new Set(cardOrderCards(battle.cards).map((card) => card.id));
    setOptimisticCardOrderTargets((current) => {
      let changed = false;
      const next: Record<string, CardOrderTarget> = {};
      for (const [cardId, target] of Object.entries(current)) {
        if (selectorIds.has(cardId)) {
          next[cardId] = target;
        } else {
          changed = true;
        }
      }
      return changed ? next : current;
    });
  }, [battle.cards, ui.card_order_selector]);

  const isGameOver = battle.game_over;
  const selectorCards = cardOrderCards(battle.cards)
    .sort((a, b) => a.position.sorting_key - b.position.sorting_key);

  const effectiveCardOrderTarget = (card: CardView): CardOrderTarget =>
    optimisticCardOrderTargets[card.id] ?? cardOrderTarget(card);

  const selectCardOrderTarget = (card: CardView, target: CardOrderTarget) => {
    const cardId = card.revealed?.actions.can_select_order;
    if (disabled || cardId == null) return;
    setOptimisticCardOrderTargets((current) => ({ ...current, [card.id]: target }));
    onAction({
      BattleAction: {
        SelectOrderForDeckCard: {
          card_id: cardId,
          target: target === "Void" ? "Void" : { Deck: 0 },
        },
      },
    });
  };

  const deckOrderCards = selectorCards.filter((card) => effectiveCardOrderTarget(card) === "Deck");
  const voidOrderCards = selectorCards.filter((card) => effectiveCardOrderTarget(card) === "Void");

  return (
    <div className="flex flex-col h-screen overflow-y-auto" style={{ paddingBottom: 48 }}>
      {/* Game over banner */}
      {isGameOver && (
        <div
          className="text-center py-3 text-lg font-bold"
          style={{
            background: battle.user.score >= battle.enemy.score ? "#065f46" : "#7f1d1d",
            color: "white",
          }}
        >
          {battle.user.score >= battle.enemy.score
            ? `Victory! You won ${battle.user.score} - ${battle.enemy.score}`
            : `Defeat. You lost ${battle.user.score} - ${battle.enemy.score}`}
          <button
            onClick={() => onReconnect()}
            className="ml-4 px-4 py-1 rounded text-sm font-bold"
            style={{
              background: "rgba(255, 255, 255, 0.2)",
              color: "white",
              border: "1px solid rgba(255, 255, 255, 0.4)",
              cursor: "pointer",
            }}
          >
            Play Again
          </button>
        </div>
      )}

      {/* Status bar */}
      <div
        className="flex items-center justify-between px-3 py-1 text-xs"
        style={{
          background: "var(--color-surface)",
          borderBottom: "1px solid var(--color-border)",
          color: "var(--color-text-dim)",
        }}
      >
        <span className="flex items-center gap-2">
          <span>Turn {battle.turn_number}</span>
          <span style={{ color: "var(--color-primary-light)" }}>
            AI: {battle.opponent_ai_label}
          </span>
        </span>
        <span
          className="font-bold"
          style={{
            color: judgmentPause
              ? "var(--color-gold)"
              : battle.user.can_act
                ? "var(--color-primary-light)"
                : "var(--color-text-dim)",
          }}
        >
          {judgmentPause
            ? "\u26A1 Judgment"
            : battle.game_over
              ? "Game Over"
              : battle.user.can_act
                ? battle.user.turn_indicator
                  ? "Your Turn"
                  : "Enemy Turn — Respond"
                : "Enemy Turn — waiting\u2026"}
        </span>
        <span>
          {battle.user.score}–{battle.enemy.score}
        </span>
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
      <div className="flex items-center justify-center gap-2 py-0.5">
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
        <div
          className="fixed inset-x-0 z-40 overflow-y-auto"
          style={{
            bottom: 56,
            maxHeight: "min(45vh, calc(100vh - 8rem))",
          }}
        >
          <DebugPanel
            onAction={onDebugAction}
            onReconnect={onReconnect}
          />
        </div>
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
      {ui.card_order_selector && selectorCards.length > 0 && (
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
            <div className="grid gap-3 md:grid-cols-2">
              {ui.card_order_selector!.include_deck && (
                <div
                  className="rounded-lg p-3"
                  style={{
                    background: "rgba(59, 130, 246, 0.08)",
                    border: "1px solid rgba(59, 130, 246, 0.25)",
                  }}
                >
                  <div className="mb-2">
                    <div className="font-bold text-sm" style={{ color: "var(--color-primary-light)" }}>
                      Top of Deck
                    </div>
                    <div className="text-xs" style={{ color: "var(--color-text-dim)" }}>
                      Cards that will stay on top of your deck.
                    </div>
                  </div>
                  <div className="flex flex-col gap-3">
                    {deckOrderCards.length === 0 && (
                      <div className="rounded p-3 text-xs text-center" style={{ background: "rgba(15, 23, 42, 0.35)", color: "var(--color-text-dim)" }}>
                        No cards assigned here yet.
                      </div>
                    )}
                    {deckOrderCards.map((card) => {
                      const cardId = card.revealed?.actions.can_select_order;
                      return (
                        <div
                          key={card.id}
                          className="flex items-center gap-3 p-2 rounded"
                          style={{ background: "var(--color-surface-light)" }}
                        >
                          <CardDisplay card={card} compact disabled />
                          <div className="flex gap-2 flex-wrap">
                            <button
                              onClick={() => selectCardOrderTarget(card, "Deck")}
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
                            {ui.card_order_selector!.include_void && (
                              <button
                                onClick={() => selectCardOrderTarget(card, "Void")}
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
                </div>
              )}
              {ui.card_order_selector!.include_void && (
                <div
                  className="rounded-lg p-3"
                  style={{
                    background: "rgba(168, 85, 247, 0.08)",
                    border: "1px solid rgba(168, 85, 247, 0.25)",
                  }}
                >
                  <div className="mb-2">
                    <div className="font-bold text-sm" style={{ color: "#d8b4fe" }}>
                      Void
                    </div>
                    <div className="text-xs" style={{ color: "var(--color-text-dim)" }}>
                      Cards that will be put into your void.
                    </div>
                  </div>
                  <div className="flex flex-col gap-3">
                    {voidOrderCards.length === 0 && (
                      <div className="rounded p-3 text-xs text-center" style={{ background: "rgba(15, 23, 42, 0.35)", color: "var(--color-text-dim)" }}>
                        No cards assigned here yet.
                      </div>
                    )}
                    {voidOrderCards.map((card) => {
                      const cardId = card.revealed?.actions.can_select_order;
                      return (
                        <div
                          key={card.id}
                          className="flex items-center gap-3 p-2 rounded"
                          style={{ background: "var(--color-surface-light)" }}
                        >
                          <CardDisplay card={card} compact disabled />
                          <div className="flex gap-2 flex-wrap">
                            {ui.card_order_selector!.include_deck && (
                              <button
                                onClick={() => selectCardOrderTarget(card, "Deck")}
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
                            <button
                              onClick={() => selectCardOrderTarget(card, "Void")}
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
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
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

      {/* Judgment pause indicator */}
      {judgmentPause && (
        <div
          className="fixed right-4 top-1/2 -translate-y-1/2 z-50"
        >
          <div
            className="rounded-lg p-3 text-center"
            style={{
              background: "var(--color-surface)",
              border: "1px solid var(--color-gold)",
              boxShadow: "0 2px 12px rgba(0,0,0,0.4)",
            }}
          >
            <div
              className="text-sm font-bold mb-1"
              style={{ color: "var(--color-gold)" }}
            >
              {"\u26A1"} Judgment
            </div>
            <div className="text-xs mb-2" style={{ color: "var(--color-text-dim)" }}>
              You <span style={{ color: "var(--color-gold)" }}>{battle.user.score}</span> — <span style={{ color: "var(--color-gold)" }}>{battle.enemy.score}</span> Enemy
            </div>
            <button
              onClick={onContinueFromJudgment}
              className="px-4 py-1 rounded text-xs font-bold"
              style={{
                background: "var(--color-primary)",
                color: "var(--color-text)",
                border: "1px solid var(--color-primary-light)",
                cursor: "pointer",
              }}
            >
              Continue
            </button>
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
            {battle.user.turn_indicator ? "Your Turn" : "Respond"}
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
