import { useCallback } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";
import { initializeDraftState } from "../draft/draft-engine";
import { logEvent } from "../logging";
import type { Tide } from "../types/cards";

/** Intro screen where the player picks one of 7 tides to draft from. */
export function QuestStartScreen() {
  const { state, mutations, cardDatabase } = useQuest();

  const handlePickTide = useCallback(
    (tide: Tide) => {
      mutations.setChosenTide(tide);

      const excludedTides = NAMED_TIDES.filter((t) => t !== tide);
      const playerHasBanes =
        state.deck.some((e) => e.isBane) ||
        state.dreamsigns.some((d) => d.isBane);
      const atlas = generateInitialAtlas(state.completionLevel, {
        cardDatabase,
        dreamsignPool: DREAMSIGNS,
        playerHasBanes,
        excludedTides,
      });

      const draftState = initializeDraftState(cardDatabase, tide);
      mutations.setDraftState(draftState);
      mutations.updateAtlas(atlas);

      // Auto-enter the first available dreamscape
      const firstNode = Object.values(atlas.nodes).find(
        (n) => n.status === "available",
      );

      logEvent("quest_started", {
        initialEssence: state.essence,
        chosenTide: tide,
        dreamscapesGenerated: Object.keys(atlas.nodes).length - 1,
      });

      if (firstNode) {
        mutations.setCurrentDreamscape(firstNode.id);
        mutations.setScreen({ type: "dreamscape" });
      } else {
        mutations.setScreen({ type: "atlas" });
      }
    },
    [state.completionLevel, state.essence, state.deck, state.dreamsigns, mutations, cardDatabase],
  );

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
        Choose Your Tide
      </motion.p>

      <motion.div
        className="flex flex-wrap justify-center gap-4"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.5 }}
      >
        {NAMED_TIDES.map((tide, index) => {
          const color = TIDE_COLORS[tide];
          return (
            <motion.button
              key={tide}
              className="flex cursor-pointer flex-col items-center rounded-xl px-6 py-5"
              style={{
                background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                border: `2px solid ${color}40`,
                boxShadow: `0 0 15px ${color}15`,
                minWidth: "120px",
              }}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4, delay: 0.6 + index * 0.07 }}
              whileHover={{
                boxShadow: `0 0 30px ${color}50`,
                borderColor: `${color}90`,
                scale: 1.08,
              }}
              whileTap={{ scale: 0.95 }}
              onClick={() => {
                handlePickTide(tide);
              }}
            >
              <img
                src={tideIconUrl(tide)}
                alt={tide}
                className="mb-3 h-14 w-14 rounded-full object-contain md:h-16 md:w-16"
                style={{ border: `2px solid ${color}` }}
              />
              <span
                className="text-lg font-bold"
                style={{ color }}
              >
                {tide}
              </span>
            </motion.button>
          );
        })}
      </motion.div>
    </div>
  );
}
