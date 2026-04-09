import { useCallback, useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { DREAMCALLERS } from "../data/dreamcallers";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { countDeckTides, selectRareRewards } from "../data/tide-weights";
import { CardDisplay } from "../components/CardDisplay";
import { logEvent } from "../logging";
import { generateNewNodes } from "../atlas/atlas-generator";
import { DREAMSIGNS } from "../data/dreamsigns";

type BattlePhase = "preBattle" | "animation" | "victory";

interface EnemyData {
  name: string;
  abilityText: string;
  dreamsignCount: number;
  tide: Tide;
}

/** Generates synthetic enemy data from the dreamcaller pool. */
function generateEnemy(): EnemyData {
  const template = DREAMCALLERS[Math.floor(Math.random() * DREAMCALLERS.length)];
  const prefixes = [
    "Shadow",
    "Nightmare",
    "Phantom",
    "Dark",
    "Cursed",
    "Twisted",
    "Fallen",
    "Spectral",
  ];
  const prefix = prefixes[Math.floor(Math.random() * prefixes.length)];
  const baseName = template.name.split(",")[0].split(" the ")[0];

  return {
    name: `${prefix} ${baseName}`,
    abilityText: template.abilityDescription,
    dreamsignCount: Math.floor(Math.random() * 5) + 1,
    tide: template.tides[0],
  };
}

/** Animates a number counting up from 0 to the target. */
function EssenceCountUp({ target, duration }: { target: number; duration: number }) {
  const [value, setValue] = useState(0);
  const frameRef = useRef(0);

  useEffect(() => {
    const startTime = performance.now();

    function tick(now: number) {
      const elapsed = now - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3);
      setValue(Math.round(target * eased));

      if (progress < 1) {
        frameRef.current = requestAnimationFrame(tick);
      }
    }

    frameRef.current = requestAnimationFrame(tick);

    return () => {
      cancelAnimationFrame(frameRef.current);
    };
  }, [target, duration]);

  return (
    <span className="text-3xl font-bold md:text-4xl" style={{ color: "#fbbf24" }}>
      +{String(value)}
    </span>
  );
}

interface PreBattleProps {
  enemy: EnemyData;
  completionLevel: number;
  isMiniboss: boolean;
  isFinalBoss: boolean;
  onStartBattle: () => void;
}

/** Pre-battle phase: shows enemy info and Start Battle button. */
function PreBattlePhase({
  enemy,
  completionLevel,
  isMiniboss,
  isFinalBoss,
  onStartBattle,
}: PreBattleProps) {
  let borderColor = "rgba(124, 58, 237, 0.3)";
  let accentColor = "#a855f7";
  let label = "Battle";

  if (isFinalBoss) {
    borderColor = "rgba(251, 191, 36, 0.5)";
    accentColor = "#fbbf24";
    label = "Final Boss Battle";
  } else if (isMiniboss) {
    borderColor = "rgba(239, 68, 68, 0.5)";
    accentColor = "#ef4444";
    label = "Miniboss Battle";
  }

  const tideColor = TIDE_COLORS[enemy.tide];

  return (
    <motion.div
      className="flex min-h-screen flex-col items-center justify-center px-4 py-8"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
    >
      <motion.div
        className="flex max-w-lg flex-col items-center rounded-xl px-6 py-8 md:px-10 md:py-10"
        style={{
          background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
          border: `2px solid ${borderColor}`,
          boxShadow: `0 0 30px ${borderColor}`,
        }}
        initial={{ scale: 0.9, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.5, delay: 0.1 }}
      >
        {/* Battle type label */}
        <span
          className="mb-4 rounded-full px-4 py-1 text-xs font-bold uppercase tracking-wider"
          style={{
            background: `${accentColor}20`,
            color: accentColor,
            border: `1px solid ${accentColor}40`,
          }}
        >
          {label}
        </span>

        {/* Completion level */}
        <span className="mb-6 text-sm opacity-50">
          Battle {String(completionLevel + 1)}/7
        </span>

        {/* Enemy tide icon */}
        <img
          src={tideIconUrl(enemy.tide)}
          alt={enemy.tide}
          className="mb-4 h-16 w-16 rounded-full object-contain md:h-20 md:w-20"
          style={{
            border: `3px solid ${tideColor}`,
            boxShadow: `0 0 20px ${tideColor}40`,
          }}
        />

        {/* Enemy name */}
        <h2
          className="mb-3 text-center text-2xl font-bold md:text-3xl"
          style={{ color: accentColor }}
        >
          {enemy.name}
        </h2>

        {/* Ability text */}
        <p
          className="mb-4 max-w-sm text-center text-sm leading-relaxed opacity-70"
          style={{ color: "#e2e8f0" }}
        >
          {enemy.abilityText}
        </p>

        {/* Dreamsign count */}
        <div className="mb-8 flex items-center gap-2">
          <span className="text-sm opacity-60">{"\u2728"}</span>
          <span className="text-sm font-medium opacity-70">
            {String(enemy.dreamsignCount)} Dreamsign{enemy.dreamsignCount !== 1 ? "s" : ""}
          </span>
        </div>

        {/* Start Battle button */}
        <motion.button
          className="cursor-pointer rounded-lg px-8 py-3 text-lg font-bold text-white"
          style={{
            background: `linear-gradient(135deg, ${accentColor} 0%, ${accentColor}cc 100%)`,
            border: `2px solid ${accentColor}80`,
            boxShadow: `0 0 20px ${accentColor}30`,
          }}
          whileHover={{
            boxShadow: `0 0 30px ${accentColor}50`,
            scale: 1.05,
          }}
          whileTap={{ scale: 0.97 }}
          onClick={onStartBattle}
        >
          Start Battle
        </motion.button>
      </motion.div>
    </motion.div>
  );
}

/** Battle animation phase with dramatic visual effects. */
function BattleAnimationPhase() {
  return (
    <motion.div
      className="flex min-h-screen flex-col items-center justify-center px-4"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      {/* Pulsing energy ring */}
      <motion.div
        className="relative flex items-center justify-center"
        style={{ width: 200, height: 200 }}
      >
        {/* Outer ring */}
        <motion.div
          className="absolute rounded-full"
          style={{
            width: 200,
            height: 200,
            border: "4px solid #a855f7",
            boxShadow: "0 0 40px #a855f780, inset 0 0 40px #a855f730",
          }}
          animate={{
            scale: [1, 1.3, 1],
            opacity: [1, 0.5, 1],
            boxShadow: [
              "0 0 40px #a855f780, inset 0 0 40px #a855f730",
              "0 0 80px #ef444480, inset 0 0 80px #ef444430",
              "0 0 40px #fbbf2480, inset 0 0 40px #fbbf2430",
            ],
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />

        {/* Inner ring */}
        <motion.div
          className="absolute rounded-full"
          style={{
            width: 120,
            height: 120,
            border: "3px solid #fbbf24",
            boxShadow: "0 0 30px #fbbf2460",
          }}
          animate={{
            scale: [1, 0.8, 1.2, 1],
            rotate: [0, 180, 360],
            opacity: [0.8, 1, 0.6, 0.8],
          }}
          transition={{
            duration: 1.2,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />

        {/* Center clash */}
        <motion.div
          className="text-4xl"
          animate={{
            scale: [1, 1.4, 1],
            opacity: [1, 0.7, 1],
          }}
          transition={{
            duration: 0.6,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        >
          {"\u2694\uFE0F"}
        </motion.div>
      </motion.div>

      {/* Screen shake effect via the container */}
      <motion.p
        className="mt-8 text-xl font-bold tracking-wider"
        style={{ color: "#a855f7" }}
        animate={{
          x: [-2, 2, -1, 1, 0],
          opacity: [0.5, 1, 0.5, 1, 0.5],
        }}
        transition={{
          duration: 0.4,
          repeat: Infinity,
          ease: "linear",
        }}
      >
        Battle in progress...
      </motion.p>
    </motion.div>
  );
}

interface VictoryPhaseProps {
  essenceReward: number;
  rareCards: CardData[];
  selectedRewardIndex: number | null;
  onSelectReward: (index: number) => void;
}

/** Victory phase: displays rewards and rare card picks. */
function VictoryPhase({
  essenceReward,
  rareCards,
  selectedRewardIndex,
  onSelectReward,
}: VictoryPhaseProps) {
  return (
    <motion.div
      className="flex min-h-screen flex-col items-center px-4 py-8 md:px-8 md:py-12"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
    >
      {/* Victory header */}
      <motion.h1
        className="mb-6 text-center text-4xl font-extrabold tracking-wide md:text-5xl"
        style={{
          background: "linear-gradient(135deg, #fbbf24 0%, #d4a017 50%, #f59e0b 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          filter: "drop-shadow(0 0 30px rgba(251, 191, 36, 0.4))",
        }}
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.6 }}
      >
        Victory!
      </motion.h1>

      {/* Essence reward */}
      <motion.div
        className="mb-8 flex flex-col items-center gap-2 rounded-xl px-8 py-4"
        style={{
          background: "rgba(212, 160, 23, 0.1)",
          border: "1px solid rgba(251, 191, 36, 0.3)",
        }}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.2 }}
      >
        <span className="text-sm font-medium opacity-60">Essence Earned</span>
        <div className="flex items-center gap-2">
          <span style={{ color: "#fbbf24" }}>{"\u25C6"}</span>
          <EssenceCountUp target={essenceReward} duration={800} />
        </div>
      </motion.div>

      {/* Card reward section */}
      <motion.div
        className="w-full max-w-4xl"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.4 }}
      >
        <h2
          className="mb-4 text-center text-lg font-bold md:text-xl"
          style={{ color: "#e2e8f0" }}
        >
          Choose a Rare Card
        </h2>

        <div className="grid grid-cols-2 gap-3 md:grid-cols-4 md:gap-4">
          <AnimatePresence>
            {rareCards.map((card, index) => {
              const isSelected = selectedRewardIndex === index;
              const isDismissed =
                selectedRewardIndex !== null && selectedRewardIndex !== index;

              return (
                <motion.div
                  key={card.cardNumber}
                  animate={
                    isSelected
                      ? { scale: 1.05 }
                      : isDismissed
                        ? { opacity: 0, scale: 0.9 }
                        : { scale: 1, opacity: 1 }
                  }
                  transition={{ duration: 0.4 }}
                >
                  <CardDisplay
                    card={card}
                    onClick={() => {
                      onSelectReward(index);
                    }}
                    selected={isSelected}
                    selectionColor="#fbbf24"
                  />
                </motion.div>
              );
            })}
          </AnimatePresence>
        </div>
      </motion.div>

    </motion.div>
  );
}

/** Full battle site screen with pre-battle, animation, and victory phases. */
export function BattleScreen({
  site,
  cardDatabase,
}: {
  site: SiteState;
  cardDatabase: Map<number, CardData>;
}) {
  const { state, mutations } = useQuest();
  const { completionLevel, atlas, currentDreamscape, deck } = state;

  const [phase, setPhase] = useState<BattlePhase>("preBattle");
  const [selectedRewardIndex, setSelectedRewardIndex] = useState<number | null>(null);
  const timersRef = useRef<ReturnType<typeof setTimeout>[]>([]);

  // Clear all pending timers on unmount
  useEffect(() => {
    return () => {
      for (const id of timersRef.current) {
        clearTimeout(id);
      }
    };
  }, []);

  const isMiniboss = completionLevel === 3;
  const isFinalBoss = completionLevel === 6;
  const essenceReward = 100 + completionLevel * 50;

  // Generate enemy and rare card rewards once on mount and keep stable.
  const enemyRef = useRef<EnemyData | null>(null);
  if (enemyRef.current === null) {
    enemyRef.current = generateEnemy();
  }
  const enemy = enemyRef.current;

  const rareCardsRef = useRef<CardData[] | null>(null);
  if (rareCardsRef.current === null) {
    const tideCounts = countDeckTides(deck, cardDatabase);
    rareCardsRef.current = selectRareRewards(cardDatabase, tideCounts, state.excludedTides);
  }
  const rareCards = rareCardsRef.current;

  const hasCompletedRef = useRef(false);

  const handleStartBattle = useCallback(() => {
    logEvent("battle_started", {
      completionLevel,
      enemyName: enemy.name,
      isMiniboss,
      isFinalBoss,
    });

    setPhase("animation");

    // After 1.5 seconds, transition to victory
    timersRef.current.push(
      setTimeout(() => {
        setPhase("victory");
      }, 1500),
    );
  }, [completionLevel, enemy.name, isMiniboss, isFinalBoss]);

  const handleSelectReward = useCallback(
    (index: number) => {
      if (selectedRewardIndex !== null || hasCompletedRef.current) return;
      hasCompletedRef.current = true;

      const card = rareCards[index];
      setSelectedRewardIndex(index);

      // Grant essence reward
      mutations.changeEssence(essenceReward, "battle_reward");

      // Add reward card to deck
      mutations.addCard(card.cardNumber, "battle_reward");

      // Mark site visited
      mutations.markSiteVisited(site.id);

      // Increment completion level (handles quest complete for level 7)
      mutations.incrementCompletionLevel(
        essenceReward,
        card.cardNumber,
        card.name,
        isMiniboss,
      );

      logEvent("site_completed", {
        siteType: "Battle",
        outcome: `Victory - earned ${String(essenceReward)} essence, card #${String(card.cardNumber)}`,
      });

      // Capture dreamscape info before the delayed callback to avoid
      // stale closure issues.
      const dreamscapeId = currentDreamscape;

      // After animation delay, navigate away and complete the dreamscape.
      // The dreamscape clearing is deferred to this callback so that the
      // screen transitions to "atlas" first. Clearing currentDreamscape
      // while the screen is still "site" would cause SiteScreen to show
      // "Site not found." and unmount BattleScreen, canceling this timer.
      timersRef.current.push(
        setTimeout(() => {
          // If final boss, incrementCompletionLevel already transitions
          // to quest complete. Otherwise go to atlas.
          if (!isFinalBoss) {
            mutations.setScreen({ type: "atlas" });
          }

          if (dreamscapeId) {
            const node = atlas.nodes[dreamscapeId];
            const playerHasBanes =
              state.deck.some((e) => e.isBane) ||
              state.dreamsigns.some((d) => d.isBane);
            const updatedAtlas = generateNewNodes(
              atlas,
              dreamscapeId,
              completionLevel,
              {
                cardDatabase,
                dreamsignPool: DREAMSIGNS,
                playerHasBanes,
                excludedTides: state.excludedTides,
              },
            );
            mutations.updateAtlas(updatedAtlas);

            logEvent("dreamscape_completed", {
              dreamscapeId,
              sitesVisitedCount: node?.sites.length ?? 0,
            });

            mutations.setCurrentDreamscape(null);
          }
        }, 800),
      );
    },
    [
      selectedRewardIndex,
      rareCards,
      mutations,
      essenceReward,
      site.id,
      currentDreamscape,
      atlas,
      completionLevel,
      isFinalBoss,
      isMiniboss,
      state.deck,
      state.dreamsigns,
      cardDatabase,
    ],
  );

  return (
    <AnimatePresence mode="wait">
      {phase === "preBattle" && (
        <motion.div key="pre-battle">
          <PreBattlePhase
            enemy={enemy}
            completionLevel={completionLevel}
            isMiniboss={isMiniboss}
            isFinalBoss={isFinalBoss}
            onStartBattle={handleStartBattle}
          />
        </motion.div>
      )}

      {phase === "animation" && (
        <motion.div key="animation">
          <BattleAnimationPhase />
        </motion.div>
      )}

      {phase === "victory" && (
        <motion.div key="victory">
          <VictoryPhase
            essenceReward={essenceReward}
            rareCards={rareCards}
            selectedRewardIndex={selectedRewardIndex}
            onSelectReward={handleSelectReward}
          />
        </motion.div>
      )}
    </AnimatePresence>
  );
}
