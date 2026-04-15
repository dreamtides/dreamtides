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
    <div className="flex min-h-screen flex-col items-center justify-center px-4 py-6">
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
        className="mb-6 text-center text-lg opacity-60 md:text-xl"
        style={{ color: "#e2e8f0" }}
        initial={{ opacity: 0 }}
        animate={{ opacity: 0.6 }}
        transition={{ duration: 0.8, delay: 0.3 }}
      >
        Choose Your Dreamcaller
      </motion.p>

      <motion.div
        className="flex flex-col items-center gap-3 md:flex-row md:items-start md:gap-5"
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
              className="flex w-full max-w-[286px] flex-col items-center gap-3"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.4, delay: 0.6 + index * 0.1 }}
            >
              <motion.button
                className="flex h-[472px] w-full cursor-pointer flex-col rounded-[22px] px-4 pt-4 pb-5 text-left md:px-5"
                style={{
                  background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                  border: `2px solid ${accentColor}40`,
                  boxShadow: `0 0 20px ${accentColor}15`,
                }}
                whileHover={{
                  boxShadow: `0 0 40px ${accentColor}50`,
                  borderColor: `${accentColor}90`,
                  y: -4,
                  transition: DREAMCALLER_HOVER_TRANSITION,
                }}
                whileTap={{
                  scale: 0.985,
                  transition: DREAMCALLER_TAP_TRANSITION,
                }}
                onClick={() => {
                  handlePickDreamcaller(dreamcaller);
                }}
              >
                <div className="mb-3 flex min-h-[78px] items-start justify-between gap-3">
                  <div className="min-w-0 pr-2">
                    <h3
                      className="text-xl font-bold leading-tight md:text-2xl"
                      style={{ color: "#f8fafc" }}
                    >
                      {dreamcaller.name}
                    </h3>
                    <p
                      className="mt-1 text-sm italic opacity-80 md:text-base"
                      style={{ color: "#cbd5f5" }}
                    >
                      {dreamcaller.title}
                    </p>
                  </div>
                  <span
                    className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-sm font-bold"
                    style={{
                      background: "rgba(5, 2, 10, 0.42)",
                      color: accentColor,
                      border: `2px solid ${accentColor}55`,
                      boxShadow: `0 0 18px ${accentColor}22`,
                    }}
                    aria-label={`Awakening ${String(dreamcaller.awakening)}`}
                  >
                    {String(dreamcaller.awakening)}
                  </span>
                </div>
                <DreamcallerPortrait
                  dreamcaller={dreamcaller}
                  variant="panel"
                  style={{
                    width: "100%",
                    aspectRatio: "0.8 / 1",
                    marginBottom: 14,
                    borderRadius: 18,
                    boxShadow: `0 14px 28px ${accentColor}16`,
                  }}
                />
                <p
                  className="flex flex-1 items-center justify-center px-2 text-center text-sm leading-relaxed opacity-80"
                  style={{ color: "#e2e8f0" }}
                >
                  {dreamcaller.renderedText}
                </p>
              </motion.button>
              {structuralTides.length > 0 && (
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
              )}
            </motion.div>
          );
        })}
      </motion.div>
    </div>
  );
}
