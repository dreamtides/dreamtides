import { useCallback, useEffect, useMemo } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { DraftState } from "../types/draft";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { extractBotSummaries, type BotSummary } from "./debug-helpers";

/** Props for the DebugScreen component. */
interface DebugScreenProps {
  isOpen: boolean;
  onClose: () => void;
  draftState: DraftState | null;
  cardDatabase: Map<number, CardData>;
}

/**
 * Full-screen overlay showing AI bot draft intelligence data.
 * Displays each bot's tide preferences, drafted cards, and preference vectors.
 */
export function DebugScreen({
  isOpen,
  onClose,
  draftState,
  cardDatabase,
}: DebugScreenProps) {
  const botSummaries = useMemo(
    () => extractBotSummaries(draftState, cardDatabase),
    [draftState, cardDatabase],
  );

  const handleClose = useCallback(() => {
    onClose();
  }, [onClose]);

  useEffect(() => {
    if (!isOpen) return undefined;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        handleClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, handleClose]);

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          key="debug-screen-backdrop"
          className="fixed inset-0 z-[60] flex flex-col"
          style={{ backgroundColor: "rgba(5, 2, 10, 0.95)" }}
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 40 }}
          transition={{ duration: 0.3 }}
        >
          {/* Header */}
          <div
            className="flex items-center justify-between px-4 py-3 md:px-6"
            style={{
              borderBottom: "1px solid rgba(124, 58, 237, 0.3)",
              background:
                "linear-gradient(180deg, rgba(10, 6, 18, 0.95) 0%, rgba(10, 6, 18, 0.8) 100%)",
            }}
          >
            <h2
              className="text-lg font-bold md:text-xl"
              style={{ color: "#e2e8f0" }}
            >
              Debug: AI Draft Intelligence
            </h2>
            <button
              className="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full text-lg transition-colors"
              style={{
                background: "rgba(255, 255, 255, 0.1)",
                color: "#e2e8f0",
              }}
              onClick={handleClose}
              aria-label="Close debug screen"
            >
              {"\u2715"}
            </button>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto px-4 py-4 md:px-6">
            {botSummaries.length === 0 ? (
              <div className="flex h-full items-center justify-center">
                <p className="text-sm opacity-40">
                  No draft data available yet.
                </p>
              </div>
            ) : (
              <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                {botSummaries.map((bot) => (
                  <BotCard
                    key={bot.seatIndex}
                    bot={bot}
                    cardDatabase={cardDatabase}
                  />
                ))}
              </div>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/** Card displaying a single bot's draft intelligence. */
function BotCard({
  bot,
  cardDatabase: _cardDatabase,
}: {
  bot: BotSummary;
  cardDatabase: Map<number, CardData>;
}) {
  const primaryColor =
    bot.primaryTide !== null ? TIDE_COLORS[bot.primaryTide] : "#6b7280";

  return (
    <div
      className="rounded-lg p-4"
      style={{
        background: "rgba(10, 6, 18, 0.6)",
        border: `1px solid ${primaryColor}40`,
      }}
    >
      {/* Bot header */}
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div
            className="flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold"
            style={{
              background: `${primaryColor}20`,
              border: `2px solid ${primaryColor}60`,
              color: primaryColor,
            }}
          >
            {String(bot.seatIndex)}
          </div>
          <div>
            <span
              className="text-sm font-bold"
              style={{ color: "#e2e8f0" }}
            >
              Seat {String(bot.seatIndex)}
            </span>
            <div className="flex items-center gap-1.5">
              {bot.primaryTide !== null && (
                <TideBadge tide={bot.primaryTide} size="primary" />
              )}
              {bot.secondaryTide !== null && (
                <TideBadge tide={bot.secondaryTide} size="secondary" />
              )}
              {bot.primaryTide === null && (
                <span className="text-[10px] opacity-40">
                  No tide preference
                </span>
              )}
            </div>
          </div>
        </div>
        <span
          className="text-xs opacity-50"
          style={{ color: "#e2e8f0" }}
        >
          {String(bot.totalCards)} cards
        </span>
      </div>

      {/* Preference weights bar chart */}
      <PreferenceBar weights={bot.preferenceWeights} />

      {/* Cards by tide summary */}
      {bot.totalCards > 0 && (
        <div className="mt-3">
          <h4
            className="mb-1.5 text-[10px] font-bold uppercase tracking-wider"
            style={{ color: "#a855f7" }}
          >
            Drafted Cards
          </h4>
          <TideCardCounts cardsByTide={bot.cardsByTide} />
        </div>
      )}

      {/* Card name list */}
      {bot.draftedCards.length > 0 && (
        <div className="mt-2">
          <div
            className="max-h-32 overflow-y-auto rounded p-2"
            style={{
              background: "rgba(0, 0, 0, 0.3)",
              border: "1px solid rgba(124, 58, 237, 0.1)",
            }}
          >
            <div className="flex flex-wrap gap-1">
              {bot.draftedCards.map((card, i) => (
                <span
                  key={`${String(card.cardNumber)}-${String(i)}`}
                  className="rounded-full px-2 py-0.5 text-[10px] font-medium"
                  style={{
                    background: `${TIDE_COLORS[card.tide]}15`,
                    border: `1px solid ${TIDE_COLORS[card.tide]}30`,
                    color: TIDE_COLORS[card.tide],
                  }}
                >
                  {card.name}
                </span>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

/** Small tide icon+name badge. */
function TideBadge({
  tide,
  size,
}: {
  tide: Tide;
  size: "primary" | "secondary";
}) {
  const isPrimary = size === "primary";
  return (
    <div
      className="flex items-center gap-1 rounded-full px-1.5 py-0.5"
      style={{
        background: `${TIDE_COLORS[tide]}15`,
        border: `1px solid ${TIDE_COLORS[tide]}30`,
      }}
    >
      <img
        src={tideIconUrl(tide)}
        alt={tide}
        className="rounded-full"
        style={{
          width: isPrimary ? "14px" : "10px",
          height: isPrimary ? "14px" : "10px",
          border: `1px solid ${TIDE_COLORS[tide]}`,
        }}
      />
      <span
        className="font-medium"
        style={{
          fontSize: isPrimary ? "11px" : "9px",
          color: TIDE_COLORS[tide],
          opacity: isPrimary ? 1 : 0.7,
        }}
      >
        {tide}
      </span>
    </div>
  );
}

/** Horizontal bar chart showing preference weights for all tides. */
function PreferenceBar({ weights }: { weights: Record<string, number> }) {
  const maxWeight = Math.max(...Object.values(weights), 0.01);

  return (
    <div className="flex flex-col gap-1">
      <h4
        className="text-[10px] font-bold uppercase tracking-wider"
        style={{ color: "#a855f7" }}
      >
        Preference Weights
      </h4>
      {NAMED_TIDES.map((tide) => {
        const weight = weights[tide] ?? 0;
        const pct = (weight / maxWeight) * 100;
        return (
          <div key={tide} className="flex items-center gap-1.5">
            <img
              src={tideIconUrl(tide)}
              alt={tide}
              className="h-3 w-3 rounded-full"
              style={{ border: `1px solid ${TIDE_COLORS[tide]}` }}
            />
            <span
              className="w-10 text-[9px] font-medium"
              style={{ color: TIDE_COLORS[tide] }}
            >
              {tide}
            </span>
            <div
              className="h-2.5 flex-1 overflow-hidden rounded-full"
              style={{ background: "rgba(255, 255, 255, 0.05)" }}
            >
              <div
                className="h-full rounded-full transition-all duration-300"
                style={{
                  width: `${String(Math.max(pct, 0))}%`,
                  background: TIDE_COLORS[tide],
                  opacity: 0.7,
                }}
              />
            </div>
            <span
              className="w-8 text-right text-[9px] font-mono opacity-50"
              style={{ color: "#e2e8f0" }}
            >
              {(weight * 100).toFixed(0)}%
            </span>
          </div>
        );
      })}
    </div>
  );
}

/** Compact display of card counts per tide. */
function TideCardCounts({
  cardsByTide,
}: {
  cardsByTide: Record<string, number>;
}) {
  const nonZeroTides = [...NAMED_TIDES, "Wild" as Tide].filter(
    (tide) => (cardsByTide[tide] ?? 0) > 0,
  );

  if (nonZeroTides.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-wrap gap-1.5">
      {nonZeroTides.map((tide) => (
        <div
          key={tide}
          className="flex items-center gap-1 rounded-full px-1.5 py-0.5"
          style={{
            background: `${TIDE_COLORS[tide]}10`,
            border: `1px solid ${TIDE_COLORS[tide]}20`,
          }}
        >
          <img
            src={tideIconUrl(tide)}
            alt={tide}
            className="h-3 w-3 rounded-full"
            style={{ border: `1px solid ${TIDE_COLORS[tide]}` }}
          />
          <span
            className="text-[10px] font-bold"
            style={{ color: TIDE_COLORS[tide] }}
          >
            {String(cardsByTide[tide])}
          </span>
        </div>
      ))}
    </div>
  );
}
