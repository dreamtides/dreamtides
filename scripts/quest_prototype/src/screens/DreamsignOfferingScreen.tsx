import { useCallback, useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { DreamsignImage } from "../components/DreamsignImage";
import type { Dreamsign, SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import { drawDreamsignOptions } from "../dreamsign/dreamsign-pool";

const MAX_DREAMSIGNS = 12;

/** Props for the DreamsignOfferingScreen component. */
interface DreamsignOfferingScreenProps {
  site: SiteState;
}

/** Renders the Dreamsign Offering site. Normal: 1 dreamsign, Enhanced: 3 (mini-draft). */
export function DreamsignOfferingScreen({
  site,
}: DreamsignOfferingScreenProps) {
  const { state, mutations, questContent } = useQuest();
  const { dreamsigns: currentDreamsigns } = state;
  const selectedPackageTides = state.resolvedPackage?.selectedTides ?? [];

  const optionCount = site.isEnhanced ? 3 : 1;
  const revealedRef = useRef<ReturnType<typeof drawDreamsignOptions> | null>(null);
  if (revealedRef.current === null) {
    revealedRef.current = drawDreamsignOptions(
      state.remainingDreamsignPool,
      questContent.dreamsignTemplates,
      selectedPackageTides,
      optionCount,
    );
  }
  const revealed = revealedRef.current;
  if (revealed === null) {
    throw new Error("Failed to reveal Dreamsign offering");
  }
  const options = revealed.offeredDreamsigns;

  const [purging, setPurging] = useState(false);
  const [pendingDreamsign, setPendingDreamsign] = useState<Dreamsign | null>(
    null,
  );

  useEffect(() => {
    logEvent("site_entered", {
      siteType: "DreamsignOffering",
      isEnhanced: site.isEnhanced,
      optionCount,
    });
    mutations.setRemainingDreamsignPool(
      revealed.remainingDreamsignPool,
      "dreamsign_offering_revealed",
    );
  }, [site.isEnhanced, optionCount, mutations, revealed.remainingDreamsignPool]);

  const completeSite = useCallback(() => {
    logEvent("site_completed", {
      siteType: "DreamsignOffering",
      isEnhanced: site.isEnhanced,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations]);

  const handleAccept = useCallback(
    (dreamsign: Dreamsign) => {
      if (currentDreamsigns.length >= MAX_DREAMSIGNS) {
        setPendingDreamsign(dreamsign);
        setPurging(true);
        return;
      }
      mutations.addDreamsign(dreamsign, "DreamsignOffering");
      completeSite();
    },
    [currentDreamsigns.length, mutations, completeSite],
  );

  const handlePurge = useCallback(
    (index: number) => {
      mutations.removeDreamsign(index, "purged_for_new_dreamsign");
      if (pendingDreamsign) {
        mutations.addDreamsign(pendingDreamsign, "DreamsignOffering");
      }
      setPurging(false);
      setPendingDreamsign(null);
      completeSite();
    },
    [pendingDreamsign, mutations, completeSite],
  );

  const handleReject = useCallback(() => {
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
                <DreamsignImage
                  name={sign.name}
                  imageName={sign.imageName}
                  imageAlt={sign.imageAlt}
                  className="h-10 w-10"
                  frameClassName="border border-white/10"
                  placeholderClassName="text-sm text-slate-200"
                />
                <span className="text-xs font-bold" style={{ color: "#e9d5ff" }}>
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
          Dreamsign Offering
        </h2>
        {site.isEnhanced && (
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(168, 85, 247, 0.15)",
              color: "#c084fc",
              border: "1px solid rgba(168, 85, 247, 0.3)",
            }}
          >
            Enhanced -- Choose from 3
          </span>
        )}
      </div>

      {/* Dreamsign options */}
      {options.length === 0 ? (
        <p className="text-sm opacity-60">
          The Dreamsign pool is exhausted.
        </p>
      ) : (
        <div
          className={`flex flex-wrap justify-center gap-6 ${options.length === 1 ? "" : "max-w-3xl"}`}
        >
          {options.map((dreamsign, index) => (
            <motion.div
              key={`offer-${dreamsign.name}`}
              className="flex flex-col items-center gap-3"
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1, duration: 0.4 }}
            >
              <DreamsignCard dreamsign={dreamsign} />
              <button
                className="w-full rounded-lg px-5 py-2.5 font-bold text-white transition-opacity"
                style={{ backgroundColor: "#7c3aed" }}
                onClick={() => handleAccept(dreamsign)}
              >
                Accept
              </button>
            </motion.div>
          ))}
        </div>
      )}

      {/* Reject / Skip */}
      <button
        className="mt-8 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
        style={{
          background: "rgba(107, 114, 128, 0.2)",
          border: "1px solid rgba(107, 114, 128, 0.4)",
          color: "#9ca3af",
        }}
        onClick={handleReject}
      >
        {options.length > 1 ? "Skip" : "Reject"}
      </button>
    </motion.div>
  );
}

/** Renders a dreamsign card with artwork, name, and effect description. */
function DreamsignCard({ dreamsign }: { dreamsign: Dreamsign }) {
  return (
    <div
      className="flex w-56 flex-1 flex-col items-center gap-2 rounded-lg p-4"
      style={{
        background:
          "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: "1px solid rgba(192, 132, 252, 0.35)",
        boxShadow: "0 0 12px rgba(168, 85, 247, 0.18)",
      }}
    >
      <DreamsignImage
        name={dreamsign.name}
        imageName={dreamsign.imageName}
        imageAlt={dreamsign.imageAlt}
        className="h-24 w-24"
        frameClassName="border border-fuchsia-300/25 shadow-[0_0_18px_rgba(168,85,247,0.18)]"
        placeholderClassName="text-3xl text-fuchsia-100"
      />
      <h3
        className="text-center text-base font-bold"
        style={{ color: "#f5d0fe" }}
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
