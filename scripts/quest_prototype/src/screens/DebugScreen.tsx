import { useCallback, useEffect, useMemo, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData } from "../types/cards";
import type { ResolvedDreamcallerPackage } from "../types/content";
import type { DraftState } from "../types/draft";
import { extractDraftDebugInfo } from "./debug-helpers";

/** Props for the DebugScreen component. */
interface DebugScreenProps {
  isOpen: boolean;
  onClose: () => void;
  draftState: DraftState | null;
  cardDatabase: Map<number, CardData>;
  resolvedPackage: ResolvedDreamcallerPackage | null;
  remainingDreamsignPool: string[];
}

/** Full-screen overlay showing package and draft pool debug info. */
export function DebugScreen({
  isOpen,
  onClose,
  draftState,
  cardDatabase,
  resolvedPackage,
  remainingDreamsignPool,
}: DebugScreenProps) {
  const debugInfo = useMemo(
    () => extractDraftDebugInfo(draftState, cardDatabase),
    [draftState, cardDatabase],
  );

  const handleClose = useCallback(() => {
    onClose();
  }, [onClose]);

  useEffect(() => {
    if (!isOpen) {
      return undefined;
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
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
              Debug: Package State
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

          <div className="flex-1 overflow-y-auto px-4 py-4 md:px-6">
            {resolvedPackage === null ? (
              <div className="flex h-full items-center justify-center">
                <p className="text-sm opacity-40">
                  No package data available yet.
                </p>
              </div>
            ) : (
              <div className="mx-auto max-w-2xl space-y-4">
                <div className="flex flex-wrap gap-3">
                  <StatBadge
                    label="Draft Pool"
                    value={String(resolvedPackage.draftPoolSize)}
                  />
                  <StatBadge
                    label="Signs Left"
                    value={String(remainingDreamsignPool.length)}
                  />
                  {debugInfo !== null && (
                    <>
                      <StatBadge
                        label="Pick"
                        value={String(debugInfo.pickNumber)}
                      />
                      <StatBadge
                        label="Remaining"
                        value={String(debugInfo.remainingCards)}
                      />
                      <StatBadge
                        label="Unique"
                        value={String(debugInfo.remainingUniqueCards)}
                      />
                    </>
                  )}
                </div>

                <InfoCard title="Dreamcaller">
                  <p className="text-sm font-bold" style={{ color: "#e2e8f0" }}>
                    {resolvedPackage.dreamcaller.name}
                  </p>
                </InfoCard>

                <InfoCard title="Optional Subset">
                  <p className="text-sm opacity-80">
                    {resolvedPackage.optionalSubset.join(", ")}
                  </p>
                </InfoCard>

                <InfoCard title="Selected Package Tides">
                  <p className="text-sm opacity-80">
                    {resolvedPackage.selectedTides.join(", ")}
                  </p>
                </InfoCard>

                {debugInfo !== null && (
                  <InfoCard title="Current Offer">
                    {debugInfo.currentOfferSize === 0 ? (
                      <p className="text-sm opacity-50">
                        No offer is currently active.
                      </p>
                    ) : (
                      <div className="flex flex-wrap gap-1">
                        {debugInfo.currentOffer.map((card) => (
                          <span
                            key={card.cardNumber}
                            className="rounded-full px-2 py-0.5 text-[10px] font-medium"
                            style={{
                              background: "rgba(168, 85, 247, 0.12)",
                              border: "1px solid rgba(168, 85, 247, 0.24)",
                              color: "#c084fc",
                            }}
                          >
                            {card.name}
                          </span>
                        ))}
                      </div>
                    )}
                  </InfoCard>
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
      <span className="text-[10px] uppercase tracking-wider opacity-50">
        {label}
      </span>
      <span className="ml-1.5 text-sm font-bold" style={{ color: "#c084fc" }}>
        {value}
      </span>
    </div>
  );
}

function InfoCard({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <div
      className="space-y-1 rounded-lg p-3"
      style={{
        background: "rgba(0, 0, 0, 0.3)",
        border: "1px solid rgba(124, 58, 237, 0.15)",
      }}
    >
      <p
        className="text-[10px] font-bold uppercase tracking-wider"
        style={{ color: "#a855f7" }}
      >
        {title}
      </p>
      {children}
    </div>
  );
}
