import { useCallback, useEffect, useRef } from "react";
import { motion } from "framer-motion";
import type { SiteState, Dreamsign } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import { CardDisplay } from "../components/CardDisplay";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import type { Tide } from "../types/cards";
import {
  generateRewardSiteData,
  type RewardSiteData,
} from "../rewards/reward-generator";

/** Props for the RewardSiteScreen component. */
interface RewardSiteScreenProps {
  site: SiteState;
}

/** Displays the reward site: shows the pre-defined reward and accept/decline buttons. */
export function RewardSiteScreen({ site }: RewardSiteScreenProps) {
  const { state, mutations, cardDatabase, questContent } = useQuest();
  const rewardRef = useRef<{
    reward: RewardSiteData;
    remainingDreamsignPoolIds: string[];
    spentDreamsignPoolIds: string[];
  } | null>(null);

  if (rewardRef.current === null) {
    rewardRef.current = generateRewardSiteData({
      cardDatabase,
      dreamsignTemplates: questContent.dreamsignTemplates,
      remainingDreamsignPoolIds: state.remainingDreamsignPool,
      selectedPackageTides: state.resolvedPackage?.selectedTides ?? [],
    });
  }

  const rewardData = rewardRef.current.reward;

  useEffect(() => {
    if (rewardRef.current === null) {
      return;
    }

    if (rewardRef.current.spentDreamsignPoolIds.length > 0) {
      mutations.setRemainingDreamsignPool(
        rewardRef.current.remainingDreamsignPoolIds,
        "reward_site_revealed",
      );
    }

    logEvent("site_entered", {
      siteType: "Reward",
      isEnhanced: site.isEnhanced,
      rewardType: rewardData.rewardType,
    });
  }, [
    mutations,
    rewardData.rewardType,
    site.isEnhanced,
  ]);

  const completeSite = useCallback(() => {
    logEvent("site_completed", {
      siteType: "Reward",
      isEnhanced: site.isEnhanced,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations]);

  const handleAccept = useCallback(() => {
    if (rewardData.rewardType === "card") {
      mutations.addCard(rewardData.cardNumber, "reward_site");
    } else if (rewardData.rewardType === "dreamsign") {
      const dreamsign: Dreamsign = {
        name: rewardData.dreamsignName,
        tide: rewardData.dreamsignTide,
        effectDescription: rewardData.dreamsignEffect,
        isBane: false,
      };
      mutations.addDreamsign(dreamsign, "Reward");
    } else {
      mutations.changeEssence(rewardData.essenceAmount, "reward_site");
    }

    completeSite();
  }, [rewardData, mutations, completeSite]);

  const handleDecline = useCallback(() => {
    logEvent("reward_declined", {
      rewardType: rewardData.rewardType,
    });
    completeSite();
  }, [rewardData.rewardType, completeSite]);

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
          Reward
        </h2>
        <p className="mt-1 text-sm opacity-50">
          A gift from the dreamscape awaits
        </p>
      </div>

      {/* Reward display */}
      <motion.div
        className="mb-8 flex items-center justify-center"
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
      >
        {rewardData.rewardType === "card" && (
          <CardRewardDisplay cardNumber={rewardData.cardNumber} />
        )}
        {rewardData.rewardType === "dreamsign" && (
          <DreamsignRewardDisplay
            name={rewardData.dreamsignName}
            tide={rewardData.dreamsignTide}
            effectDescription={rewardData.dreamsignEffect}
          />
        )}
        {rewardData.rewardType === "essence" && (
          <EssenceRewardDisplay
            amount={rewardData.essenceAmount}
          />
        )}
      </motion.div>

      {/* Action buttons */}
      <div className="flex gap-4">
        <motion.button
          className="rounded-lg px-8 py-3 text-lg font-bold text-white transition-opacity"
          style={{
            background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
            boxShadow: "0 0 20px rgba(124, 58, 237, 0.3)",
          }}
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.97 }}
          onClick={handleAccept}
        >
          Accept
        </motion.button>
        <button
          className="rounded-lg px-8 py-3 text-lg font-medium transition-colors"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={handleDecline}
        >
          Decline
        </button>
      </div>
    </motion.div>
  );
}

/** Renders a card reward using the full CardDisplay component. */
function CardRewardDisplay({ cardNumber }: { cardNumber: number }) {
  const { cardDatabase } = useQuest();
  const card = cardDatabase.get(cardNumber);

  if (!card) {
    return (
      <p className="text-lg opacity-50">
        Unknown card #{String(cardNumber)}
      </p>
    );
  }

  return (
    <div
      className="rounded-xl p-6"
      style={{
        background:
          "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: "1px solid rgba(168, 85, 247, 0.3)",
        boxShadow: "0 0 30px rgba(168, 85, 247, 0.15)",
      }}
    >
      <p className="mb-3 text-center text-xs font-bold uppercase tracking-wider opacity-50">
        Card Reward
      </p>
      <div style={{ width: "200px" }}>
        <CardDisplay card={card} />
      </div>
    </div>
  );
}

/** Renders a dreamsign reward with tide icon, name, and effect. */
function DreamsignRewardDisplay({
  name,
  tide,
  effectDescription,
}: {
  name: string;
  tide: Tide;
  effectDescription: string;
}) {
  const tideColor = TIDE_COLORS[tide];

  return (
    <div
      className="flex w-64 flex-col items-center gap-3 rounded-xl p-6"
      style={{
        background:
          "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `1px solid ${tideColor}60`,
        boxShadow: `0 0 30px ${tideColor}15`,
      }}
    >
      <p className="text-xs font-bold uppercase tracking-wider opacity-50">
        Dreamsign Reward
      </p>
      <img
        src={tideIconUrl(tide)}
        alt={tide}
        className="h-14 w-14 rounded-full object-contain"
        style={{ border: `2px solid ${tideColor}` }}
      />
      <span
        className="rounded-full px-2 py-0.5 text-[10px] font-bold uppercase tracking-wider"
        style={{
          background: `${tideColor}20`,
          color: tideColor,
          border: `1px solid ${tideColor}40`,
        }}
      >
        {tide}
      </span>
      <h3
        className="text-center text-lg font-bold"
        style={{ color: tideColor }}
      >
        {name}
      </h3>
      <p
        className="text-center text-sm leading-relaxed opacity-70"
        style={{ color: "#e2e8f0" }}
      >
        {effectDescription}
      </p>
    </div>
  );
}

/** Renders an essence reward with a glowing amount display. */
function EssenceRewardDisplay({ amount }: { amount: number }) {
  return (
    <div
      className="flex flex-col items-center gap-4 rounded-xl p-8"
      style={{
        background:
          "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: "1px solid rgba(251, 191, 36, 0.3)",
        boxShadow: "0 0 30px rgba(251, 191, 36, 0.15)",
      }}
    >
      <p className="text-xs font-bold uppercase tracking-wider opacity-50">
        Essence Reward
      </p>
      <motion.div
        className="flex h-24 w-24 items-center justify-center rounded-full"
        style={{
          background:
            "radial-gradient(circle, rgba(251,191,36,0.2) 0%, rgba(251,191,36,0.05) 60%, transparent 100%)",
        }}
        animate={{
          boxShadow: [
            "0 0 20px rgba(251, 191, 36, 0.3)",
            "0 0 40px rgba(251, 191, 36, 0.5)",
            "0 0 20px rgba(251, 191, 36, 0.3)",
          ],
        }}
        transition={{ duration: 1.5, repeat: Infinity }}
      >
        <span
          className="text-4xl font-black"
          style={{ color: "#fbbf24" }}
        >
          +{String(amount)}
        </span>
      </motion.div>
      <p
        className="text-lg font-medium"
        style={{ color: "#fbbf24" }}
      >
        Essence
      </p>
    </div>
  );
}
