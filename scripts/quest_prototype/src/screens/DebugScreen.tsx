import { Fragment, useCallback, useEffect, useMemo } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { DraftState } from "../types/draft";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { extractDraftDebugInfo, type SeatSummary } from "./debug-helpers";

/** Props for the DebugScreen component. */
interface DebugScreenProps {
  isOpen: boolean;
  onClose: () => void;
  draftState: DraftState | null;
  cardDatabase: Map<number, CardData>;
}

/**
 * Full-screen overlay showing draft intelligence data for all seats.
 * Displays each seat's tide preferences, drafted cards, passing direction,
 * and highlights the human player seat.
 */
export function DebugScreen({
  isOpen,
  onClose,
  draftState,
  cardDatabase,
}: DebugScreenProps) {
  const debugInfo = useMemo(
    () => extractDraftDebugInfo(draftState, cardDatabase),
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
            <div>
              <h2
                className="text-lg font-bold md:text-xl"
                style={{ color: "#e2e8f0" }}
              >
                Debug: AI Draft Intelligence
              </h2>
              {debugInfo !== null && (
                <PassingBanner
                  displayRound={debugInfo.displayRound}
                  seatPassingToPlayer={debugInfo.seatPassingToPlayer}
                />
              )}
            </div>
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
            {debugInfo === null ? (
              <div className="flex h-full items-center justify-center">
                <p className="text-sm opacity-40">
                  No draft data available yet.
                </p>
              </div>
            ) : (
              <>
                {/* Pack flow diagram */}
                <PackFlowDiagram
                  seatCount={debugInfo.seats.length}
                  seatPassingToPlayer={debugInfo.seatPassingToPlayer}
                />

                {/* All seats in passing order, human last */}
                <SeatFlowGrid
                  seats={debugInfo.seats}
                  cardDatabase={cardDatabase}
                />
              </>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/** Banner showing current round and passing info. */
function PassingBanner({
  displayRound,
  seatPassingToPlayer,
}: {
  displayRound: number;
  seatPassingToPlayer: number;
}) {
  return (
    <div
      className="mt-1 flex items-center gap-3 text-xs"
      style={{ color: "#94a3b8" }}
    >
      <span>Round {String(displayRound)} {"\u2190"} Packs pass left</span>
      <span
        className="rounded-full px-2 py-0.5"
        style={{
          background: "rgba(251, 191, 36, 0.15)",
          border: "1px solid rgba(251, 191, 36, 0.3)",
          color: "#fbbf24",
        }}
      >
        Seat {String(seatPassingToPlayer)} passes to you
      </span>
    </div>
  );
}

/** Compact horizontal diagram showing all seats in pack-passing order. */
function PackFlowDiagram({
  seatCount,
  seatPassingToPlayer,
}: {
  seatCount: number;
  seatPassingToPlayer: number;
}) {
  const orderedSeats: number[] = [];
  for (let i = 1; i < seatCount; i++) {
    orderedSeats.push(i);
  }
  orderedSeats.push(0);

  return (
    <div
      className="mb-4 flex flex-wrap items-center justify-center gap-0.5 rounded-lg px-4 py-3"
      style={{
        background: "rgba(10, 6, 18, 0.8)",
        border: "1px solid rgba(124, 58, 237, 0.2)",
      }}
    >
      <span
        className="mr-3 text-[10px] font-bold uppercase tracking-wider"
        style={{ color: "#64748b" }}
      >
        Pack flow
      </span>
      {orderedSeats.map((seatIdx, i) => {
        const isPlayer = seatIdx === 0;
        const isPasser = seatIdx === seatPassingToPlayer;
        const arrowToPlayer = i > 0 && isPlayer;
        const arrowFromPlayer =
          i > 0 && orderedSeats[i - 1] === 0;

        return (
          <Fragment key={seatIdx}>
            {i > 0 && (
              <span
                className="mx-0.5"
                style={{
                  color: arrowToPlayer
                    ? "#fbbf24"
                    : arrowFromPlayer
                      ? "#38bdf8"
                      : "#334155",
                  fontSize:
                    arrowToPlayer || arrowFromPlayer
                      ? "14px"
                      : "10px",
                  fontWeight:
                    arrowToPlayer || arrowFromPlayer ? 800 : 400,
                }}
              >
                {"\u2192"}
              </span>
            )}
            <div
              className="flex items-center justify-center rounded-full font-bold"
              style={{
                width: isPlayer ? "28px" : isPasser ? "24px" : "20px",
                height: isPlayer
                  ? "28px"
                  : isPasser
                    ? "24px"
                    : "20px",
                fontSize: isPlayer ? "12px" : "9px",
                background: isPlayer
                  ? "rgba(251, 191, 36, 0.3)"
                  : isPasser
                    ? "rgba(56, 189, 248, 0.2)"
                    : "rgba(255, 255, 255, 0.05)",
                border: isPlayer
                  ? "2px solid rgba(251, 191, 36, 0.8)"
                  : isPasser
                    ? "2px solid rgba(56, 189, 248, 0.6)"
                    : "1px solid rgba(255, 255, 255, 0.1)",
                color: isPlayer
                  ? "#fbbf24"
                  : isPasser
                    ? "#38bdf8"
                    : "#64748b",
              }}
            >
              {isPlayer ? "\u2605" : String(seatIdx)}
            </div>
          </Fragment>
        );
      })}
      <span
        className="ml-1 text-xs opacity-30"
        style={{ color: "#e2e8f0" }}
      >
        {"\u21A9"}
      </span>
    </div>
  );
}

/** Grid showing all seats in passing order with arrows between pairs. */
function SeatFlowGrid({
  seats,
  cardDatabase,
}: {
  seats: SeatSummary[];
  cardDatabase: Map<number, CardData>;
}) {
  // Order: seats 1..N-1, then seat 0 (human last)
  const orderedSeats = [
    ...seats.filter((s) => !s.isPlayer).sort((a, b) => a.seatIndex - b.seatIndex),
    seats[0],
  ];

  // Group into pairs for 2-column layout
  const pairs: SeatSummary[][] = [];
  for (let i = 0; i < orderedSeats.length; i += 2) {
    pairs.push(orderedSeats.slice(i, i + 2));
  }

  return (
    <div className="space-y-0">
      {pairs.map((pair, pairIdx) => (
        <Fragment key={pairIdx}>
          {pairIdx > 0 && (
            <RowConnector />
          )}
          <div className="flex flex-col items-stretch gap-0 md:flex-row">
            <div className="min-w-0 flex-1">
              <SeatCard
                seat={pair[0]}
                cardDatabase={cardDatabase}
                passesToPlayer={false}
              />
            </div>
            {pair.length > 1 && (
              <>
                <PassingArrow
                  label={pair[1].isPlayer ? "passes to you" : ""}
                  highlighted={pair[1].isPlayer}
                />
                <div className="min-w-0 flex-1">
                  <SeatCard
                    seat={pair[1]}
                    cardDatabase={cardDatabase}
                    passesToPlayer={false}
                  />
                </div>
              </>
            )}
          </div>
        </Fragment>
      ))}
    </div>
  );
}

/** U-shaped connector between rows: down from right, left across, down to left. */
function RowConnector() {
  const color = "#475569";
  return (
    <>
      {/* Desktop: U-shaped connector */}
      <div className="relative my-1 hidden h-8 w-full md:block" aria-hidden="true">
        {/* Down from right card center */}
        <div
          className="absolute top-0 h-2.5"
          style={{ right: "25%", width: "2px", background: color }}
        />
        {/* Horizontal line going left */}
        <div
          className="absolute h-[2px]"
          style={{ left: "25%", right: "25%", top: "10px", background: color }}
        />
        {/* Down to left card center */}
        <div
          className="absolute h-3"
          style={{ left: "25%", top: "10px", width: "2px", background: color }}
        />
        {/* Arrowhead */}
        <span
          className="absolute text-[10px]"
          style={{ left: "calc(25% - 4px)", top: "18px", color }}
        >
          {"\u25BC"}
        </span>
      </div>
      {/* Mobile: simple centered down arrow */}
      <div className="flex justify-center py-1 md:hidden" aria-hidden="true">
        <span className="text-sm" style={{ color }}>{"\u25BC"}</span>
      </div>
    </>
  );
}

/** Arrow connector between seats in the passing trio. */
function PassingArrow({
  label,
  highlighted,
}: {
  label: string;
  highlighted?: boolean;
}) {
  const color = highlighted ? "#fbbf24" : "#64748b";
  return (
    <div className="flex shrink-0 flex-col items-center justify-center px-1 py-2 md:px-3 md:py-0">
      {/* Desktop: horizontal arrow */}
      <div
        className="hidden items-center md:flex"
        style={{ color }}
      >
        <div
          className="h-[2px] w-8"
          style={{
            background: `linear-gradient(90deg, transparent, ${color})`,
          }}
        />
        <span className="text-xl">{"\u25B6"}</span>
      </div>
      {/* Mobile: vertical arrow */}
      <span
        className="block text-xl md:hidden"
        style={{ color }}
      >
        {"\u25BC"}
      </span>
      {label !== "" && (
        <span
          className="mt-0.5 text-[9px] font-bold whitespace-nowrap"
          style={{ color }}
        >
          {label}
        </span>
      )}
    </div>
  );
}

/** Card displaying a single seat's draft intelligence. */
function SeatCard({
  seat,
  cardDatabase: _cardDatabase,
  passesToPlayer,
}: {
  seat: SeatSummary;
  cardDatabase: Map<number, CardData>;
  passesToPlayer: boolean;
}) {
  const primaryColor =
    seat.primaryTide !== null ? TIDE_COLORS[seat.primaryTide] : "#6b7280";

  return (
    <div
      className="rounded-lg p-4"
      style={{
        background: seat.isPlayer
          ? "rgba(251, 191, 36, 0.05)"
          : "rgba(10, 6, 18, 0.6)",
        border: seat.isPlayer
          ? "2px solid rgba(251, 191, 36, 0.4)"
          : passesToPlayer
            ? `1px solid rgba(56, 189, 248, 0.4)`
            : `1px solid ${primaryColor}40`,
      }}
    >
      {/* Seat header */}
      <div className="mb-3 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div
            className="flex h-8 w-8 items-center justify-center rounded-full text-sm font-bold"
            style={{
              background: seat.isPlayer
                ? "rgba(251, 191, 36, 0.2)"
                : `${primaryColor}20`,
              border: seat.isPlayer
                ? "2px solid rgba(251, 191, 36, 0.6)"
                : `2px solid ${primaryColor}60`,
              color: seat.isPlayer ? "#fbbf24" : primaryColor,
            }}
          >
            {seat.isPlayer ? "\u2605" : String(seat.seatIndex)}
          </div>
          <div>
            <div className="flex items-center gap-1.5">
              <span
                className="text-sm font-bold"
                style={{ color: seat.isPlayer ? "#fbbf24" : "#e2e8f0" }}
              >
                {seat.isPlayer
                  ? "You (Seat 0)"
                  : `Seat ${String(seat.seatIndex)}`}
              </span>
            </div>
            <div className="flex items-center gap-1.5">
              {seat.primaryTide !== null && (
                <TideBadge tide={seat.primaryTide} size="primary" />
              )}
              {seat.secondaryTide !== null && (
                <TideBadge tide={seat.secondaryTide} size="secondary" />
              )}
              {seat.primaryTide === null && (
                <span className="text-[10px] opacity-40">
                  No tide preference
                </span>
              )}
            </div>
          </div>
        </div>
        <div>
          <span
            className="text-xs opacity-50"
            style={{ color: "#e2e8f0" }}
          >
            {String(seat.totalCards)} cards
          </span>
        </div>
      </div>

      {/* Preference weights bar chart */}
      {!seat.isPlayer && (
        <PreferenceBar weights={seat.preferenceWeights} />
      )}

      {/* Player seat: simplified view */}
      {seat.isPlayer && (
        <div
          className="rounded p-2 text-center text-xs"
          style={{
            background: "rgba(251, 191, 36, 0.08)",
            border: "1px solid rgba(251, 191, 36, 0.15)",
            color: "#fbbf24",
          }}
        >
          Human Player
        </div>
      )}

      {/* Cards by tide summary */}
      {seat.totalCards > 0 && (
        <div className="mt-3">
          <h4
            className="mb-1.5 text-[10px] font-bold uppercase tracking-wider"
            style={{ color: "#a855f7" }}
          >
            Drafted Cards
          </h4>
          <TideCardCounts cardsByTide={seat.cardsByTide} />
        </div>
      )}

      {/* Card name list */}
      {seat.draftedCards.length > 0 && (
        <div className="mt-2">
          <div
            className="max-h-32 overflow-y-auto rounded p-2"
            style={{
              background: "rgba(0, 0, 0, 0.3)",
              border: "1px solid rgba(124, 58, 237, 0.1)",
            }}
          >
            <div className="flex flex-wrap gap-1">
              {seat.draftedCards.map((card, i) => (
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
  const nonZeroTides = [...NAMED_TIDES, "Neutral" as Tide].filter(
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
