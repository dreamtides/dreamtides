import { useCallback, useState } from "react";
import { motion } from "framer-motion";
import type { SiteState } from "../types/quest";
import { CardDisplay } from "../components/CardDisplay";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";

/** Props for the PurgeSiteScreen component. */
interface PurgeSiteScreenProps {
  site: SiteState;
}

/** Renders the Purge site screen, allowing the player to remove cards from their deck. */
export function PurgeSiteScreen({ site }: PurgeSiteScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const { deck } = state;

  const maxPurge = site.isEnhanced ? 6 : 3;
  const [selectedEntryIds, setSelectedEntryIds] = useState<Set<string>>(
    new Set(),
  );

  const toggleSelection = useCallback(
    (entryId: string) => {
      setSelectedEntryIds((prev) => {
        const next = new Set(prev);
        if (next.has(entryId)) {
          next.delete(entryId);
        } else if (next.size < maxPurge) {
          next.add(entryId);
        }
        return next;
      });
    },
    [maxPurge],
  );

  const handlePurge = useCallback(() => {
    if (selectedEntryIds.size === 0) return;

    const purgedCardNumbers: number[] = [];
    for (const entryId of selectedEntryIds) {
      const entry = deck.find((e) => e.entryId === entryId);
      if (entry) {
        purgedCardNumbers.push(entry.cardNumber);
        mutations.removeCard(entryId, "purge");
      }
    }

    logEvent("purge_completed", {
      purgedCardNumbers,
      count: purgedCardNumbers.length,
    });

    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [selectedEntryIds, deck, mutations, site.id]);

  const handleClose = useCallback(() => {
    logEvent("site_completed", {
      siteType: "Purge",
      outcome: "skipped",
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [mutations, site.id]);

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
          style={{ color: "#ef4444" }}
        >
          Purge
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
            Enhanced -- Remove up to 6
          </span>
        )}
        <p className="mt-2 text-sm opacity-60">
          Select cards to permanently remove from your deck.
        </p>
      </div>

      {/* Selection counter */}
      <div
        className="mb-4 rounded-lg px-4 py-2 text-sm font-medium"
        style={{
          background: "rgba(239, 68, 68, 0.1)",
          border: "1px solid rgba(239, 68, 68, 0.3)",
          color: "#fca5a5",
        }}
      >
        Selected {String(selectedEntryIds.size)} of {String(maxPurge)}
      </div>

      {/* Deck grid */}
      <div className="grid w-full max-w-5xl grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
        {deck.map((entry, index) => {
          const card = cardDatabase.get(entry.cardNumber);
          if (!card) return null;
          const isSelected = selectedEntryIds.has(entry.entryId);
          return (
            <motion.div
              key={entry.entryId}
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.3, delay: index * 0.03 }}
            >
              <CardDisplay
                card={card}
                onClick={() => toggleSelection(entry.entryId)}
                selected={isSelected}
                selectionColor="#ef4444"
              />
            </motion.div>
          );
        })}
      </div>

      {/* Action buttons */}
      <div className="mt-8 flex gap-4">
        <button
          className="rounded-lg px-6 py-2.5 text-base font-bold transition-opacity"
          style={{
            background:
              selectedEntryIds.size > 0
                ? "linear-gradient(135deg, #ef4444 0%, #dc2626 100%)"
                : "#4b5563",
            color: selectedEntryIds.size > 0 ? "#ffffff" : "#9ca3af",
            opacity: selectedEntryIds.size > 0 ? 1 : 0.6,
            cursor: selectedEntryIds.size > 0 ? "pointer" : "not-allowed",
          }}
          disabled={selectedEntryIds.size === 0}
          onClick={handlePurge}
        >
          Purge {String(selectedEntryIds.size)} Card
          {selectedEntryIds.size !== 1 ? "s" : ""}
        </button>
        <button
          className="rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={handleClose}
        >
          Close
        </button>
      </div>

    </motion.div>
  );
}
