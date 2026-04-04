import { useCallback, useState } from "react";
import { motion } from "framer-motion";
import type { ProvisionerOption, SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { siteTypeName } from "../atlas/atlas-generator";
import { generateProvisionerOptions } from "../provisioner/provisioner-logic";
import { useQuestConfig } from "../state/quest-config";
import { logEvent } from "../logging";

/** Props for the ProvisionerScreen component. */
interface ProvisionerScreenProps {
  site: SiteState;
}

/** Displays purchasable site options that get added to the current dreamscape. */
export function ProvisionerScreen({ site }: ProvisionerScreenProps) {
  const { state, mutations } = useQuest();
  const config = useQuestConfig();

  const [options, setOptions] = useState<ProvisionerOption[]>(() =>
    generateProvisionerOptions(config),
  );

  const dreamscapeId = state.currentDreamscape;

  const handleBuy = useCallback(
    (index: number) => {
      const option = options[index];
      if (!option || option.purchased) return;
      if (state.essence < option.cost) return;
      if (dreamscapeId === null) return;

      mutations.changeEssence(-option.cost, "provisioner");

      const newSite: SiteState = {
        id: crypto.randomUUID(),
        type: option.siteType,
        isEnhanced: false,
        isVisited: false,
      };
      mutations.addProvisionedSite(dreamscapeId, newSite);

      logEvent("provisioner_purchase", {
        siteType: option.siteType,
        cost: option.cost,
      });

      setOptions((prev) =>
        prev.map((o, i) => (i === index ? { ...o, purchased: true } : o)),
      );
    },
    [options, state.essence, dreamscapeId, mutations],
  );

  const handleContinue = useCallback(() => {
    const purchased = options.filter((o) => o.purchased);
    logEvent("site_completed", {
      siteType: "Provisioner",
      outcome: `Purchased ${String(purchased.length)} sites`,
      purchasedSites: purchased.map((o) => o.siteType),
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations, options]);

  return (
    <motion.div
      className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      {/* Header */}
      <div className="mb-6 text-center">
        <h2
          className="text-2xl font-bold tracking-wide md:text-3xl"
          style={{ color: "#a855f7" }}
        >
          Provisioner
        </h2>
        <p className="mt-2 text-sm opacity-50">
          Purchase additional sites for this dreamscape
        </p>
        <p className="mt-1 text-sm font-bold" style={{ color: "#fbbf24" }}>
          Essence: {String(state.essence)}
        </p>
      </div>

      {/* Options */}
      <div className="mb-8 flex flex-wrap justify-center gap-4">
        {options.map((option, index) => {
          const canAfford = state.essence >= option.cost;
          const disabled = option.purchased || !canAfford;
          return (
            <motion.div
              key={`${option.siteType}-${String(index)}`}
              className="flex w-56 flex-col items-center gap-3 rounded-xl border p-5"
              style={{
                borderColor: option.purchased
                  ? "rgba(16, 185, 129, 0.4)"
                  : disabled
                    ? "rgba(255,255,255,0.1)"
                    : "rgba(168, 85, 247, 0.3)",
                background: option.purchased
                  ? "rgba(16, 185, 129, 0.08)"
                  : "rgba(0,0,0,0.3)",
                opacity: disabled && !option.purchased ? 0.5 : 1,
              }}
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 + index * 0.1, duration: 0.3 }}
            >
              <h3
                className="text-lg font-bold"
                style={{
                  color: option.purchased ? "#10b981" : "#e2e8f0",
                }}
              >
                {siteTypeName(option.siteType)}
              </h3>
              <p
                className="text-sm font-bold"
                style={{ color: "#fbbf24" }}
              >
                {String(option.cost)} Essence
              </p>
              {option.purchased ? (
                <span
                  className="rounded-full px-3 py-1 text-sm font-bold"
                  style={{ color: "#10b981" }}
                >
                  Purchased
                </span>
              ) : (
                <button
                  className="rounded-lg px-4 py-2 text-sm font-bold text-white transition-opacity"
                  style={{
                    backgroundColor: canAfford ? "#7c3aed" : "#4b5563",
                    cursor: canAfford ? "pointer" : "not-allowed",
                  }}
                  disabled={!canAfford}
                  onClick={() => handleBuy(index)}
                >
                  Buy
                </button>
              )}
            </motion.div>
          );
        })}
      </div>

      {/* Continue button */}
      <motion.button
        className="rounded-lg px-8 py-3 text-lg font-bold text-white"
        style={{
          background: "linear-gradient(135deg, #7c3aedcc 0%, #7c3aed 100%)",
          boxShadow: "0 0 20px rgba(124, 58, 237, 0.25)",
        }}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.97 }}
        onClick={handleContinue}
      >
        Continue
      </motion.button>
    </motion.div>
  );
}
