import { useCallback, useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { CardData } from "../types/cards";
import type { SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { CardDisplay } from "../components/CardDisplay";
import {
  countDeckTides,
  weightedSample,
  tideWeight,
} from "../data/tide-weights";
import { logEvent } from "../logging";

/** Displays cards weighted toward player tides; player picks 1 to add to pool. */
export function DraftSiteScreen({ site }: { site: SiteState }) {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();

  const draftCards = useMemo(() => {
    const deckTideCounts = countDeckTides(state.pool, cardDatabase);
    const allCards = Array.from(cardDatabase.values()).filter(
      (c) => c.rarity !== "Starter",
    );

    const selected = weightedSample<CardData>(
      allCards,
      config.draftSiteTotal,
      (card) => tideWeight(card.tide, deckTideCounts),
    );

    logEvent("site_entered", {
      siteType: "DraftSite",
      cardNames: selected.map((c) => c.name),
    });

    return selected;
  }, [site.id]);

  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [confirmed, setConfirmed] = useState(false);

  const handleSelect = useCallback(
    (index: number) => {
      if (confirmed) return;
      setSelectedIndex(index);
    },
    [confirmed],
  );

  const handleConfirm = useCallback(() => {
    if (selectedIndex === null || confirmed) return;
    setConfirmed(true);

    const pickedCard = draftCards[selectedIndex];
    mutations.addToPool(pickedCard.cardNumber, "draft_site");

    logEvent("site_completed", {
      siteType: "DraftSite",
      outcome: "drafted",
      pickedCard: pickedCard.name,
      pickedCardNumber: pickedCard.cardNumber,
    });
  }, [selectedIndex, confirmed, draftCards, mutations]);

  const handleContinue = useCallback(() => {
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site.id, mutations]);

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
          Draft
        </h2>
        <p className="mt-2 text-sm opacity-50">
          {confirmed ? "Card added to your pool!" : "Choose 1 card to keep"}
        </p>
      </div>

      {/* Card grid */}
      <motion.div
        className="mb-8 grid gap-4"
        style={{
          gridTemplateColumns: `repeat(${String(config.draftSiteTotal)}, minmax(150px, 200px))`,
        }}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
      >
        {draftCards.map((card, index) => {
          const isSelected = selectedIndex === index;
          const isDimmed = confirmed && !isSelected;
          return (
            <motion.div
              key={`${String(card.cardNumber)}-${String(index)}`}
              initial={{ opacity: 0, y: 30 }}
              animate={{
                opacity: isDimmed ? 0.3 : 1,
                y: 0,
                scale: isSelected && confirmed ? 1.05 : 1,
              }}
              transition={{ delay: 0.3 + index * 0.1, duration: 0.4 }}
              style={{ cursor: confirmed ? "default" : "pointer" }}
            >
              <CardDisplay
                card={card}
                onClick={() => handleSelect(index)}
                selected={isSelected}
                selectionColor="#a855f7"
              />
            </motion.div>
          );
        })}
      </motion.div>

      {/* Action buttons */}
      {!confirmed ? (
        <motion.button
          className="rounded-lg px-8 py-3 text-lg font-bold text-white"
          style={{
            background:
              selectedIndex !== null
                ? "linear-gradient(135deg, #7c3aedcc 0%, #7c3aed 100%)"
                : "#4b5563",
            boxShadow:
              selectedIndex !== null
                ? "0 0 20px rgba(124, 58, 237, 0.25)"
                : "none",
            cursor: selectedIndex !== null ? "pointer" : "not-allowed",
          }}
          disabled={selectedIndex === null}
          whileHover={selectedIndex !== null ? { scale: 1.05 } : {}}
          whileTap={selectedIndex !== null ? { scale: 0.97 } : {}}
          onClick={handleConfirm}
        >
          Confirm Pick
        </motion.button>
      ) : (
        <motion.button
          className="rounded-lg px-8 py-3 text-lg font-bold text-white"
          style={{
            background:
              "linear-gradient(135deg, #7c3aedcc 0%, #7c3aed 100%)",
            boxShadow: "0 0 20px rgba(124, 58, 237, 0.25)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={handleContinue}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3 }}
        >
          Continue
        </motion.button>
      )}
    </motion.div>
  );
}
