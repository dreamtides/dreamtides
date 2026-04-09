import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { AnteState, SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { DREAMCALLERS } from "../data/dreamcallers";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { logEvent } from "../logging";
import { generateNewNodes } from "../atlas/atlas-generator";
import { DREAMSIGNS } from "../data/dreamsigns";
import { useQuestConfig } from "../state/quest-config";
import { CardDisplay } from "../components/CardDisplay";
import {
  generateOpponentAnteCards,
  aiEscalationDecision,
  resolveAnte,
} from "../ante/ante-logic";
import { DeckEditor } from "../components/DeckEditor";
import { weightedSample, tideWeight } from "../data/tide-weights";

type BattlePhase =
  | "preBattle"
  | "anteSelection"
  | "animation"
  | "escalation"
  | "victory"
  | "defeat";

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
  cardDatabase: Map<number, CardData>;
  onStartBattle: () => void;
}

/** Pre-battle phase: shows enemy info and Start Battle button. */
function PreBattlePhase({
  enemy,
  completionLevel,
  isMiniboss,
  isFinalBoss,
  cardDatabase,
  onStartBattle,
}: PreBattleProps) {
  const [showDeckEditor, setShowDeckEditor] = useState(false);

  const previewCards = useMemo(() => {
    const allCards = Array.from(cardDatabase.values());
    const deckTideCounts = new Map<Tide, number>([[enemy.tide, 10]]);
    return weightedSample(allCards, 3, (card) => tideWeight(card.tide, deckTideCounts));
  }, [cardDatabase, enemy.tide]);
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
        <div className="mb-6 flex items-center gap-2">
          <span className="text-sm opacity-60">{"\u2728"}</span>
          <span className="text-sm font-medium opacity-70">
            {String(enemy.dreamsignCount)} Dreamsign{enemy.dreamsignCount !== 1 ? "s" : ""}
          </span>
        </div>

        {/* Opponent preview cards */}
        {previewCards.length > 0 && (
          <div className="mb-8">
            <p className="mb-2 text-center text-xs font-semibold uppercase tracking-wider opacity-50">
              Key Cards
            </p>
            <div className="flex gap-3">
              {previewCards.map((card) => (
                <div key={card.cardNumber} style={{ width: 120 }}>
                  <CardDisplay card={card} />
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex gap-4">
          <motion.button
            className="cursor-pointer rounded-lg px-6 py-3 text-base font-bold text-white"
            style={{
              background: "rgba(100, 100, 100, 0.3)",
              border: "2px solid rgba(168, 85, 247, 0.4)",
            }}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.97 }}
            onClick={() => setShowDeckEditor(true)}
          >
            Edit Deck
          </motion.button>

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
        </div>
      </motion.div>

      {showDeckEditor && (
        <DeckEditor
          isOpen={showDeckEditor}
          onClose={() => setShowDeckEditor(false)}
          cardDatabase={cardDatabase}
        />
      )}
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

interface AnteSelectionProps {
  opponentCard: CardData | null;
  playerAnteCard: CardData | null;
  onAccept: (playerCardNumber: number) => void;
  onDecline: () => void;
}

/** Ante selection phase: shows opponent card and the randomly-chosen player ante card. */
function AnteSelectionPhase({
  opponentCard,
  playerAnteCard,
  onAccept,
  onDecline,
}: AnteSelectionProps) {
  return (
    <motion.div
      className="flex min-h-screen flex-col items-center px-4 py-8"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
    >
      <h2
        className="mb-2 text-center text-2xl font-bold md:text-3xl"
        style={{ color: "#fbbf24" }}
      >
        Ante
      </h2>
      <p className="mb-8 text-center text-sm opacity-60">
        Both sides wager a card. Win to claim your opponent&apos;s ante, or decline to fight without stakes.
      </p>

      {/* Side-by-side ante display */}
      <div className="mb-8 flex items-start gap-8">
        {/* Opponent's anted card */}
        {opponentCard && (
          <div>
            <p className="mb-2 text-center text-xs font-semibold uppercase tracking-wider opacity-50">
              Opponent&apos;s Ante
            </p>
            <div style={{ width: 180 }}>
              <CardDisplay card={opponentCard} />
            </div>
          </div>
        )}

        {/* Player's randomly chosen ante card */}
        {playerAnteCard && (
          <div>
            <p className="mb-2 text-center text-xs font-semibold uppercase tracking-wider opacity-50">
              Your Ante
            </p>
            <div style={{ width: 180 }}>
              <CardDisplay card={playerAnteCard} />
            </div>
          </div>
        )}
      </div>

      {/* Action buttons */}
      <div className="flex gap-4">
        <motion.button
          className="cursor-pointer rounded-lg px-6 py-3 text-base font-bold text-white"
          style={{
            background: "linear-gradient(135deg, #fbbf24 0%, #d4a017 100%)",
            border: "2px solid rgba(251, 191, 36, 0.5)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={() => {
            if (playerAnteCard) onAccept(playerAnteCard.cardNumber);
          }}
        >
          Accept Ante & Fight
        </motion.button>

        <motion.button
          className="cursor-pointer rounded-lg px-6 py-3 text-base font-bold text-white"
          style={{
            background: "rgba(100, 100, 100, 0.3)",
            border: "2px solid rgba(168, 85, 247, 0.4)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={onDecline}
        >
          Decline Ante & Fight
        </motion.button>
      </div>
    </motion.div>
  );
}

interface EscalationProps {
  secondOpponentCard: CardData | null;
  playerEscalationCard: CardData | null;
  onMatch: (playerCardNumber: number) => void;
  onConcede: () => void;
}

/** Escalation phase: shows both sides' escalation cards, player matches or concedes. */
function EscalationPhase({
  secondOpponentCard,
  playerEscalationCard,
  onMatch,
  onConcede,
}: EscalationProps) {
  return (
    <motion.div
      className="flex min-h-screen flex-col items-center px-4 py-8"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
    >
      <h2
        className="mb-2 text-center text-2xl font-bold md:text-3xl"
        style={{ color: "#ef4444" }}
      >
        Escalation!
      </h2>
      <p className="mb-8 text-center text-sm opacity-60">
        Your opponent raises the stakes! Match or concede (losing only your first ante card).
      </p>

      {/* Side-by-side escalation display */}
      <div className="mb-8 flex items-start gap-8">
        {secondOpponentCard && (
          <div>
            <p className="mb-2 text-center text-xs font-semibold uppercase tracking-wider opacity-50">
              Opponent&apos;s Escalation
            </p>
            <div style={{ width: 180 }}>
              <CardDisplay card={secondOpponentCard} />
            </div>
          </div>
        )}

        {playerEscalationCard && (
          <div>
            <p className="mb-2 text-center text-xs font-semibold uppercase tracking-wider opacity-50">
              Your Escalation
            </p>
            <div style={{ width: 180 }}>
              <CardDisplay card={playerEscalationCard} />
            </div>
          </div>
        )}
      </div>

      {/* Action buttons */}
      <div className="flex gap-4">
        <motion.button
          className="cursor-pointer rounded-lg px-6 py-3 text-base font-bold text-white"
          style={{
            background: "linear-gradient(135deg, #ef4444 0%, #dc2626 100%)",
            border: "2px solid rgba(239, 68, 68, 0.5)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={() => {
            if (playerEscalationCard) onMatch(playerEscalationCard.cardNumber);
          }}
        >
          Match & Fight
        </motion.button>

        <motion.button
          className="cursor-pointer rounded-lg px-6 py-3 text-base font-bold text-white"
          style={{
            background: "rgba(100, 100, 100, 0.3)",
            border: "2px solid rgba(168, 85, 247, 0.4)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={onConcede}
        >
          Concede
        </motion.button>
      </div>
    </motion.div>
  );
}

interface VictoryPhaseProps {
  essenceReward: number;
  cardsGained: CardData[];
  onContinue: () => void;
}

/** Victory phase: displays essence reward, ante trophies, and continue button. */
function VictoryPhase({ essenceReward, cardsGained, onContinue }: VictoryPhaseProps) {
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

      {/* Ante trophies */}
      {cardsGained.length > 0 && (
        <motion.div
          className="mb-8 flex flex-col items-center gap-3"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.3 }}
        >
          <span className="text-sm font-semibold uppercase tracking-wider opacity-60">
            Cards Won from Ante
          </span>
          <div className="flex gap-4">
            {cardsGained.map((card) => (
              <div key={card.cardNumber} style={{ width: 160 }}>
                <CardDisplay card={card} />
              </div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Continue button */}
      <motion.button
        className="rounded-lg px-8 py-3 text-lg font-bold text-white"
        style={{
          background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
          border: "1px solid rgba(168, 85, 247, 0.5)",
          boxShadow: "0 0 20px rgba(124, 58, 237, 0.3)",
        }}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.4 }}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.97 }}
        onClick={onContinue}
      >
        Continue
      </motion.button>
    </motion.div>
  );
}

interface DefeatPhaseProps {
  cardsLost: CardData[];
  onContinue: () => void;
}

/** Defeat phase (concession only): shows lost cards and continue button. */
function DefeatPhase({ cardsLost, onContinue }: DefeatPhaseProps) {
  return (
    <motion.div
      className="flex min-h-screen flex-col items-center px-4 py-8 md:px-8 md:py-12"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
    >
      {/* Defeat header */}
      <motion.h1
        className="mb-6 text-center text-4xl font-extrabold tracking-wide md:text-5xl"
        style={{
          background: "linear-gradient(135deg, #ef4444 0%, #dc2626 50%, #b91c1c 100%)",
          WebkitBackgroundClip: "text",
          WebkitTextFillColor: "transparent",
          filter: "drop-shadow(0 0 30px rgba(239, 68, 68, 0.4))",
        }}
        initial={{ opacity: 0, scale: 0.8 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.6 }}
      >
        Conceded
      </motion.h1>

      <p className="mb-6 text-center text-sm opacity-60">
        You conceded the battle. No essence reward.
      </p>

      {/* Cards lost */}
      {cardsLost.length > 0 && (
        <motion.div
          className="mb-8 flex flex-col items-center gap-3"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2 }}
        >
          <span
            className="text-sm font-semibold uppercase tracking-wider"
            style={{ color: "#ef4444" }}
          >
            Cards Lost
          </span>
          <div className="flex gap-4">
            {cardsLost.map((card) => (
              <div key={card.cardNumber} style={{ width: 160 }}>
                <CardDisplay card={card} tintColor="#ef4444" />
              </div>
            ))}
          </div>
        </motion.div>
      )}

      {/* Continue button */}
      <motion.button
        className="rounded-lg px-8 py-3 text-lg font-bold text-white"
        style={{
          background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
          border: "1px solid rgba(168, 85, 247, 0.5)",
          boxShadow: "0 0 20px rgba(124, 58, 237, 0.3)",
        }}
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.3 }}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.97 }}
        onClick={onContinue}
      >
        Continue
      </motion.button>
    </motion.div>
  );
}

/** Full battle site screen with pre-battle, ante, animation, escalation, and result phases. */
export function BattleScreen({
  site,
  cardDatabase,
}: {
  site: SiteState;
  cardDatabase: Map<number, CardData>;
}) {
  const { state, mutations } = useQuest();
  const config = useQuestConfig();
  const { completionLevel, atlas, currentDreamscape } = state;

  const [phase, setPhase] = useState<BattlePhase>("preBattle");
  const timersRef = useRef<ReturnType<typeof setTimeout>[]>([]);
  const [anteState, setLocalAnteState] = useState<AnteState | null>(null);

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
  const essenceReward = config.battleEssence + state.completionLevel * config.essencePerLevel;

  // Generate enemy once on mount and keep stable.
  const enemyRef = useRef<EnemyData | null>(null);
  if (enemyRef.current === null) {
    enemyRef.current = generateEnemy();
  }
  const enemy = enemyRef.current;

  // Generate opponent ante cards once on mount.
  const opponentAnteCardsRef = useRef<number[] | null>(null);
  if (opponentAnteCardsRef.current === null) {
    opponentAnteCardsRef.current = generateOpponentAnteCards(
      cardDatabase,
      state.pool,
      config,
    );
  }
  const opponentAnteCards = opponentAnteCardsRef.current;

  const hasCompletedRef = useRef(false);

  const handleStartBattle = useCallback(() => {
    logEvent("battle_started", {
      completionLevel,
      enemyName: enemy.name,
      isMiniboss,
      isFinalBoss,
    });

    if (config.anteEnabled && opponentAnteCards.length > 0) {
      setPhase("anteSelection");
    } else {
      setPhase("animation");
      timersRef.current.push(
        setTimeout(() => {
          setPhase("victory");
        }, 1500),
      );
    }
  }, [completionLevel, enemy.name, isMiniboss, isFinalBoss, config.anteEnabled, opponentAnteCards]);

  const handleAnteAccept = useCallback(
    (playerCardNumber: number) => {
      const newAnteState: AnteState = {
        anteAccepted: true,
        playerAnteCards: [playerCardNumber],
        opponentAnteCards: [opponentAnteCards[0]],
        escalationTriggered: false,
        playerConceded: false,
        escalationPhase: "none",
      };
      setLocalAnteState(newAnteState);
      mutations.setAnteState(newAnteState);

      logEvent("ante_accepted", {
        playerCard: playerCardNumber,
        opponentCard: opponentAnteCards[0],
      });

      setPhase("animation");

      // Ante accepted: first 750ms then escalation check
      timersRef.current.push(
        setTimeout(() => {
          const shouldEscalate =
            opponentAnteCards.length >= 2 && aiEscalationDecision(false);
          if (shouldEscalate) {
            setPhase("escalation");
          } else {
            setPhase("victory");
          }
        }, 750),
      );
    },
    [opponentAnteCards, mutations],
  );

  const handleAnteDecline = useCallback(() => {
    const newAnteState: AnteState = {
      anteAccepted: false,
      playerAnteCards: [],
      opponentAnteCards: [],
      escalationTriggered: false,
      playerConceded: false,
      escalationPhase: "none",
    };
    setLocalAnteState(newAnteState);
    mutations.setAnteState(newAnteState);

    logEvent("ante_declined", {});

    setPhase("animation");
    timersRef.current.push(
      setTimeout(() => {
        setPhase("victory");
      }, 1500),
    );
  }, [mutations]);

  const handleEscalationMatch = useCallback(
    (playerCardNumber: number) => {
      if (!anteState) return;

      const secondOpponent = opponentAnteCards[1];
      const updatedAnteState: AnteState = {
        ...anteState,
        playerAnteCards: [...anteState.playerAnteCards, playerCardNumber],
        opponentAnteCards: [...anteState.opponentAnteCards, secondOpponent],
        escalationTriggered: true,
        escalationPhase: "resolved",
      };
      setLocalAnteState(updatedAnteState);
      mutations.setAnteState(updatedAnteState);

      logEvent("escalation_matched", {
        playerCard: playerCardNumber,
        opponentCard: secondOpponent,
      });

      setPhase("animation");
      timersRef.current.push(
        setTimeout(() => {
          setPhase("victory");
        }, 750),
      );
    },
    [anteState, opponentAnteCards, mutations],
  );

  const handleEscalationConcede = useCallback(() => {
    if (!anteState) return;

    const updatedAnteState: AnteState = {
      ...anteState,
      playerConceded: true,
      escalationTriggered: true,
      escalationPhase: "resolved",
    };
    setLocalAnteState(updatedAnteState);
    mutations.setAnteState(updatedAnteState);

    logEvent("escalation_conceded", {});

    setPhase("defeat");
  }, [anteState, mutations]);

  // Resolve ante results for display
  const anteResult = useMemo(() => {
    if (!anteState) return { cardsGained: [] as number[], cardsLost: [] as number[] };
    return resolveAnte(anteState, true);
  }, [anteState]);

  const cardsGainedData = useMemo(
    () =>
      anteResult.cardsGained
        .map((cn) => cardDatabase.get(cn))
        .filter((c): c is CardData => c !== undefined),
    [anteResult.cardsGained, cardDatabase],
  );

  const defeatCardsLost = useMemo(() => {
    if (!anteState) return [];
    const result = resolveAnte(anteState, false);
    // If conceded, resolveAnte with playerConceded=true returns the first card
    return result.cardsLost
      .map((cn) => cardDatabase.get(cn))
      .filter((c): c is CardData => c !== undefined);
  }, [anteState, cardDatabase]);

  const handleVictoryContinue = useCallback(
    () => {
      if (hasCompletedRef.current) return;
      hasCompletedRef.current = true;

      // Grant essence reward
      mutations.changeEssence(essenceReward, "battle_reward");

      // Add opponent ante cards to pool if ante was accepted
      if (anteState?.anteAccepted) {
        for (const cardNum of anteResult.cardsGained) {
          mutations.addToPool(cardNum, "ante_win");
        }
      }

      // Clear ante state
      mutations.setAnteState(null);

      // Mark site visited
      mutations.markSiteVisited(site.id);

      // Increment completion level (handles quest complete for level 7)
      mutations.incrementCompletionLevel(isMiniboss);

      logEvent("site_completed", {
        siteType: "Battle",
        outcome: `Victory - earned ${String(essenceReward)} essence`,
      });

      const dreamscapeId = currentDreamscape;

      timersRef.current.push(
        setTimeout(() => {
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
              completionLevel + 1,
              {
                cardDatabase,
                dreamsignPool: DREAMSIGNS,
                playerHasBanes,
                startingTide: state.startingTide,
                playerPool: state.pool,
                config,
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
      mutations,
      essenceReward,
      anteState,
      anteResult.cardsGained,
      site.id,
      currentDreamscape,
      atlas,
      completionLevel,
      isFinalBoss,
      isMiniboss,
      state.deck,
      state.dreamsigns,
      state.startingTide,
      state.pool,
      cardDatabase,
      config,
    ],
  );

  const handleDefeatContinue = useCallback(() => {
    if (hasCompletedRef.current) return;
    hasCompletedRef.current = true;

    // Remove player's first ante card from pool
    if (anteState?.playerConceded && anteState.playerAnteCards.length > 0) {
      const lostCardNumber = anteState.playerAnteCards[0];
      const poolEntry = state.pool.find((e) => e.cardNumber === lostCardNumber);
      if (poolEntry) {
        mutations.removeFromPool(poolEntry.entryId, "ante_loss");
      }
    }

    // No essence reward on defeat
    // Clear ante state
    mutations.setAnteState(null);

    // Mark site visited
    mutations.markSiteVisited(site.id);

    logEvent("site_completed", {
      siteType: "Battle",
      outcome: "Defeat via concession",
    });

    // Return to dreamscape (do NOT increment completion level)
    mutations.setScreen({ type: "dreamscape" });
  }, [anteState, state.pool, mutations, site.id]);

  // Get the first opponent card for the ante selection display
  const firstOpponentCard =
    opponentAnteCards.length > 0 ? cardDatabase.get(opponentAnteCards[0]) ?? null : null;

  // Get the second opponent card for escalation display
  const secondOpponentCard =
    opponentAnteCards.length > 1 ? cardDatabase.get(opponentAnteCards[1]) ?? null : null;

  // Pre-generate random player ante cards (2 distinct random cards from deck)
  const playerAnteCardsRef = useRef<number[] | null>(null);
  if (playerAnteCardsRef.current === null) {
    const deckCards = state.deck.map((e) => e.cardNumber);
    const uniqueDeckCards = [...new Set(deckCards)];
    const shuffled = [...uniqueDeckCards].sort(() => Math.random() - 0.5);
    playerAnteCardsRef.current = shuffled.slice(0, 2);
  }
  const playerRandomAnteCards = playerAnteCardsRef.current;

  const playerAnteCard = playerRandomAnteCards.length > 0
    ? cardDatabase.get(playerRandomAnteCards[0]) ?? null : null;
  const playerEscalationCard = playerRandomAnteCards.length > 1
    ? cardDatabase.get(playerRandomAnteCards[1]) ?? null : null;

  return (
    <AnimatePresence mode="wait">
      {phase === "preBattle" && (
        <motion.div key="pre-battle">
          <PreBattlePhase
            enemy={enemy}
            completionLevel={completionLevel}
            isMiniboss={isMiniboss}
            isFinalBoss={isFinalBoss}
            cardDatabase={cardDatabase}
            onStartBattle={handleStartBattle}
          />
        </motion.div>
      )}

      {phase === "anteSelection" && (
        <motion.div key="ante-selection">
          <AnteSelectionPhase
            opponentCard={firstOpponentCard}
            playerAnteCard={playerAnteCard}
            onAccept={handleAnteAccept}
            onDecline={handleAnteDecline}
          />
        </motion.div>
      )}

      {phase === "animation" && (
        <motion.div key="animation">
          <BattleAnimationPhase />
        </motion.div>
      )}

      {phase === "escalation" && (
        <motion.div key="escalation">
          <EscalationPhase
            secondOpponentCard={secondOpponentCard}
            playerEscalationCard={playerEscalationCard}
            onMatch={handleEscalationMatch}
            onConcede={handleEscalationConcede}
          />
        </motion.div>
      )}

      {phase === "victory" && (
        <motion.div key="victory">
          <VictoryPhase
            essenceReward={essenceReward}
            cardsGained={cardsGainedData}
            onContinue={handleVictoryContinue}
          />
        </motion.div>
      )}

      {phase === "defeat" && (
        <motion.div key="defeat">
          <DefeatPhase
            cardsLost={defeatCardsLost}
            onContinue={handleDefeatContinue}
          />
        </motion.div>
      )}
    </AnimatePresence>
  );
}
