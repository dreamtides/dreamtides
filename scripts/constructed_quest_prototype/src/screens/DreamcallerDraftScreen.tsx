import { useCallback, useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Dreamcaller, SiteState } from "../types/quest";
import type { NamedTide, Tide } from "../types/cards";
import { useQuest } from "../state/quest-context";
import { DREAMCALLERS } from "../data/dreamcallers";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { countDeckTides, tideWeight, weightedSample } from "../data/tide-weights";
import { logEvent } from "../logging";

/** Returns the left neighbor of a named tide on the circle. */
function leftNeighbor(tide: NamedTide): NamedTide {
  const circle: NamedTide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
  const idx = circle.indexOf(tide);
  return circle[(idx + circle.length - 1) % circle.length];
}

/** Returns the right neighbor of a named tide on the circle. */
function rightNeighbor(tide: NamedTide): NamedTide {
  const circle: NamedTide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
  const idx = circle.indexOf(tide);
  return circle[(idx + 1) % circle.length];
}

/** Returns true if a dreamcaller's pair contains both specified tides. */
function matchesPair(dc: Dreamcaller, a: NamedTide, b: NamedTide): boolean {
  return (dc.tides[0] === a && dc.tides[1] === b) || (dc.tides[0] === b && dc.tides[1] === a);
}

/** Returns true if a dreamcaller's pair contains the specified tide. */
function containsTide(dc: Dreamcaller, tide: NamedTide): boolean {
  return dc.tides[0] === tide || dc.tides[1] === tide;
}

/**
 * Selects 3 dreamcallers for the draft:
 * 1. Left fork: pair containing startingTide + left neighbor
 * 2. Right fork: pair containing startingTide + right neighbor
 * 3. Adaptive: weighted sample from remaining callers
 */
function selectOfferedDreamcallers(
  startingTide: NamedTide | null,
  deck: Array<{ cardNumber: number }>,
  cardDatabase: Map<number, { tide: Tide }>,
): Dreamcaller[] {
  const pool = [...DREAMCALLERS];

  if (startingTide === null) {
    for (let i = pool.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [pool[i], pool[j]] = [pool[j], pool[i]];
    }
    return pool.slice(0, 3);
  }

  const left = leftNeighbor(startingTide);
  const right = rightNeighbor(startingTide);
  const selected: Dreamcaller[] = [];
  const usedNames = new Set<string>();

  // Slot 1: Left fork
  const leftCandidates = pool.filter((dc) => matchesPair(dc, startingTide, left));
  const leftFallback1 = pool.filter((dc) => containsTide(dc, startingTide) && !usedNames.has(dc.name));
  const leftFallback2 = pool.filter((dc) => (containsTide(dc, left) || containsTide(dc, right)) && !usedNames.has(dc.name));
  const leftPick = leftCandidates[0] ?? leftFallback1[0] ?? leftFallback2[0] ?? pool[0];
  selected.push(leftPick);
  usedNames.add(leftPick.name);

  // Slot 2: Right fork
  const rightCandidates = pool.filter((dc) => matchesPair(dc, startingTide, right) && !usedNames.has(dc.name));
  const rightFallback1 = pool.filter((dc) => containsTide(dc, startingTide) && !usedNames.has(dc.name));
  const rightFallback2 = pool.filter((dc) => (containsTide(dc, left) || containsTide(dc, right)) && !usedNames.has(dc.name));
  const rightPick = rightCandidates[0] ?? rightFallback1[0] ?? rightFallback2[0] ?? pool.filter((dc) => !usedNames.has(dc.name))[0] ?? pool[0];
  selected.push(rightPick);
  usedNames.add(rightPick.name);

  // Slot 3: Adaptive
  const remaining = pool.filter((dc) => !usedNames.has(dc.name));
  if (remaining.length === 0) {
    selected.push(pool.filter((dc) => !usedNames.has(dc.name))[0] ?? pool[0]);
  } else if (deck.length === 0) {
    selected.push(remaining[Math.floor(Math.random() * remaining.length)]);
  } else {
    const tideCounts = countDeckTides(deck, cardDatabase);
    const picked = weightedSample(remaining, 1, (dc) => {
      return Math.max(tideWeight(dc.tides[0], tideCounts), tideWeight(dc.tides[1], tideCounts));
    });
    selected.push(picked[0] ?? remaining[0]);
  }

  logEvent("dreamcaller_offers_generated", {
    offered: selected.map((dc) => ({ name: dc.name, tides: dc.tides })),
    startingTide,
  });

  return selected;
}

interface DreamcallerCardProps {
  dreamcaller: Dreamcaller;
  isSelected: boolean;
  isDismissed: boolean;
  onSelect: () => void;
}

function DreamcallerCard({
  dreamcaller,
  isSelected,
  isDismissed,
  onSelect,
}: DreamcallerCardProps) {
  const primaryColor = TIDE_COLORS[dreamcaller.tides[0]];
  const secondaryColor = TIDE_COLORS[dreamcaller.tides[1]];

  return (
    <motion.div
      className="flex flex-col items-center rounded-xl px-5 py-6 md:px-6 md:py-8"
      style={{
        background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `2px solid ${primaryColor}40`,
        boxShadow: `0 0 20px ${primaryColor}15`,
        minWidth: "220px",
        maxWidth: "320px",
        flex: "1 1 0",
      }}
      animate={
        isSelected
          ? { scale: 1.08, boxShadow: `0 0 40px ${primaryColor}60` }
          : isDismissed
            ? { opacity: 0, scale: 0.9 }
            : { scale: 1, opacity: 1 }
      }
      transition={{ duration: 0.5, ease: "easeOut" }}
    >
      <div className="mb-3 flex items-center gap-2">
        <img
          src={tideIconUrl(dreamcaller.tides[0])}
          alt={dreamcaller.tides[0]}
          className="h-12 w-12 rounded-full object-contain md:h-14 md:w-14"
          style={{ border: `2px solid ${primaryColor}` }}
        />
        <span className="text-lg opacity-40" style={{ color: "#e2e8f0" }}>+</span>
        <img
          src={tideIconUrl(dreamcaller.tides[1])}
          alt={dreamcaller.tides[1]}
          className="h-12 w-12 rounded-full object-contain md:h-14 md:w-14"
          style={{ border: `2px solid ${secondaryColor}` }}
        />
      </div>

      <h3
        className="mb-2 text-center text-xl font-bold leading-tight md:text-2xl"
        style={{ color: primaryColor }}
      >
        {dreamcaller.name}
      </h3>

      <div className="mb-3 flex items-center gap-2">
        <span
          className="rounded-full px-2 py-0.5 text-xs font-medium"
          style={{
            background: `${primaryColor}20`,
            color: primaryColor,
            border: `1px solid ${primaryColor}30`,
          }}
        >
          {dreamcaller.tides[0]}
        </span>
        <span
          className="rounded-full px-2 py-0.5 text-xs font-medium"
          style={{
            background: `${secondaryColor}20`,
            color: secondaryColor,
            border: `1px solid ${secondaryColor}30`,
          }}
        >
          {dreamcaller.tides[1]}
        </span>
      </div>

      <p
        className="mb-4 text-center text-sm leading-relaxed opacity-80"
        style={{ color: "#e2e8f0" }}
      >
        {dreamcaller.abilityDescription}
      </p>

      <div className="mb-2 flex items-center gap-1.5">
        <span style={{ color: "#fbbf24" }}>{"\u25C6"}</span>
        <span className="text-lg font-bold" style={{ color: "#fbbf24" }}>
          +{String(dreamcaller.essenceBonus)}
        </span>
        <span className="text-xs opacity-50">Essence</span>
      </div>

      <div className="mb-5 flex items-center gap-1.5">
        <img
          src={tideIconUrl(dreamcaller.tideCrystalGrant)}
          alt={dreamcaller.tideCrystalGrant}
          className="h-4 w-4 rounded-full object-contain"
        />
        <span className="text-sm font-medium" style={{ color: TIDE_COLORS[dreamcaller.tideCrystalGrant] }}>
          1 {dreamcaller.tideCrystalGrant} Crystal
        </span>
      </div>

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
            background: `${primaryColor}20`,
            color: primaryColor,
            border: `1px solid ${primaryColor}`,
          }}
        >
          Selected
        </span>
      )}
    </motion.div>
  );
}

export function DreamcallerDraftScreen({ site }: { site: SiteState }) {
  const { state, mutations, cardDatabase } = useQuest();
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  const offeredRef = useRef<Dreamcaller[] | null>(null);
  if (offeredRef.current === null) {
    offeredRef.current = selectOfferedDreamcallers(
      state.startingTide,
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

      mutations.setDreamcaller(dreamcaller);
      mutations.changeEssence(dreamcaller.essenceBonus, "dreamcaller_bonus");
      mutations.addTideCrystal(dreamcaller.tideCrystalGrant, 1);

      logEvent("site_completed", {
        siteType: "DreamcallerDraft",
        outcome: `Selected ${dreamcaller.name}`,
        dreamcallerName: dreamcaller.name,
        dreamcallerTides: dreamcaller.tides,
        essenceBonus: dreamcaller.essenceBonus,
      });

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
