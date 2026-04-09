import { useCallback, useEffect, useMemo } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { DraftState } from "../types/draft";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { extractDraftDebugInfo } from "./debug-helpers";

/** Props for the DebugScreen component. */
interface DebugScreenProps {
  isOpen: boolean;
  onClose: () => void;
  draftState: DraftState | null;
  cardDatabase: Map<number, CardData>;
  chosenTide: Tide | null;
}

/** Full-screen overlay showing draft state debug info. */
export function DebugScreen({
  isOpen,
  onClose,
  draftState,
  cardDatabase,
  chosenTide,
}: DebugScreenProps) {
  const debugInfo = useMemo(
    () => extractDraftDebugInfo(draftState, cardDatabase, chosenTide),
    [draftState, cardDatabase, chosenTide],
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
              Debug: Draft State
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
            {debugInfo === null ? (
              <div className="flex h-full items-center justify-center">
                <p className="text-sm opacity-40">
                  No draft data available yet.
                </p>
              </div>
            ) : (
              <div className="mx-auto max-w-2xl space-y-4">
                {/* Stats row */}
                <div className="flex flex-wrap gap-3">
                  <StatBadge label="Pick" value={String(debugInfo.pickNumber)} />
                  <StatBadge label="Pool" value={String(debugInfo.poolSize)} />
                  <StatBadge label="Seen" value={String(debugInfo.seenCards)} />
                  <StatBadge label="Drafted" value={String(debugInfo.totalCards)} />
                </div>

                {/* Chosen tide */}
                {debugInfo.chosenTide && (
                  <div
                    className="flex items-center gap-2 rounded-lg p-3"
                    style={{
                      background: "rgba(0, 0, 0, 0.3)",
                      border: "1px solid rgba(124, 58, 237, 0.15)",
                    }}
                  >
                    <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: "#a855f7" }}>
                      Chosen Tide
                    </span>
                    <img
                      src={tideIconUrl(debugInfo.chosenTide)}
                      alt={debugInfo.chosenTide}
                      className="h-5 w-5 rounded-full"
                      style={{ border: `1px solid ${TIDE_COLORS[debugInfo.chosenTide]}` }}
                    />
                    <span
                      className="text-sm font-bold"
                      style={{ color: TIDE_COLORS[debugInfo.chosenTide] }}
                    >
                      {debugInfo.chosenTide}
                    </span>
                  </div>
                )}

                {/* Cards by tide */}
                {debugInfo.totalCards > 0 && (
                  <div>
                    <h4
                      className="mb-1.5 text-[10px] font-bold uppercase tracking-wider"
                      style={{ color: "#a855f7" }}
                    >
                      Drafted Cards by Tide
                    </h4>
                    <TideCardCounts cardsByTide={debugInfo.cardsByTide} />
                  </div>
                )}

                {/* Card name list */}
                {debugInfo.draftedCards.length > 0 && (
                  <div>
                    <h4
                      className="mb-1.5 text-[10px] font-bold uppercase tracking-wider"
                      style={{ color: "#a855f7" }}
                    >
                      All Drafted Cards (newest first)
                    </h4>
                    <div
                      className="max-h-48 overflow-y-auto rounded p-2"
                      style={{
                        background: "rgba(0, 0, 0, 0.3)",
                        border: "1px solid rgba(124, 58, 237, 0.1)",
                      }}
                    >
                      <div className="flex flex-wrap gap-1">
                        {debugInfo.draftedCards.map((card, i) => (
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
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/** Small stat badge. */
function StatBadge({ label, value }: { label: string; value: string }) {
  return (
    <div
      className="rounded-lg px-3 py-1.5"
      style={{
        background: "rgba(124, 58, 237, 0.1)",
        border: "1px solid rgba(124, 58, 237, 0.2)",
      }}
    >
      <span className="text-[10px] uppercase tracking-wider opacity-50">{label}</span>
      <span className="ml-1.5 text-sm font-bold" style={{ color: "#c084fc" }}>
        {value}
      </span>
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
