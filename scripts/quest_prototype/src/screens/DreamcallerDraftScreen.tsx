import { useCallback, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Dreamcaller, SiteState } from "../types/quest";
import type { Tide } from "../types/cards";
import { useQuest } from "../state/quest-context";
import { DREAMCALLERS } from "../data/dreamcallers";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { logEvent } from "../logging";

/**
 * Counts the occurrences of each tide in the player's deck, using
 * the card database to look up tide values.
 */
function countDeckTides(
  deck: Array<{ cardNumber: number }>,
  cardDatabase: Map<number, { tide: Tide }>,
): Map<Tide, number> {
  const counts = new Map<Tide, number>();
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card) {
      counts.set(card.tide, (counts.get(card.tide) ?? 0) + 1);
    }
  }
  return counts;
}

/**
 * Selects 3 distinct dreamcallers. If the player has drafted cards,
 * weights the selection toward tides that match the player's deck.
 * Otherwise picks 3 at random.
 */
function selectOfferedDreamcallers(
  deck: Array<{ cardNumber: number }>,
  cardDatabase: Map<number, { tide: Tide }>,
): Dreamcaller[] {
  const pool = [...DREAMCALLERS];

  if (deck.length === 0) {
    // Shuffle and pick 3
    for (let i = pool.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [pool[i], pool[j]] = [pool[j], pool[i]];
    }
    return pool.slice(0, 3);
  }

  // Weight toward player's most-drafted tides
  const tideCounts = countDeckTides(deck, cardDatabase);
  const maxCount = Math.max(...tideCounts.values(), 1);

  const weighted: Array<[Dreamcaller, number]> = pool.map((dc) => {
    const tideCount = tideCounts.get(dc.tide) ?? 0;
    // Base weight 1 + proportion of max tide count
    const weight = 1 + (tideCount / maxCount) * 3;
    return [dc, weight];
  });

  const selected: Dreamcaller[] = [];
  const remaining = [...weighted];

  for (let pick = 0; pick < 3 && remaining.length > 0; pick++) {
    const total = remaining.reduce((sum, [, w]) => sum + w, 0);
    let roll = Math.random() * total;
    let chosenIndex = remaining.length - 1;

    for (let i = 0; i < remaining.length; i++) {
      roll -= remaining[i][1];
      if (roll <= 0) {
        chosenIndex = i;
        break;
      }
    }

    selected.push(remaining[chosenIndex][0]);
    remaining.splice(chosenIndex, 1);
  }

  return selected;
}

interface DreamcallerCardProps {
  dreamcaller: Dreamcaller;
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
  const tideColor = TIDE_COLORS[dreamcaller.tide];

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
        src={tideIconUrl(dreamcaller.tide)}
        alt={dreamcaller.tide}
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

      {/* Tide label */}
      <span
        className="mb-3 rounded-full px-3 py-0.5 text-xs font-medium"
        style={{
          background: `${tideColor}20`,
          color: tideColor,
          border: `1px solid ${tideColor}30`,
        }}
      >
        {dreamcaller.tide} Tide
      </span>

      {/* Ability description */}
      <p
        className="mb-4 text-center text-sm leading-relaxed opacity-80"
        style={{ color: "#e2e8f0" }}
      >
        {dreamcaller.abilityDescription}
      </p>

      {/* Essence bonus */}
      <div className="mb-2 flex items-center gap-1.5">
        <span style={{ color: "#fbbf24" }}>{"\u25C6"}</span>
        <span className="text-lg font-bold" style={{ color: "#fbbf24" }}>
          +{String(dreamcaller.essenceBonus)}
        </span>
        <span className="text-xs opacity-50">Essence</span>
      </div>

      {/* Tide crystal grant */}
      <div className="mb-5 flex items-center gap-1.5">
        <img
          src={tideIconUrl(dreamcaller.tideCrystalGrant)}
          alt={dreamcaller.tideCrystalGrant}
          className="h-4 w-4 rounded-full object-contain"
        />
        <span className="text-sm font-medium" style={{ color: tideColor }}>
          1 {dreamcaller.tideCrystalGrant} Crystal
        </span>
      </div>

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
  const { state, mutations, cardDatabase } = useQuest();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);

  // Compute offered dreamcallers once on first render and keep stable.
  const offeredRef = useRef<Dreamcaller[] | null>(null);
  if (offeredRef.current === null) {
    offeredRef.current = selectOfferedDreamcallers(
      state.deck,
      cardDatabase,
    );
  }
  const offered = offeredRef.current;

  const handleSelect = useCallback(
    (index: number) => {
      if (selectedIndex !== null) return;

      const dreamcaller = offered[index];
      setSelectedIndex(index);

      // Apply state mutations
      mutations.setDreamcaller(dreamcaller);
      mutations.changeEssence(dreamcaller.essenceBonus, "dreamcaller_bonus");
      mutations.addTideCrystal(dreamcaller.tideCrystalGrant, 1);

      logEvent("site_completed", {
        siteType: "DreamcallerDraft",
        outcome: `Selected ${dreamcaller.name}`,
        dreamcallerName: dreamcaller.name,
        dreamcallerTide: dreamcaller.tide,
        essenceBonus: dreamcaller.essenceBonus,
      });

      // After animation delay, return to dreamscape
      setTimeout(() => {
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
