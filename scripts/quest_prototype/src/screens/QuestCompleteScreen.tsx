import { useEffect, useRef } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";

/** Final summary screen shown after winning 7 battles. */
export function QuestCompleteScreen() {
  const { state, mutations } = useQuest();
  const hasLoggedRef = useRef(false);

  useEffect(() => {
    if (!hasLoggedRef.current) {
      hasLoggedRef.current = true;
      const completedNodes = Object.values(state.atlas.nodes).filter(
        (n) => n.status === "completed",
      ).length;
      logEvent("quest_completed", {
        cardsInDeck: state.deck.length,
        essenceRemaining: state.essence,
        dreamscapesVisited: completedNodes,
        battlesWon: state.completionLevel,
      });
    }
  }, [state.atlas.nodes, state.completionLevel, state.deck.length, state.essence]);

  function handleNewQuest() {
    mutations.resetQuest();
  }

  const completedDreamscapes = Object.values(state.atlas.nodes).filter(
    (n) => n.status === "completed",
  ).length;

  const stats = [
    { label: "Battles Won", value: String(state.completionLevel) },
    { label: "Cards in Deck", value: String(state.deck.length) },
    { label: "Essence Remaining", value: String(state.essence) },
    { label: "Dreamscapes Visited", value: String(completedDreamscapes) },
  ];

  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4">
      <motion.h1
        className="mb-8 text-center text-5xl font-extrabold tracking-wide md:text-7xl"
        style={{
          background: "linear-gradient(135deg, #fbbf24 0%, #d4a017 50%, #f59e0b 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          filter: "drop-shadow(0 0 40px rgba(251, 191, 36, 0.4))",
        }}
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.7, ease: "easeOut" }}
      >
        Quest Complete!
      </motion.h1>

      <motion.div
        className="mb-10 grid grid-cols-2 gap-4 md:gap-6"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.3 }}
      >
        {stats.map((stat) => (
          <div
            key={stat.label}
            className="flex flex-col items-center rounded-lg px-6 py-4"
            style={{
              background: "rgba(255, 255, 255, 0.05)",
              border: "1px solid rgba(212, 160, 23, 0.3)",
            }}
          >
            <span
              className="text-3xl font-bold md:text-4xl"
              style={{ color: "#fbbf24" }}
            >
              {stat.value}
            </span>
            <span className="mt-1 text-sm opacity-60">{stat.label}</span>
          </div>
        ))}
      </motion.div>

      <motion.button
        className="cursor-pointer rounded-lg px-8 py-3 text-lg font-bold tracking-wide text-white transition-colors"
        style={{
          background: "linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%)",
          border: "2px solid rgba(168, 85, 247, 0.6)",
          boxShadow: "0 0 20px rgba(124, 58, 237, 0.3)",
        }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.6 }}
        whileHover={{
          boxShadow: "0 0 30px rgba(124, 58, 237, 0.5)",
          scale: 1.05,
        }}
        whileTap={{ scale: 0.97 }}
        onClick={handleNewQuest}
      >
        New Quest
      </motion.button>
    </div>
  );
}
