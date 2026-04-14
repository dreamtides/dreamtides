import { useCallback, useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import {
  selectDreamcallerOffer,
  toSelectedDreamcaller,
} from "../data/dreamcaller-selection";
import { dreamcallerAccentTide } from "../data/quest-content";
import { logEvent } from "../logging";
import type { DreamcallerContent } from "../types/content";
import type { SiteState } from "../types/quest";

interface DreamcallerCardProps {
  dreamcaller: DreamcallerContent;
  isSelected: boolean;
  isDismissed: boolean;
  onSelect: () => void;
}

/** Renders a single dreamcaller option with name, tide, ability, and bonus. */
function DreamcallerCard({
  dreamcaller,
  isSelected,
  isDismissed,
  onSelect,
}: DreamcallerCardProps) {
  const accentTide = dreamcallerAccentTide(dreamcaller);
  const tideColor = TIDE_COLORS[accentTide];

  return (
    <motion.div
      className="flex flex-col items-center rounded-xl px-5 py-6 md:px-6 md:py-8"
      style={{
        background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `2px solid ${tideColor}40`,
        boxShadow: `0 0 20px ${tideColor}15`,
        minWidth: "220px",
        maxWidth: "320px",
        flex: "1 1 0",
      }}
      animate={
        isSelected
          ? { scale: 1.08, boxShadow: `0 0 40px ${tideColor}60` }
          : isDismissed
            ? { opacity: 0, scale: 0.9 }
            : { scale: 1, opacity: 1 }
      }
      transition={{ duration: 0.5, ease: "easeOut" }}
    >
      {/* Tide icon */}
      <img
        src={tideIconUrl(accentTide)}
        alt={accentTide}
        className="mb-3 h-12 w-12 rounded-full object-contain md:h-14 md:w-14"
        style={{ border: `2px solid ${tideColor}` }}
      />

      {/* Dreamcaller name */}
      <h3
        className="mb-2 text-center text-xl font-bold leading-tight md:text-2xl"
        style={{ color: tideColor }}
      >
        {dreamcaller.name}
      </h3>

      {/* Awakening label */}
      <span
        className="mb-3 rounded-full px-3 py-0.5 text-xs font-medium"
        style={{
          background: `${tideColor}20`,
          color: tideColor,
          border: `1px solid ${tideColor}30`,
        }}
      >
        Awakening {String(dreamcaller.awakening)}
      </span>

      {/* Ability description */}
      <p
        className="mb-4 text-center text-sm leading-relaxed opacity-80"
        style={{ color: "#e2e8f0" }}
      >
        {dreamcaller.renderedText}
      </p>

      {/* Select button */}
      {!isSelected && !isDismissed && (
        <motion.button
          className="cursor-pointer rounded-lg px-6 py-2 text-sm font-bold text-white"
          style={{
            background: "linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%)",
            border: "2px solid rgba(168, 85, 247, 0.6)",
            boxShadow: "0 0 12px rgba(124, 58, 237, 0.3)",
          }}
          whileHover={{
            boxShadow: "0 0 20px rgba(124, 58, 237, 0.5)",
            scale: 1.05,
          }}
          whileTap={{ scale: 0.97 }}
          onClick={onSelect}
        >
          Select
        </motion.button>
      )}

      {isSelected && (
        <span
          className="rounded-full px-4 py-1.5 text-sm font-bold"
          style={{
            background: `${tideColor}20`,
            color: tideColor,
            border: `1px solid ${tideColor}`,
          }}
        >
          Selected
        </span>
      )}
    </motion.div>
  );
}

/** Screen for selecting a dreamcaller from 3 options. */
export function DreamcallerDraftScreen({ site }: { site: SiteState }) {
  const { mutations, questContent } = useQuest();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clear pending timer on unmount
  useEffect(() => {
    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  // Compute offered dreamcallers once on first render and keep stable.
  const offeredRef = useRef<DreamcallerContent[] | null>(null);
  if (offeredRef.current === null) {
    offeredRef.current = selectDreamcallerOffer(questContent.dreamcallers);
  }
  const offered = offeredRef.current;

  const handleSelect = useCallback(
    (index: number) => {
      if (selectedIndex !== null) return;

      const dreamcaller = offered[index];
      const selectedDreamcaller = toSelectedDreamcaller(dreamcaller);
      setSelectedIndex(index);

      mutations.setDreamcaller(selectedDreamcaller);

      logEvent("site_completed", {
        siteType: "DreamcallerDraft",
        outcome: `Selected ${dreamcaller.name}`,
        dreamcallerName: dreamcaller.name,
        dreamcallerAwakening: dreamcaller.awakening,
        selectedAccentTide: selectedDreamcaller.tide,
      });

      // After animation delay, return to dreamscape
      timerRef.current = setTimeout(() => {
        mutations.markSiteVisited(site.id);
        mutations.setScreen({ type: "dreamscape" });
      }, 500);
    },
    [selectedIndex, offered, mutations, site.id],
  );

  return (
    <motion.div
      className="flex min-h-screen flex-col items-center px-4 py-8 md:px-8 md:py-12"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      {/* Header */}
      <motion.h1
        className="mb-2 text-center text-3xl font-extrabold tracking-wide md:text-4xl"
        style={{
          background: "linear-gradient(135deg, #a855f7 0%, #7c3aed 50%, #c084fc 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          filter: "drop-shadow(0 0 20px rgba(168, 85, 247, 0.3))",
        }}
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.1 }}
      >
        Choose Your Dreamcaller
      </motion.h1>

      <motion.p
        className="mb-8 text-center text-sm opacity-50 md:mb-10 md:text-base"
        style={{ color: "#e2e8f0" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 0.5 }}
        transition={{ duration: 0.5, delay: 0.2 }}
      >
        Select a dreamcaller to guide your journey through the dreamscape
      </motion.p>

      {/* Dreamcaller cards */}
      <AnimatePresence>
        <motion.div
          className="flex w-full max-w-4xl flex-col items-center gap-4 md:flex-row md:items-stretch md:justify-center md:gap-6"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          {offered.map((dreamcaller, index) => (
            <DreamcallerCard
              key={dreamcaller.name}
              dreamcaller={dreamcaller}
              isSelected={selectedIndex === index}
              isDismissed={selectedIndex !== null && selectedIndex !== index}
              onSelect={() => {
                handleSelect(index);
              }}
            />
          ))}
        </motion.div>
      </AnimatePresence>
    </motion.div>
  );
}
