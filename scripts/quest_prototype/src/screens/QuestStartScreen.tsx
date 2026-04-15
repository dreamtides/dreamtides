import { useCallback, useRef } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import {
  selectDreamcallerOffer,
  toQuestDreamcaller,
} from "../data/dreamcaller-selection";
import { dreamcallerAccentTide } from "../data/quest-content";
import { createDreamsign } from "../data/dreamsigns";
import { initializeDraftState } from "../draft/draft-engine";
import { resolveDreamsignTemplates } from "../dreamsign/dreamsign-pool";
import { logEvent } from "../logging";
import type { DreamcallerContent } from "../types/content";

/** Intro screen where the player picks a dreamcaller to start the quest. */
export function QuestStartScreen() {
  const { state, mutations, cardDatabase, questContent } = useQuest();

  const offeredRef = useRef<DreamcallerContent[] | null>(null);
  if (offeredRef.current === null) {
    offeredRef.current = selectDreamcallerOffer(questContent.dreamcallers);
  }
  const offered = offeredRef.current;

  const handlePickDreamcaller = useCallback(
    (dreamcaller: DreamcallerContent) => {
      const selectedDreamcaller = toQuestDreamcaller(dreamcaller);
      const resolvedPackage = questContent.resolvedPackagesByDreamcallerId.get(
        dreamcaller.id,
      );

      if (resolvedPackage === undefined) {
        throw new Error(`Missing resolved package for ${dreamcaller.id}`);
      }

      mutations.setDreamcallerSelection(selectedDreamcaller, resolvedPackage);

      const playerHasBanes =
        state.deck.some((e) => e.isBane) ||
        state.dreamsigns.some((d) => d.isBane);
      const atlas = generateInitialAtlas(state.completionLevel, {
        cardDatabase,
        dreamsignPool: resolveDreamsignTemplates(
          resolvedPackage.dreamsignPoolIds,
          questContent.dreamsignTemplates,
        ).map((template) => createDreamsign(template)),
        playerHasBanes,
        selectedPackageTides: resolvedPackage.selectedTides,
      });

      const draftState = initializeDraftState(cardDatabase, resolvedPackage);
      mutations.setDraftState(draftState, "quest_start");
      mutations.updateAtlas(atlas);

      // Auto-enter the first available dreamscape
      const firstNode = Object.values(atlas.nodes).find(
        (n) => n.status === "available",
      );

      logEvent("quest_started", {
        initialEssence: state.essence,
        dreamcallerId: dreamcaller.id,
        dreamcallerName: dreamcaller.name,
        dreamcallerAwakening: dreamcaller.awakening,
        packageSummary: {
          mandatoryTides: resolvedPackage.mandatoryTides,
          optionalSubset: resolvedPackage.optionalSubset,
          selectedTides: resolvedPackage.selectedTides,
        },
        selectedPackageTides: resolvedPackage.selectedTides,
        draftPoolSize: resolvedPackage.draftPoolSize,
        dreamsignPoolSize: resolvedPackage.dreamsignPoolIds.length,
        dreamscapesGenerated: Object.keys(atlas.nodes).length - 1,
      });

      if (firstNode) {
        mutations.setCurrentDreamscape(firstNode.id);
        mutations.setScreen({ type: "dreamscape" });
      } else {
        mutations.setScreen({ type: "atlas" });
      }
    },
    [
      state.completionLevel,
      state.deck,
      state.dreamsigns,
      state.essence,
      mutations,
      cardDatabase,
      questContent.dreamsignTemplates,
      questContent.resolvedPackagesByDreamcallerId,
    ],
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
        Choose Your Dreamcaller
      </motion.p>

      <motion.div
        className="flex flex-col items-center gap-4 md:flex-row md:items-stretch md:gap-6"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6, delay: 0.5 }}
      >
        {offered.map((dreamcaller, index) => {
          const accentTide = dreamcallerAccentTide(dreamcaller);
          const color = TIDE_COLORS[accentTide];
          return (
            <motion.button
              key={dreamcaller.name}
              className="flex cursor-pointer flex-col items-center rounded-xl px-5 py-6 md:px-6 md:py-8"
              style={{
                background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                border: `2px solid ${color}40`,
                boxShadow: `0 0 20px ${color}15`,
                minWidth: "220px",
                maxWidth: "320px",
                flex: "1 1 0",
              }}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4, delay: 0.6 + index * 0.1 }}
              whileHover={{
                boxShadow: `0 0 40px ${color}50`,
                borderColor: `${color}90`,
                scale: 1.05,
              }}
              whileTap={{ scale: 0.97 }}
              onClick={() => {
                handlePickDreamcaller(dreamcaller);
              }}
            >
              <img
                src={tideIconUrl(accentTide)}
                alt={accentTide}
                className="mb-3 h-12 w-12 rounded-full object-contain md:h-14 md:w-14"
                style={{ border: `2px solid ${color}` }}
              />
              <h3
                className="mb-2 text-center text-xl font-bold leading-tight md:text-2xl"
                style={{ color }}
              >
                {dreamcaller.name}
              </h3>
              <span
                className="mb-3 rounded-full px-3 py-0.5 text-xs font-medium"
                style={{
                  background: `${color}20`,
                  color,
                  border: `1px solid ${color}30`,
                }}
              >
                Awakening {String(dreamcaller.awakening)}
              </span>
              <p
                className="mb-4 text-center text-sm leading-relaxed opacity-80"
                style={{ color: "#e2e8f0" }}
              >
                {dreamcaller.renderedText}
              </p>
            </motion.button>
          );
        })}
      </motion.div>
    </div>
  );
}
