import { useCallback } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { logEvent } from "../logging";

/** Intro screen with dark fantasy styling and "Begin Quest" button. */
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

      <motion.button
        className="cursor-pointer rounded-lg px-10 py-4 text-lg font-bold tracking-wide text-white transition-colors md:text-xl"
        style={{
          background: "linear-gradient(135deg, #d4a017 0%, #b8860b 100%)",
          border: "2px solid rgba(251, 191, 36, 0.6)",
          boxShadow:
            "0 0 20px rgba(212, 160, 23, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.15)",
        }}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.5, ease: "easeOut" }}
        whileHover={{
          boxShadow:
            "0 0 30px rgba(212, 160, 23, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.2)",
          scale: 1.05,
        }}
        whileTap={{ scale: 0.97 }}
        onClick={handleBeginQuest}
      >
        Begin Quest
      </motion.button>
    </div>
  );
}
