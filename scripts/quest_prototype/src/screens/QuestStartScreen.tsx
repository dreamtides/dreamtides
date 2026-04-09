import { useCallback, useEffect, useMemo, useRef } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";
import { logEvent } from "../logging";
import {
  buildStartingDeckPlan,
  selectStartingTideOptions,
} from "../quest-start/quest-start-generator";
import { useQuestConfig } from "../state/quest-config";
import type { NamedTide } from "../types/cards";

/** Selects N random core tides to exclude (never Neutral). */
function selectExcludedTides(count: number): NamedTide[] {
  if (count <= 0) return [];
  const pool = [...NAMED_TIDES] as NamedTide[];
  const excluded: NamedTide[] = [];
  for (let i = 0; i < count && pool.length > 0; i++) {
    const idx = Math.floor(Math.random() * pool.length);
    excluded.push(pool[idx]);
    pool.splice(idx, 1);
  }
  return excluded;
}

interface StartingTideCardProps {
  tide: NamedTide;
  onSelect: () => void;
}

function StartingTideCard({ tide, onSelect }: StartingTideCardProps) {
  return (
    <motion.button
      className="flex cursor-pointer flex-col items-center rounded-2xl px-5 py-6 text-left text-white"
      style={{
        background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `2px solid ${TIDE_COLORS[tide]}40`,
        boxShadow: `0 0 24px ${TIDE_COLORS[tide]}18`,
      }}
      whileHover={{
        scale: 1.03,
        boxShadow: `0 0 32px ${TIDE_COLORS[tide]}35`,
      }}
      whileTap={{ scale: 0.98 }}
      onClick={onSelect}
    >
      <img
        src={tideIconUrl(tide)}
        alt={tide}
        className="mb-3 h-14 w-14 rounded-full object-contain"
        style={{ border: `2px solid ${TIDE_COLORS[tide]}` }}
      />
      <h2 className="text-2xl font-bold" style={{ color: TIDE_COLORS[tide] }}>
        {tide}
      </h2>
      <p className="mt-2 text-center text-sm leading-relaxed text-slate-200/80">
        Start with 10 {tide} cards, 10 Starter cards, 10 Neutral cards, and 1{" "}
        {tide} crystal.
      </p>
      <span
        className="mt-4 rounded-full px-3 py-1 text-xs font-bold uppercase tracking-wider"
        style={{
          background: `${TIDE_COLORS[tide]}20`,
          border: `1px solid ${TIDE_COLORS[tide]}60`,
          color: TIDE_COLORS[tide],
        }}
      >
        Select Tide
      </span>
    </motion.button>
  );
}

/** Intro screen with dark fantasy styling and "Begin Quest" button. */
export function QuestStartScreen() {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();
  const excludedTides = useMemo(
    () => selectExcludedTides(config.excludedTideCount),
    [config.excludedTideCount],
  );
  const startingTideOptions = useMemo(
    () => selectStartingTideOptions(excludedTides),
    [excludedTides],
  );
  const hasLoggedStartingTideOptions = useRef(false);

  useEffect(() => {
    if (hasLoggedStartingTideOptions.current) return;
    hasLoggedStartingTideOptions.current = true;
    logEvent("starting_tide_options_generated", {
      options: startingTideOptions,
    });
  }, [startingTideOptions]);

  const handleChooseStartingTide = useCallback((startingTide: NamedTide) => {
    mutations.setExcludedTides(excludedTides);
    const startingDeck = buildStartingDeckPlan(cardDatabase, startingTide);
    mutations.chooseStartingTide(
      startingTide,
      startingDeck.starterCardNumbers,
      startingDeck.tideCardNumbers,
      startingDeck.neutralCardNumbers,
      startingDeck.consumedRandomCardNumbers,
    );

    const atlas = generateInitialAtlas(state.completionLevel, {
      cardDatabase,
      dreamsignPool: DREAMSIGNS,
      playerHasBanes: false,
      excludedTides,
    });
    const nodeCount = Object.keys(atlas.nodes).length - 1; // subtract nexus

    logEvent("quest_started", {
      initialEssence: state.essence,
      initialDeckSize: startingDeck.deckCardNumbers.length,
      startingTide,
      dreamscapesGenerated: nodeCount,
      excludedTides,
    });

    mutations.updateAtlas(atlas);
    mutations.setScreen({ type: "atlas" });
  }, [
    cardDatabase,
    excludedTides,
    mutations,
    state.completionLevel,
    state.essence,
  ]);

  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4">
      <motion.h1
        className="mb-2 text-center text-5xl font-extrabold tracking-wide md:text-7xl lg:text-8xl"
        style={{
          background: "linear-gradient(135deg, #a855f7 0%, #7c3aed 40%, #c084fc 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          textShadow: "0 0 60px rgba(168, 85, 247, 0.4), 0 0 120px rgba(124, 58, 237, 0.2)",
          filter: "drop-shadow(0 0 40px rgba(168, 85, 247, 0.3))",
        }}
        initial={{ opacity: 0, y: -30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: "easeOut" }}
      >
        Dreamtides
      </motion.h1>

      <motion.p
        className="mb-10 text-center text-lg opacity-60 md:text-xl"
        style={{ color: "#e2e8f0" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 0.6 }}
        transition={{ duration: 0.8, delay: 0.3 }}
      >
        A Roguelike Deckbuilding Quest
      </motion.p>

      <motion.div
        className="grid w-full max-w-5xl grid-cols-1 gap-4 md:grid-cols-3 md:gap-6"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.5, ease: "easeOut" }}
      >
        {startingTideOptions.map((tide) => (
          <StartingTideCard
            key={tide}
            tide={tide}
            onSelect={() => {
              handleChooseStartingTide(tide);
            }}
          />
        ))}
      </motion.div>
    </div>
  );
}
