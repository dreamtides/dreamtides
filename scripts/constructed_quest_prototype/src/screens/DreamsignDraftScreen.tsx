import { useCallback, useEffect, useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { Dreamsign, SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import { DREAMSIGNS } from "../data/dreamsigns";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import {
  countDeckTides,
  tideWeight,
  weightedSample,
} from "../data/tide-weights";

const MAX_DREAMSIGNS = 12;

/** Props for the DreamsignDraftScreen component. */
interface DreamsignDraftScreenProps {
  site: SiteState;
}

/** Shows 3 (or 4 enhanced) dreamsign options. Pick 1 or skip. */
export function DreamsignDraftScreen({ site }: DreamsignDraftScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const { dreamsigns: currentDreamsigns, deck } = state;

  const optionCount = site.isEnhanced ? 4 : 3;

  const options = useMemo<Dreamsign[]>(() => {
    const deckTides = countDeckTides(deck, cardDatabase);
    return weightedSample(DREAMSIGNS, optionCount, (t) =>
      tideWeight(t.tide, deckTides),
    ).map((t) => ({ ...t, isBane: false }));
  }, [optionCount, deck, cardDatabase]);

  const [purging, setPurging] = useState(false);
  const [pendingDreamsign, setPendingDreamsign] = useState<Dreamsign | null>(
    null,
  );

  useEffect(() => {
    logEvent("site_entered", {
      siteType: "DreamsignDraft",
      isEnhanced: site.isEnhanced,
      optionCount,
    });
  }, [site.isEnhanced, optionCount]);

  const completeSite = useCallback(() => {
    logEvent("site_completed", {
      siteType: "DreamsignDraft",
      isEnhanced: site.isEnhanced,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations]);

  const handleSelect = useCallback(
    (dreamsign: Dreamsign) => {
      if (currentDreamsigns.length >= MAX_DREAMSIGNS) {
        setPendingDreamsign(dreamsign);
        setPurging(true);
        return;
      }
      mutations.addDreamsign(dreamsign, "DreamsignDraft");
      completeSite();
    },
    [currentDreamsigns.length, mutations, completeSite],
  );

  const handlePurge = useCallback(
    (index: number) => {
      mutations.removeDreamsign(index, "purged_for_new_dreamsign");
      if (pendingDreamsign) {
        mutations.addDreamsign(pendingDreamsign, "DreamsignDraft");
      }
      setPurging(false);
      setPendingDreamsign(null);
      completeSite();
    },
    [pendingDreamsign, mutations, completeSite],
  );

  const handleSkip = useCallback(() => {
    completeSite();
  }, [completeSite]);

  if (purging) {
    return (
      <motion.div
        className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.3 }}
      >
        <h2
          className="mb-2 text-2xl font-bold"
          style={{ color: "#ef4444" }}
        >
          Dreamsign Limit Reached
        </h2>
        <p className="mb-6 text-sm opacity-70">
          You have {String(MAX_DREAMSIGNS)} dreamsigns. Remove one to accept
          the new dreamsign.
        </p>

        {pendingDreamsign && (
          <div className="mb-6">
            <p className="mb-2 text-center text-xs font-bold uppercase tracking-wider opacity-50">
              New Dreamsign
            </p>
            <DreamsignCard dreamsign={pendingDreamsign} />
          </div>
        )}

        <p className="mb-3 text-xs font-bold uppercase tracking-wider opacity-50">
          Select one to remove
        </p>
        <div className="grid max-w-3xl grid-cols-3 gap-3 md:grid-cols-4">
          {currentDreamsigns.map((sign, index) => (
            <button
              key={`purge-${sign.name}-${String(index)}`}
              className="cursor-pointer rounded-lg p-2 text-left transition-colors"
              style={{
                background: "rgba(239, 68, 68, 0.05)",
                border: "1px solid rgba(239, 68, 68, 0.2)",
              }}
              onClick={() => handlePurge(index)}
            >
              <div className="flex items-center gap-2">
                <img
                  src={tideIconUrl(sign.tide)}
                  alt={sign.tide}
                  className="h-6 w-6 rounded-full object-contain"
                  style={{
                    border: `1px solid ${TIDE_COLORS[sign.tide]}`,
                  }}
                />
                <span
                  className="text-xs font-bold"
                  style={{ color: TIDE_COLORS[sign.tide] }}
                >
                  {sign.name}
                </span>
              </div>
            </button>
          ))}
        </div>

        <button
          className="mt-6 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={() => {
            setPurging(false);
            setPendingDreamsign(null);
          }}
        >
          Cancel
        </button>
      </motion.div>
    );
  }

  return (
    <motion.div
      className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      {/* Header */}
      <div className="mb-6 text-center">
        <h2
          className="text-2xl font-bold tracking-wide md:text-3xl"
          style={{ color: "#a855f7" }}
        >
          Dreamsign Draft
        </h2>
        <p className="mt-1 text-sm opacity-50">
          Choose one dreamsign or skip
        </p>
        {site.isEnhanced && (
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(168, 85, 247, 0.15)",
              color: "#c084fc",
              border: "1px solid rgba(168, 85, 247, 0.3)",
            }}
          >
            Enhanced -- 4 Options
          </span>
        )}
      </div>

      {/* Dreamsign options */}
      <div className="flex max-w-4xl flex-wrap justify-center gap-5">
        {options.map((dreamsign, index) => (
          <motion.div
            key={`draft-${dreamsign.name}`}
            className="flex flex-col items-center gap-3"
            initial={{ opacity: 0, x: -30 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.12, duration: 0.4 }}
          >
            <DreamsignCard dreamsign={dreamsign} />
            <button
              className="w-full rounded-lg px-5 py-2.5 font-bold text-white transition-opacity"
              style={{ backgroundColor: "#7c3aed" }}
              onClick={() => handleSelect(dreamsign)}
            >
              Select
            </button>
          </motion.div>
        ))}
      </div>

      {/* Skip */}
      <button
        className="mt-8 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
        style={{
          background: "rgba(107, 114, 128, 0.2)",
          border: "1px solid rgba(107, 114, 128, 0.4)",
          color: "#9ca3af",
        }}
        onClick={handleSkip}
      >
        Skip
      </button>
    </motion.div>
  );
}

/** Renders a dreamsign card with tide icon, name, and effect description. */
function DreamsignCard({ dreamsign }: { dreamsign: Dreamsign }) {
  const tideColor = TIDE_COLORS[dreamsign.tide];

  return (
    <div
      className="flex w-52 flex-1 flex-col items-center gap-2 rounded-lg p-4"
      style={{
        background:
          "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `1px solid ${tideColor}60`,
        boxShadow: `0 0 12px ${tideColor}20`,
      }}
    >
      <img
        src={tideIconUrl(dreamsign.tide)}
        alt={dreamsign.tide}
        className="h-12 w-12 rounded-full object-contain"
        style={{ border: `2px solid ${tideColor}` }}
      />
      <span
        className="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider"
        style={{
          background: `${tideColor}20`,
          color: tideColor,
          border: `1px solid ${tideColor}40`,
        }}
      >
        {dreamsign.tide}
      </span>
      <h3
        className="text-center text-sm font-bold"
        style={{ color: tideColor }}
      >
        {dreamsign.name}
      </h3>
      <p
        className="text-center text-xs leading-relaxed opacity-70"
        style={{ color: "#e2e8f0" }}
      >
        {dreamsign.effectDescription}
      </p>
    </div>
  );
}
