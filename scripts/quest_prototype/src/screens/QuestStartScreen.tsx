import { useCallback, useRef } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { selectDreamcallerOffer } from "../data/dreamcaller-selection";
import { structuralTidesForPackageTides } from "../data/structural-tides";
import { DreamcallerPortrait } from "../components/DreamcallerPortrait";
import { bootstrapQuestStart } from "./quest-start-bootstrap";
import type { DreamcallerContent } from "../types/content";

const DREAMCALLER_ACCENTS = ["#c084fc", "#fbbf24", "#7dd3fc"] as const;
const DREAMCALLER_HOVER_TRANSITION = { duration: 0.12, delay: 0 } as const;
const DREAMCALLER_TAP_TRANSITION = { duration: 0.08, delay: 0 } as const;

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
      bootstrapQuestStart({
        dreamcaller,
        state: {
          completionLevel: state.completionLevel,
          deck: state.deck,
          dreamsigns: state.dreamsigns,
          essence: state.essence,
        },
        mutations,
        cardDatabase,
        questContent,
      });
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
          const accentColor = DREAMCALLER_ACCENTS[
            index % DREAMCALLER_ACCENTS.length
          ];
          const structuralTides = structuralTidesForPackageTides(
            dreamcaller.mandatoryTides,
          );
          return (
            <motion.div
              key={dreamcaller.name}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4, delay: 0.6 + index * 0.1 }}
            >
              <motion.button
                className="flex cursor-pointer flex-col items-center rounded-xl px-5 py-6 md:px-6 md:py-8"
                style={{
                  background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                  border: `2px solid ${accentColor}40`,
                  boxShadow: `0 0 20px ${accentColor}15`,
                  minWidth: "260px",
                  maxWidth: "360px",
                  flex: "1 1 0",
                }}
                whileHover={{
                  boxShadow: `0 0 40px ${accentColor}50`,
                  borderColor: `${accentColor}90`,
                  scale: 1.05,
                  transition: DREAMCALLER_HOVER_TRANSITION,
                }}
                whileTap={{
                  scale: 0.97,
                  transition: DREAMCALLER_TAP_TRANSITION,
                }}
                onClick={() => {
                  handlePickDreamcaller(dreamcaller);
                }}
              >
                <DreamcallerPortrait
                  dreamcaller={dreamcaller}
                  variant="hero"
                  style={{
                    width: "100%",
                    marginBottom: 16,
                    boxShadow: `0 14px 28px ${accentColor}20`,
                  }}
                />
                <h3
                  className="text-center text-xl font-bold leading-tight md:text-2xl"
                  style={{ color: "#f8fafc" }}
                >
                  {dreamcaller.name}
                </h3>
                <p
                  className="mb-3 text-center text-sm italic opacity-80 md:text-base"
                  style={{ color: "#cbd5f5" }}
                >
                  {dreamcaller.title}
                </p>
                <span
                  className="mb-3 rounded-full px-3 py-0.5 text-xs font-medium"
                  style={{
                    background: `${accentColor}20`,
                    color: accentColor,
                    border: `1px solid ${accentColor}30`,
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
                {structuralTides.length > 0 && (
                  <div className="flex w-full flex-col items-center gap-2">
                    <span
                      className="text-[10px] font-semibold uppercase tracking-[0.28em]"
                      style={{ color: "#f8fafc", opacity: 0.6 }}
                    >
                      Structural Tides
                    </span>
                    <div className="flex flex-wrap justify-center gap-2">
                      {structuralTides.map((tide) => (
                        <span
                          key={tide.id}
                          className="group/structural relative"
                          data-structural-tide-chip={tide.id}
                          title={tide.hoverBlurb}
                        >
                          <span
                            className="inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-medium"
                            style={{
                              background: "#000000",
                              borderColor: "rgba(255, 255, 255, 0.16)",
                              color: "#ffffff",
                            }}
                          >
                            <i
                              aria-hidden="true"
                              className={`bx ${tide.iconClass} text-sm leading-none`}
                              data-structural-tide-icon={tide.id}
                            />
                            <span>{tide.displayName}</span>
                          </span>
                          <span
                            className="pointer-events-none absolute bottom-full left-1/2 z-20 mb-2 hidden w-56 -translate-x-1/2 rounded-lg border px-3 py-2 text-left text-xs leading-relaxed shadow-2xl group-hover/structural:block"
                            style={{
                              background: "#000000",
                              borderColor: "rgba(255, 255, 255, 0.16)",
                              color: "#ffffff",
                            }}
                            data-structural-tide-tooltip={tide.id}
                          >
                            {tide.hoverBlurb}
                          </span>
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </motion.button>
            </motion.div>
          );
        })}
      </motion.div>
    </div>
  );
}
