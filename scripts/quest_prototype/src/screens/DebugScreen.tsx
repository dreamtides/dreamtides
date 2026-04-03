import { useCallback, useEffect, useMemo } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { DraftState, PackStrategy } from "../types/draft";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { extractDraftDebugInfo } from "./debug-helpers";
import type { DecayEntry } from "./debug-helpers";

/** Props for the DebugScreen component. */
interface DebugScreenProps {
  isOpen: boolean;
  onClose: () => void;
  draftState: DraftState | null;
  cardDatabase: Map<number, CardData>;
  excludedTides: Tide[];
}

/** Full-screen overlay showing draft state debug info. */
export function DebugScreen({
  isOpen,
  onClose,
  draftState,
  cardDatabase,
  excludedTides,
}: DebugScreenProps) {
  const debugInfo = useMemo(
    () => extractDraftDebugInfo(draftState, cardDatabase, excludedTides),
    [draftState, cardDatabase, excludedTides],
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
                  <StatBadge label="Focus" value={String(debugInfo.focus)} />
                  <StatBadge label="Drafted" value={String(debugInfo.totalCards)} />
                  <StatBadge label="Exp. Dom" value={String(debugInfo.expectedDominant)} />
                </div>

                {/* Quest setup: strategy + excluded tides */}
                <QuestSetupSection
                  packStrategy={debugInfo.packStrategy}
                  excludedTides={debugInfo.excludedTides}
                />

                {/* Primary/secondary tides */}
                <div className="flex items-center gap-2">
                  {debugInfo.primaryTide !== null && (
                    <TideBadge tide={debugInfo.primaryTide} size="primary" />
                  )}
                  {debugInfo.secondaryTide !== null && (
                    <TideBadge tide={debugInfo.secondaryTide} size="secondary" />
                  )}
                  {debugInfo.primaryTide === null && (
                    <span className="text-xs opacity-40">No dominant tide yet</span>
                  )}
                </div>

                {/* Tide affinity bar chart */}
                <AffinityBar affinities={debugInfo.tideAffinities} />

                {/* Effective sampling weights */}
                <EffectiveWeightsBar
                  effectiveWeights={debugInfo.effectiveWeights}
                  tideProbabilities={debugInfo.tideProbabilities}
                  packStrategy={debugInfo.packStrategy}
                  excludedTides={debugInfo.excludedTides}
                />

                {/* Recency decay profile */}
                {debugInfo.decayProfile.length > 0 && (
                  <DecayProfileSection decayProfile={debugInfo.decayProfile} />
                )}

                {/* Cards by tide */}
                {debugInfo.totalCards > 0 && (
                  <div>
                    <SectionHeader>Drafted Cards by Tide</SectionHeader>
                    <TideCardCounts cardsByTide={debugInfo.cardsByTide} />
                  </div>
                )}

                {/* Card name list */}
                {debugInfo.draftedCards.length > 0 && (
                  <div>
                    <SectionHeader>All Drafted Cards (newest first)</SectionHeader>
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

/** Section header label. */
function SectionHeader({ children }: { children: string }) {
  return (
    <h4
      className="mb-1.5 text-[10px] font-bold uppercase tracking-wider"
      style={{ color: "#a855f7" }}
    >
      {children}
    </h4>
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

/** Quest setup: pack strategy and excluded tides. */
function QuestSetupSection({
  packStrategy,
  excludedTides,
}: {
  packStrategy: PackStrategy;
  excludedTides: Tide[];
}) {
  return (
    <div
      className="rounded-lg p-3"
      style={{
        background: "rgba(0, 0, 0, 0.3)",
        border: "1px solid rgba(124, 58, 237, 0.15)",
      }}
    >
      <SectionHeader>Quest Setup</SectionHeader>
      <div className="flex flex-wrap items-center gap-2">
        {/* Strategy badge */}
        <span
          className="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider"
          style={{
            background: packStrategy.type === "pool_bias"
              ? "rgba(249, 115, 22, 0.15)"
              : "rgba(124, 58, 237, 0.15)",
            border: `1px solid ${packStrategy.type === "pool_bias" ? "rgba(249, 115, 22, 0.4)" : "rgba(124, 58, 237, 0.3)"}`,
            color: packStrategy.type === "pool_bias" ? "#fb923c" : "#c084fc",
          }}
        >
          {packStrategy.type === "pool_bias" ? "Pool Bias" : "Tide Current"}
        </span>

        {/* Featured tides (pool bias only) */}
        {packStrategy.type === "pool_bias" && packStrategy.featuredTides.map((tide) => (
          <div
            key={tide}
            className="flex items-center gap-1 rounded-full px-1.5 py-0.5"
            style={{
              background: `${TIDE_COLORS[tide]}20`,
              border: `1px solid ${TIDE_COLORS[tide]}50`,
            }}
          >
            <img
              src={tideIconUrl(tide)}
              alt={tide}
              className="h-3 w-3 rounded-full"
              style={{ border: `1px solid ${TIDE_COLORS[tide]}` }}
            />
            <span className="text-[10px] font-bold" style={{ color: TIDE_COLORS[tide] }}>
              {tide}
            </span>
            <span className="text-[9px] font-medium opacity-60" style={{ color: TIDE_COLORS[tide] }}>
              {String(packStrategy.featuredWeight)}x
            </span>
          </div>
        ))}

        {/* Excluded tides */}
        {excludedTides.length > 0 && (
          <>
            <span className="text-[9px] uppercase tracking-wider opacity-30">Excluded:</span>
            {excludedTides.map((tide) => (
              <span
                key={tide}
                className="rounded-full px-1.5 py-0.5 text-[9px] font-medium"
                style={{
                  background: "rgba(255, 255, 255, 0.05)",
                  border: "1px solid rgba(255, 255, 255, 0.1)",
                  color: "rgba(255, 255, 255, 0.3)",
                  textDecoration: "line-through",
                }}
              >
                {tide}
              </span>
            ))}
          </>
        )}
      </div>
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

/** Horizontal bar chart showing tide affinities. */
function AffinityBar({ affinities }: { affinities: Record<string, number> }) {
  const maxAffinity = Math.max(...Object.values(affinities), 1);

  return (
    <div className="flex flex-col gap-1">
      <SectionHeader>Tide Affinities</SectionHeader>
      {[...NAMED_TIDES, "Neutral" as Tide].map((tide) => {
        const value = affinities[tide] ?? 0;
        const pct = (value / maxAffinity) * 100;
        return (
          <div key={tide} className="flex items-center gap-1.5">
            <img
              src={tideIconUrl(tide)}
              alt={tide}
              className="h-3 w-3 rounded-full"
              style={{ border: `1px solid ${TIDE_COLORS[tide]}` }}
            />
            <span
              className="w-12 text-[9px] font-medium"
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
              {value.toFixed(1)}
            </span>
          </div>
        );
      })}
    </div>
  );
}

/** Horizontal bar chart showing effective sampling weights with probabilities. */
function EffectiveWeightsBar({
  effectiveWeights,
  tideProbabilities,
  packStrategy,
  excludedTides,
}: {
  effectiveWeights: Record<string, number>;
  tideProbabilities: Record<string, number>;
  packStrategy: PackStrategy;
  excludedTides: Tide[];
}) {
  const excludedSet = new Set<string>(excludedTides);
  const activeTides = [...NAMED_TIDES, "Neutral" as Tide].filter(
    (t) => !excludedSet.has(t),
  );
  const maxWeight = Math.max(
    ...activeTides.map((t) => effectiveWeights[t] ?? 0),
    0.001,
  );
  const featuredSet = new Set<string>(
    packStrategy.type === "pool_bias" ? packStrategy.featuredTides : [],
  );

  return (
    <div className="flex flex-col gap-1">
      <SectionHeader>Effective Sampling Weights</SectionHeader>
      {activeTides.map((tide) => {
        const weight = effectiveWeights[tide] ?? 0;
        const prob = tideProbabilities[tide] ?? 0;
        const pct = (weight / maxWeight) * 100;
        const isFeatured = featuredSet.has(tide);
        return (
          <div key={tide} className="flex items-center gap-1.5">
            <img
              src={tideIconUrl(tide)}
              alt={tide}
              className="h-3 w-3 rounded-full"
              style={{ border: `1px solid ${TIDE_COLORS[tide]}` }}
            />
            <span
              className="w-12 text-[9px] font-medium"
              style={{ color: TIDE_COLORS[tide] }}
            >
              {tide}
            </span>
            {isFeatured && (
              <span className="text-[8px]" title="Featured tide">
                {"*"}
              </span>
            )}
            <div
              className="h-2.5 flex-1 overflow-hidden rounded-full"
              style={{ background: "rgba(255, 255, 255, 0.05)" }}
            >
              <div
                className="h-full rounded-full transition-all duration-300"
                style={{
                  width: `${String(Math.max(pct, 0))}%`,
                  background: isFeatured
                    ? `linear-gradient(90deg, ${TIDE_COLORS[tide]}, #fb923c)`
                    : TIDE_COLORS[tide],
                  opacity: 0.8,
                }}
              />
            </div>
            <span
              className="w-12 text-right text-[9px] font-mono opacity-50"
              style={{ color: "#e2e8f0" }}
            >
              {prob.toFixed(1)}%
            </span>
          </div>
        );
      })}
    </div>
  );
}

/** Recency decay profile showing how older picks fade. */
function DecayProfileSection({ decayProfile }: { decayProfile: DecayEntry[] }) {
  return (
    <div>
      <SectionHeader>{`Recency Decay (last ${String(decayProfile.length)} picks)`}</SectionHeader>
      <div className="flex flex-wrap gap-1">
        {decayProfile.map((entry, i) => (
          <div
            key={`decay-${String(i)}`}
            className="flex items-center gap-1 rounded-full px-1.5 py-0.5"
            style={{
              background: `${TIDE_COLORS[entry.tide]}${Math.round(entry.decay * 20).toString(16).padStart(2, "0")}`,
              border: `1px solid ${TIDE_COLORS[entry.tide]}30`,
              opacity: 0.4 + entry.decay * 0.6,
            }}
          >
            <img
              src={tideIconUrl(entry.tide)}
              alt={entry.tide}
              className="h-2.5 w-2.5 rounded-full"
              style={{ border: `1px solid ${TIDE_COLORS[entry.tide]}` }}
            />
            <span
              className="text-[9px] font-medium"
              style={{ color: TIDE_COLORS[entry.tide] }}
            >
              {entry.cardName}
            </span>
            <span
              className="text-[8px] font-mono opacity-60"
              style={{ color: "#e2e8f0" }}
            >
              {"\u00d7"}{entry.decay.toFixed(2)}
            </span>
          </div>
        ))}
      </div>
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
