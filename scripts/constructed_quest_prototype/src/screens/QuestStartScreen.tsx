import { useCallback } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { NAMED_TIDES, adjacentTides } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";
import { weightedSample } from "../data/tide-weights";
import { logEvent } from "../logging";
import { useQuestConfig } from "../state/quest-config";
import type { Tide, CardData } from "../types/cards";

/** Energy cost bracket for a card. */
function costBracket(card: CardData): "low" | "mid" | "high" {
  const cost = card.energyCost ?? 0;
  if (cost <= 2) return "low";
  if (cost <= 4) return "mid";
  return "high";
}

/**
 * Enforces energy curve minimums by swapping cards between brackets.
 * Mutates the array in place.
 */
function enforceEnergyCurve(
  cards: CardData[],
  cardDatabase: Map<number, CardData>,
  minLow: number,
  minMid: number,
  minHigh: number,
): void {
  const count = (bracket: "low" | "mid" | "high") =>
    cards.filter((c) => costBracket(c) === bracket).length;

  const brackets: Array<"low" | "mid" | "high"> = ["low", "mid", "high"];
  const mins: Record<string, number> = { low: minLow, mid: minMid, high: minHigh };

  for (const needed of brackets) {
    while (count(needed) < mins[needed]) {
      // Find an over-represented bracket to swap from
      const donor = brackets.find(
        (b) => b !== needed && count(b) > mins[b],
      );
      if (!donor) break;

      // Find the donor card index in our array
      const donorIdx = cards.findIndex((c) => costBracket(c) === donor);
      if (donorIdx === -1) break;

      // Find a replacement card from the database in the needed bracket
      const existingNumbers = new Set(cards.map((c) => c.cardNumber));
      const replacements = Array.from(cardDatabase.values()).filter(
        (c) => costBracket(c) === needed && !existingNumbers.has(c.cardNumber),
      );
      if (replacements.length === 0) break;

      const replacement =
        replacements[Math.floor(Math.random() * replacements.length)];
      cards[donorIdx] = replacement;
    }
  }
}

/** Intro screen with dark fantasy styling and "Begin Quest" button. */
export function QuestStartScreen() {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();

  const handleBeginQuest = useCallback(() => {
    // Select center tide randomly from the 7 named tides
    const centerTide =
      NAMED_TIDES[Math.floor(Math.random() * NAMED_TIDES.length)];

    // Determine starting tides
    let selectedTides: Tide[];
    if (config.sequentialTides) {
      const neighbors = adjacentTides(centerTide);
      selectedTides = [centerTide, ...neighbors];
    } else {
      // Pick config.startingTides random distinct tides
      const pool = [...NAMED_TIDES];
      selectedTides = [];
      for (let i = 0; i < config.startingTides && pool.length > 0; i++) {
        const idx = Math.floor(Math.random() * pool.length);
        selectedTides.push(pool[idx]);
        pool.splice(idx, 1);
      }
    }

    mutations.setStartingTides(selectedTides);

    // Generate tide-colored starter cards
    const cardsPerTide = Math.floor(config.initialCards / selectedTides.length);
    const allCards = Array.from(cardDatabase.values());
    const starterCards: CardData[] = [];

    for (let t = 0; t < selectedTides.length; t++) {
      const tide = selectedTides[t];
      const tidePool = allCards.filter((c) => c.tide === tide);
      const count =
        t < config.initialCards % selectedTides.length
          ? cardsPerTide + 1
          : cardsPerTide;
      const picked = weightedSample(tidePool, count, () => 1);
      starterCards.push(...picked);
    }
    // Trim to exactly initialCards
    starterCards.length = Math.min(starterCards.length, config.initialCards);

    // Generate neutral cards
    const neutralPool = allCards.filter((c) => c.tide === "Neutral");
    const neutralCards = weightedSample(
      neutralPool,
      config.starterNeutral,
      () => 1,
    );

    const allStarterCards = [...starterCards, ...neutralCards];

    // Energy curve enforcement
    enforceEnergyCurve(
      allStarterCards,
      cardDatabase,
      config.starterLowCost,
      config.starterMidCost,
      config.starterHighCost,
    );

    // Add all cards to pool
    for (const card of allStarterCards) {
      mutations.addToPool(card.cardNumber, "starter");
    }

    // Initialize deck from pool
    mutations.initializeDeckFromPool();

    // Adjust essence if needed
    if (config.startingEssence !== 250) {
      mutations.changeEssence(config.startingEssence - 250, "starting_essence");
    }

    // Generate initial atlas (level 0 produces single node with fixed composition)
    const atlas = generateInitialAtlas(state.completionLevel, {
      cardDatabase,
      dreamsignPool: DREAMSIGNS,
      playerHasBanes: false,
      startingTides: selectedTides,
      playerPool: state.pool,
      config,
    });

    const firstNodeId = atlas.edges[0]?.[1];
    const nodeCount = Object.keys(atlas.nodes).length - 1;

    logEvent("quest_started", {
      initialEssence: config.startingEssence,
      dreamscapesGenerated: nodeCount,
      startingTides: selectedTides,
      starterCardCount: allStarterCards.length,
    });

    mutations.updateAtlas(atlas);

    // Set current dreamscape to first non-nexus node
    if (firstNodeId) {
      mutations.setCurrentDreamscape(firstNodeId);
    }

    // Navigate to dreamscape screen
    mutations.setScreen({ type: "dreamscape" });
  }, [
    state.completionLevel,
    mutations,
    cardDatabase,
    config,
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
