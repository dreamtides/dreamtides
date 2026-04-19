import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { BattleEnemyDescriptor, BattleInit } from "../battle/types";
import type { SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { buildCardSourceDebugState } from "../debug/card-source-debug";
import { logEvent } from "../logging";
import { BattleRewardSurface } from "../battle/components/BattleRewardSurface";
import { completeBattleSiteVictory } from "../battle/integration/battle-completion-bridge";
import { buttonVariant, radius, typography } from "../battle/design-tokens";

type BattlePhase = "preBattle" | "animation" | "victory";

type EnemyData = Pick<
  BattleEnemyDescriptor,
  "name" | "abilityText" | "dreamsignCount" | "tide"
>;

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
  // FIND-10-2 (Stage 4): pre-battle splash, "Battle in progress..." loader,
  // and Victory overlay now share the same panel surface: cyan design-token
  // border, slate-950 panel, `radius.md` rounding, and typography ramp.
  // Boss/miniboss emphasis is kept as a subtle eyebrow chip and accent ring
  // instead of a re-coloured panel, so the visual language is unified.
  let label = "Battle";
  let accentBadgeClass = "border-cyan-300/45 bg-cyan-400/10 text-cyan-100";
  let accentRing = "ring-0";
  if (isFinalBoss) {
    label = "Final Boss Battle";
    accentBadgeClass = "border-amber-300/60 bg-amber-400/15 text-amber-100";
    accentRing = "ring-1 ring-amber-300/40";
  } else if (isMiniboss) {
    label = "Miniboss Battle";
    accentBadgeClass = "border-rose-300/55 bg-rose-500/15 text-rose-100";
    accentRing = "ring-1 ring-rose-300/40";
  }

  const tideColor = TIDE_COLORS[enemy.tide];

  return (
    <motion.div
      data-battle-pre-surface=""
      className="flex min-h-screen flex-col items-center justify-center px-4 py-8"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4 }}
    >
      <motion.div
        className={[
          "flex max-w-lg flex-col items-center p-6 md:p-8",
          radius.md,
          "border border-cyan-300/25 bg-slate-950/92 shadow-2xl shadow-slate-950/60 backdrop-blur",
          accentRing,
        ].join(" ")}
        initial={{ scale: 0.96, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.45, delay: 0.05 }}
      >
        {/* Battle type label — rendered as a design-token chip. */}
        <span
          className={[
            "mb-3 rounded-md border px-3 py-1",
            typography.caption,
            "font-semibold uppercase tracking-[0.28em]",
            accentBadgeClass,
          ].join(" ")}
        >
          {label}
        </span>

        {/* Completion level — FIND-09-11: "Battle N of 7" with the 1-based
            current index so the header reads consistently with the footer's
            "Battles won X/7" completed count. */}
        <span
          data-battle-header-counter=""
          className={`mb-5 ${typography.caption} uppercase tracking-[0.22em] text-slate-500`}
        >
          Battle {String(completionLevel + 1)} of 7
        </span>

        {/* Enemy tide icon */}
        <img
          src={tideIconUrl(enemy.tide)}
          alt={enemy.tide}
          className="mb-3 h-16 w-16 rounded-full object-contain md:h-20 md:w-20"
          style={{
            border: `3px solid ${tideColor}`,
            boxShadow: `0 0 18px ${tideColor}40`,
          }}
        />

        {/* Enemy name */}
        <h2 className={`mb-2 text-center ${typography.heading} text-white`}>
          {enemy.name}
        </h2>

        {/* Ability text */}
        <p className={`mb-3 max-w-sm text-center ${typography.body} text-slate-300`}>
          {enemy.abilityText}
        </p>

        {/* Dreamsign count */}
        <div className="mb-6 flex items-center gap-2">
          <span className={`${typography.caption} text-slate-400`}>{"\u2728"}</span>
          <span className={`${typography.caption} font-medium text-slate-300`}>
            {String(enemy.dreamsignCount)} Dreamsign{enemy.dreamsignCount !== 1 ? "s" : ""}
          </span>
        </div>

        {/* Start Battle button — uses the design-token primary variant so
            hover / focus-visible / active / disabled states are identical
            to every other primary CTA in the app. */}
        <button
          type="button"
          data-battle-pre-start=""
          className={buttonVariant("primary")}
          onClick={onStartBattle}
        >
          Start Battle
        </button>
      </motion.div>
    </motion.div>
  );
}

/** Battle animation phase with dramatic visual effects. */
function BattleAnimationPhase() {
  // FIND-10-2 (Stage 4): the loader sits inside the same panel surface as
  // the pre-battle splash and the Victory overlay. Keep the ring animation
  // for drama, but recolour it with the cyan/amber design-token palette
  // so the panel chrome matches the rest of the battle chrome.
  return (
    <motion.div
      // FIND-09-10: explicitly tag the auto-resolve loader so QA and
      // agent-browser tooling can distinguish "battle auto-resolving" from
      // a hung playable battle.
      data-battle-loader=""
      data-mode="auto"
      className="flex min-h-screen flex-col items-center justify-center px-4"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.2 }}
    >
      <motion.div
        data-battle-loader-surface=""
        className={[
          "flex w-full max-w-lg flex-col items-center p-8",
          radius.md,
          "border border-cyan-300/25 bg-slate-950/92 shadow-2xl shadow-slate-950/60 backdrop-blur",
        ].join(" ")}
        initial={{ scale: 0.96, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ duration: 0.35 }}
      >
        {/* Pulsing energy ring */}
        <motion.div
          className="relative flex items-center justify-center"
          style={{ width: 160, height: 160 }}
        >
          {/* Outer ring — cyan player-side accent. */}
          <motion.div
            className="absolute rounded-full"
            style={{
              width: 160,
              height: 160,
              border: "3px solid rgba(103, 232, 249, 0.65)",
              boxShadow:
                "0 0 30px rgba(103, 232, 249, 0.35), inset 0 0 30px rgba(103, 232, 249, 0.18)",
            }}
            animate={{
              scale: [1, 1.2, 1],
              opacity: [1, 0.55, 1],
            }}
            transition={{
              duration: 1.5,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          />

          {/* Inner ring — amber "clash" accent. */}
          <motion.div
            className="absolute rounded-full"
            style={{
              width: 96,
              height: 96,
              border: "2px solid rgba(251, 191, 36, 0.85)",
              boxShadow: "0 0 22px rgba(251, 191, 36, 0.35)",
            }}
            animate={{
              scale: [1, 0.85, 1.12, 1],
              rotate: [0, 180, 360],
              opacity: [0.85, 1, 0.7, 0.85],
            }}
            transition={{
              duration: 1.2,
              repeat: Infinity,
              ease: "easeInOut",
            }}
          />
        </motion.div>

        <motion.p
          className={`mt-6 ${typography.subheading} tracking-[0.18em] uppercase text-cyan-100`}
          animate={{
            opacity: [0.55, 1, 0.55],
          }}
          transition={{
            duration: 1.6,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        >
          Battle in progress
        </motion.p>
      </motion.div>
    </motion.div>
  );
}

/** Full battle site screen with pre-battle, animation, and victory phases. */
export function AutoBattleScreen({
  battleInit,
  site,
}: {
  battleInit: BattleInit;
  site: SiteState;
}) {
  const { state, mutations } = useQuest();
  const { atlas, currentDreamscape, resolvedPackage } = state;
  const {
    battleId,
    completionLevelAtStart,
    isMiniboss,
    isFinalBoss,
    essenceReward,
    enemyDescriptor,
    rewardOptions,
  } = battleInit;

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

  // Enemy descriptor and reward options come from the frozen `BattleInit`
  // snapshot so the auto mode uses the same seeded bootstrap as the playable
  // path (bug-007). Session stability is a `BattleSiteRoute` responsibility.
  const enemy: EnemyData = enemyDescriptor;
  const rewardCards = rewardOptions;
  const cardSourceDebugState = useMemo(
    () =>
      phase === "victory"
        ? buildCardSourceDebugState(
          "Battle Rewards",
          "BattleReward",
          rewardCards,
          resolvedPackage,
        )
        : null,
    [phase, resolvedPackage, rewardCards],
  );

  const hasCompletedRef = useRef(false);

  useEffect(() => {
    mutations.setCardSourceDebug(cardSourceDebugState, "battle_reward_cards_shown");
  }, [cardSourceDebugState, mutations]);

  useEffect(
    () => () => {
      mutations.setCardSourceDebug(null, "battle_reward_cards_hidden");
    },
    [mutations],
  );

  const handleStartBattle = useCallback(() => {
    logEvent("battle_started", {
      completionLevel: completionLevelAtStart,
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
  }, [completionLevelAtStart, enemy.name, isMiniboss, isFinalBoss]);

  const handleSelectReward = useCallback(
    (index: number) => {
      if (selectedRewardIndex !== null || hasCompletedRef.current) return;
      hasCompletedRef.current = true;

      const card = rewardCards[index];
      setSelectedRewardIndex(index);
      completeBattleSiteVictory({
        battleId,
        siteId: site.id,
        dreamscapeId: currentDreamscape,
        completionLevelAtBattleStart: completionLevelAtStart,
        atlasSnapshot: atlas,
        selectedRewardCard: card,
        essenceReward,
        isMiniboss,
        isFinalBoss,
        playerHasBanes:
          state.deck.some((entry) => entry.isBane) ||
          state.dreamsigns.some((dreamsign) => dreamsign.isBane),
        mutations,
        postVictoryHandoffDelayMs: 800,
      });
    },
    [
      battleId,
      selectedRewardIndex,
      rewardCards,
      mutations,
      essenceReward,
      site.id,
      currentDreamscape,
      atlas,
      completionLevelAtStart,
      isFinalBoss,
      isMiniboss,
      state.deck,
      state.dreamsigns,
    ],
  );

  return (
    <AnimatePresence mode="wait">
      {phase === "preBattle" && (
        <motion.div key="pre-battle">
          <PreBattlePhase
            enemy={enemy}
            completionLevel={completionLevelAtStart}
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
          <BattleRewardSurface
            battleId={battleId}
            canCancel={false}
            enemyName={enemy.name}
            essenceReward={essenceReward}
            rewardCards={rewardCards}
            rewardSource="AUTO_BATTLE"
            selectedRewardIndex={selectedRewardIndex}
            onCancel={() => {
              // bug-112: Auto battles are linear simulations with no undo
              // stack — the cancel button is hidden (canCancel=false) and this
              // handler only exists to satisfy the prop contract.
            }}
            onSelectReward={handleSelectReward}
          />
        </motion.div>
      )}
    </AnimatePresence>
  );
}
