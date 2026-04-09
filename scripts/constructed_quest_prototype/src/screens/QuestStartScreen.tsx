import { useCallback, useRef } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { NAMED_TIDES, adjacentTides, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { DREAMSIGNS } from "../data/dreamsigns";
import { weightedSample } from "../data/tide-weights";
import { logEvent } from "../logging";
import { useQuestConfig } from "../state/quest-config";
import type { NamedTide, CardData } from "../types/cards";

/** Shuffles an array using Fisher-Yates. Returns a new array. */
function shuffled<T>(items: readonly T[]): T[] {
  const result = [...items];
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

/** Generates 3 distinct named tide options for quest start. */
function generateStartingTideOptions(): NamedTide[] {
  return shuffled(NAMED_TIDES).slice(0, 3) as NamedTide[];
}

/** Card numbers for starter cards with special copy counts. */
const STARTER_3_COPIES = [711, 713]; // Nocturne Strummer, Marked Direwolf
const STARTER_1_COPY = [717, 719]; // Flashpoint Detonation, Sign of Arrival

/** Builds the 30-card starting deck for a chosen tide. */
function buildStartingDeck(
  cardDatabase: Map<number, CardData>,
  startingTide: NamedTide,
): { starterCards: CardData[]; tideCards: CardData[]; neutralCards: CardData[] } {
  const allCards = Array.from(cardDatabase.values());

  // 20 Starter cards with specific copy counts:
  // 3x Nocturne Strummer, 3x Marked Direwolf
  // 1x Flashpoint Detonation, 1x Sign of Arrival
  // 2x each of the other 6 starters
  const starterCards: CardData[] = [];
  for (const card of allCards.filter((c) => c.rarity === "Starter")) {
    let copies = 2;
    if (STARTER_3_COPIES.includes(card.cardNumber)) copies = 3;
    if (STARTER_1_COPY.includes(card.cardNumber)) copies = 1;
    for (let i = 0; i < copies; i++) {
      starterCards.push(card);
    }
  }

  // 5 random cards from starting tide (excluding Starter, Legendary)
  const tideCandidates = allCards.filter(
    (c) =>
      c.tide === startingTide &&
      c.rarity !== "Starter" &&
      c.rarity !== "Legendary",
  );
  const tideCards = weightedSample(tideCandidates, 5, () => 1);

  // 5 random Neutral cards (excluding Starter, Legendary)
  const neutralCandidates = allCards.filter(
    (c) =>
      c.tide === "Neutral" &&
      c.rarity !== "Starter" &&
      c.rarity !== "Legendary",
  );
  const neutralCards = weightedSample(neutralCandidates, 5, () => 1);

  return { starterCards, tideCards, neutralCards };
}

/** Starting tide selection screen. */
export function QuestStartScreen() {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();

  // Generate stable tide options once
  const optionsRef = useRef<NamedTide[] | null>(null);
  if (optionsRef.current === null) {
    optionsRef.current = generateStartingTideOptions();
    logEvent("starting_tide_options_generated", {
      options: optionsRef.current,
    });
  }
  const options = optionsRef.current;

  const handleChooseTide = useCallback(
    (startingTide: NamedTide) => {
      // Set starting tide and grant crystal
      mutations.setStartingTide(startingTide);
      mutations.addTideCrystal(startingTide, 1);

      // Build starting deck
      const { starterCards, tideCards, neutralCards } = buildStartingDeck(
        cardDatabase,
        startingTide,
      );

      logEvent("starting_deck_initialized", {
        startingTide,
        starterCount: starterCards.length,
        tideCount: tideCards.length,
        neutralCount: neutralCards.length,
        totalDeckSize:
          starterCards.length + tideCards.length + neutralCards.length,
      });

      // Add all cards to pool
      const allCards = [...starterCards, ...tideCards, ...neutralCards];
      for (const card of allCards) {
        mutations.addToPool(card.cardNumber, "starter");
      }

      // Initialize deck from pool
      mutations.initializeDeckFromPool();

      // Adjust essence if needed
      if (config.startingEssence !== 250) {
        mutations.changeEssence(
          config.startingEssence - 250,
          "starting_essence",
        );
      }

      // Generate initial atlas
      const atlas = generateInitialAtlas(state.completionLevel, {
        cardDatabase,
        dreamsignPool: DREAMSIGNS,
        playerHasBanes: false,
        startingTide,
        playerPool: state.pool,
        config,
      });

      const firstNodeId = atlas.edges[0]?.[1];
      const nodeCount = Object.keys(atlas.nodes).length - 1;

      logEvent("quest_started", {
        initialEssence: config.startingEssence,
        dreamscapesGenerated: nodeCount,
        startingTide,
        startingDeckSize:
          starterCards.length + tideCards.length + neutralCards.length,
      });

      mutations.updateAtlas(atlas);

      if (firstNodeId) {
        mutations.setCurrentDreamscape(firstNodeId);
      }

      mutations.setScreen({ type: "viewStartingDeck" });
    },
    [state.completionLevel, mutations, cardDatabase, config],
  );

  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4">
      <motion.h1
        className="mb-2 text-center text-5xl font-extrabold tracking-wide md:text-7xl lg:text-8xl"
        style={{
          background:
            "linear-gradient(135deg, #a855f7 0%, #7c3aed 40%, #c084fc 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          textShadow:
            "0 0 60px rgba(168, 85, 247, 0.4), 0 0 120px rgba(124, 58, 237, 0.2)",
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
        Choose Your Starting Tide
      </motion.p>

      <motion.div
        className="flex w-full max-w-3xl flex-col items-center gap-4 md:flex-row md:justify-center md:gap-6"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.5 }}
      >
        {options.map((tide) => {
          const color = TIDE_COLORS[tide];
          const neighbors = adjacentTides(tide);
          return (
            <motion.button
              key={tide}
              className="flex w-full max-w-[260px] cursor-pointer flex-col items-center rounded-xl px-5 py-6"
              style={{
                background:
                  "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                border: `2px solid ${color}40`,
                boxShadow: `0 0 20px ${color}15`,
              }}
              whileHover={{
                boxShadow: `0 0 30px ${color}40`,
                scale: 1.05,
                borderColor: `${color}80`,
              }}
              whileTap={{ scale: 0.97 }}
              onClick={() => {
                handleChooseTide(tide);
              }}
            >
              <img
                src={tideIconUrl(tide)}
                alt={tide}
                className="mb-3 h-14 w-14 rounded-full object-contain"
                style={{ border: `2px solid ${color}` }}
              />
              <span
                className="mb-2 text-xl font-bold"
                style={{ color }}
              >
                {tide}
              </span>
              <span
                className="text-center text-xs leading-relaxed opacity-60"
                style={{ color: "#e2e8f0" }}
              >
                5 {tide} cards, 20 Starter cards, 5 Neutral cards, +1{" "}
                {tide} crystal
              </span>
              <span
                className="mt-2 text-center text-[10px] opacity-40"
                style={{ color: "#e2e8f0" }}
              >
                Neighbors: {neighbors.join(" & ")}
              </span>
            </motion.button>
          );
        })}
      </motion.div>
    </div>
  );
}
