import { useCallback } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { logEvent } from "../logging";

/** Intro screen with a "Begin Quest" button that initializes the atlas. */
export function QuestStartScreen() {
  const { state, mutations } = useQuest();

  const handleBeginQuest = useCallback(() => {
    const atlas = generateInitialAtlas(state.completionLevel);
    const nodeCount = Object.keys(atlas.nodes).length - 1; // subtract nexus

    logEvent("quest_started", {
      initialEssence: state.essence,
      dreamscapesGenerated: nodeCount,
    });

    mutations.updateAtlas(atlas);
    mutations.setScreen({ type: "atlas" });
  }, [state.completionLevel, state.essence, mutations]);

  return (
    <motion.div
      className="flex h-full flex-col items-center justify-center gap-6 p-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.5 }}
    >
      <h1
        className="text-5xl font-bold tracking-wider md:text-6xl"
        style={{ color: "#a855f7" }}
      >
        Dreamtides
      </h1>
      <p
        className="text-lg opacity-70"
        style={{ color: "#e2e8f0" }}
      >
        A Roguelike Deckbuilding Quest
      </p>
      <div className="mt-4 rounded-lg border border-gray-700 bg-black/30 p-4 text-center text-sm opacity-60">
        <p>Starting essence: {String(state.essence)}</p>
      </div>
      <button
        className="mt-6 rounded-xl px-8 py-3 text-lg font-bold text-white shadow-lg transition-transform hover:scale-105"
        style={{
          background: "linear-gradient(135deg, #7c3aed, #a855f7)",
          boxShadow: "0 0 20px #7c3aed40",
        }}
        onClick={handleBeginQuest}
        aria-label="Begin Quest"
      >
        Begin Quest
      </button>
    </motion.div>
  );
}
