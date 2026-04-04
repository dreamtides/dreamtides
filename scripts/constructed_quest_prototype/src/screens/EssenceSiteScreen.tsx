import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { logEvent } from "../logging";

/** Props for the EssenceSiteScreen component. */
interface EssenceSiteScreenProps {
  site: SiteState;
}

/** Returns a random integer between min and max (inclusive). */
function randomIntInRange(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

/** Grants essence with a count-up animation and auto-completes. */
export function EssenceSiteScreen({ site }: EssenceSiteScreenProps) {
  const { mutations } = useQuest();
  const config = useQuestConfig();

  const baseAmount = config.essenceSiteAmount;
  const essenceAmount = useMemo(
    () => {
      const base = baseAmount;
      const min = Math.round(base * 0.9);
      const max = Math.round(base * 1.2);
      const amount = randomIntInRange(min, max);
      return site.isEnhanced ? amount * 2 : amount;
    },
    [site.isEnhanced, baseAmount],
  );

  const [displayValue, setDisplayValue] = useState(0);
  const [phase, setPhase] = useState<"counting" | "done">("counting");

  const handleComplete = useCallback(() => {
    mutations.changeEssence(essenceAmount, "essence_site");
    logEvent("site_completed", {
      siteType: "Essence",
      outcome: `Granted ${String(essenceAmount)} essence`,
      isEnhanced: site.isEnhanced,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [essenceAmount, site, mutations]);

  // Count-up animation
  useEffect(() => {
    const duration = 800;
    const steps = 20;
    const interval = duration / steps;
    let step = 0;

    const timer = setInterval(() => {
      step += 1;
      const progress = step / steps;
      setDisplayValue(Math.round(essenceAmount * progress));

      if (step >= steps) {
        clearInterval(timer);
        setDisplayValue(essenceAmount);
        setPhase("done");
      }
    }, interval);

    return () => clearInterval(timer);
  }, [essenceAmount]);

  // Log site entry once on mount
  useEffect(() => {
    logEvent("site_entered", {
      siteType: "Essence",
      isEnhanced: site.isEnhanced,
    });
  }, [site.isEnhanced]);

  // Auto-complete after animation finishes
  useEffect(() => {
    if (phase !== "done") return;
    const timer = setTimeout(handleComplete, 600);
    return () => clearTimeout(timer);
  }, [phase, handleComplete]);

  return (
    <AnimatePresence>
      <motion.div
        className="flex min-h-full flex-col items-center justify-center gap-6 p-8"
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.9 }}
        transition={{ duration: 0.3 }}
      >
        {/* Gold particle effect */}
        <motion.div
          className="relative flex h-32 w-32 items-center justify-center"
          animate={{
            boxShadow: [
              "0 0 20px rgba(251, 191, 36, 0.3)",
              "0 0 60px rgba(251, 191, 36, 0.6)",
              "0 0 20px rgba(251, 191, 36, 0.3)",
            ],
          }}
          transition={{ duration: 1.2, repeat: Infinity }}
          style={{
            borderRadius: "50%",
            background:
              "radial-gradient(circle, rgba(251,191,36,0.2) 0%, rgba(251,191,36,0.05) 60%, transparent 100%)",
          }}
        >
          <motion.span
            className="text-5xl font-black"
            style={{ color: "#fbbf24" }}
            animate={{ scale: [1, 1.1, 1] }}
            transition={{ duration: 0.6, repeat: Infinity }}
          >
            +{String(displayValue)}
          </motion.span>
        </motion.div>

        <motion.p
          className="text-lg font-medium"
          style={{ color: "#fbbf24" }}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          Essence
        </motion.p>

        {site.isEnhanced && (
          <motion.span
            className="rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(168, 85, 247, 0.15)",
              color: "#c084fc",
              border: "1px solid rgba(168, 85, 247, 0.3)",
            }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.3 }}
          >
            Enhanced
          </motion.span>
        )}
      </motion.div>
    </AnimatePresence>
  );
}
