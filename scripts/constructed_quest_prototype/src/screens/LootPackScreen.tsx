import { useCallback, useEffect, useMemo, useRef } from "react";
import { motion } from "framer-motion";
import type { SiteState } from "../types/quest";
import type { Tide } from "../types/cards";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { CardDisplay } from "../components/CardDisplay";
import { generateLootPack } from "../pack/pack-generator";
import { logEvent } from "../logging";

/** Props for the LootPackScreen component. */
interface LootPackScreenProps {
  site: SiteState;
}

/** Displays a loot pack opening screen: shows themed cards and adds them to the player's pool. */
export function LootPackScreen({ site }: LootPackScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();

  const packTide = (site.data?.["packTide"] as Tide) ?? "Neutral";
  const tideColor = TIDE_COLORS[packTide];

  const packCards = useMemo(
    () =>
      generateLootPack(
        cardDatabase,
        state.pool,
        packTide,
        config,
        site.isEnhanced,
      ),
    // Generate once on mount — intentionally using site.id as the stable key
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [site.id],
  );

  const hasAddedRef = useRef(false);

  useEffect(() => {
    if (hasAddedRef.current) return;
    hasAddedRef.current = true;

    logEvent("site_entered", {
      siteType: "LootPack",
      isEnhanced: site.isEnhanced,
      packTide,
      cardCount: packCards.length,
      cardNames: packCards.map((c) => c.name),
    });

    // Add all pack cards to the player's pool
    for (const card of packCards) {
      mutations.addToPool(card.cardNumber, "loot_pack");
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [site.id]);

  const handleContinue = useCallback(() => {
    logEvent("site_completed", {
      siteType: "LootPack",
      isEnhanced: site.isEnhanced,
      packTide,
      cardCount: packCards.length,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations, packTide, packCards]);

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
          style={{ color: tideColor }}
        >
          Loot Pack
        </h2>
        <div className="mt-2 flex items-center justify-center gap-2">
          <img
            src={tideIconUrl(packTide)}
            alt={packTide}
            className="h-6 w-6 rounded-full object-contain"
            style={{ border: `1px solid ${tideColor}` }}
          />
          <span
            className="rounded-full px-2.5 py-0.5 text-xs font-bold uppercase tracking-wider"
            style={{
              background: `${tideColor}20`,
              color: tideColor,
              border: `1px solid ${tideColor}40`,
            }}
          >
            {packTide}
          </span>
          {site.isEnhanced && (
            <span
              className="rounded-full px-2.5 py-0.5 text-xs font-bold"
              style={{
                background: "rgba(168, 85, 247, 0.15)",
                color: "#c084fc",
                border: "1px solid rgba(168, 85, 247, 0.3)",
              }}
            >
              Enhanced
            </span>
          )}
        </div>
        <p className="mt-2 text-sm opacity-50">
          {String(packCards.length)} cards added to your pool
        </p>
      </div>

      {/* Card grid */}
      <motion.div
        className="mb-8 grid gap-4"
        style={{
          gridTemplateColumns: `repeat(${Math.min(packCards.length, 4)}, minmax(150px, 200px))`,
        }}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
      >
        {packCards.map((card, index) => (
          <motion.div
            key={`${String(card.cardNumber)}-${String(index)}`}
            initial={{ opacity: 0, y: 30 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 + index * 0.1, duration: 0.4 }}
          >
            <CardDisplay card={card} />
          </motion.div>
        ))}
      </motion.div>

      {/* Continue button */}
      <motion.button
        className="rounded-lg px-8 py-3 text-lg font-bold text-white transition-opacity"
        style={{
          background: `linear-gradient(135deg, ${tideColor}cc 0%, ${tideColor} 100%)`,
          boxShadow: `0 0 20px ${tideColor}40`,
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
