import { useCallback, useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { SiteState, PackShopSlot } from "../types/quest";
import type { NamedTide } from "../types/cards";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { adjacentTides, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { CardDisplay } from "../components/CardDisplay";
import { generatePackShopInventory } from "../shop/pack-shop-generator";
import { logEvent } from "../logging";
import { startingTideSeedTides } from "../data/tide-weights";

/** Props for the PackShopScreen component. */
interface PackShopScreenProps {
  site: SiteState;
}

/** Display names for each pack type. */
const PACK_TYPE_LABELS: Record<string, string> = {
  tide: "Tide Pack",
  alliance: "Alliance Pack",
  removal: "Removal Pack",
  aggro: "Aggro Pack",
  events: "Events Pack",
};

/** Theme colors for each pack type. */
const PACK_TYPE_COLORS: Record<string, string> = {
  tide: "#a855f7",
  alliance: "#f59e0b",
  removal: "#ef4444",
  aggro: "#10b981",
  events: "#3b82f6",
};

/** Renders the Pack Shop site screen where players buy themed packs. */
export function PackShopScreen({ site }: PackShopScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();
  const { essence } = state;

  const seedTides = useMemo(() => {
    const dreamscapeTide = site.data?.dreamscapeTide as NamedTide | undefined;
    if (dreamscapeTide) {
      return [dreamscapeTide, ...adjacentTides(dreamscapeTide)];
    }
    return startingTideSeedTides(state.startingTide);
  }, [site.data, state.startingTide]);

  const [packs, setPacks] = useState<PackShopSlot[]>(() =>
    generatePackShopInventory(
      cardDatabase,
      state.pool,
      seedTides,
      config,
    ),
  );
  const [expandedIndex, setExpandedIndex] = useState<number | null>(null);

  const handleBuy = useCallback(
    (index: number) => {
      const pack = packs[index];
      if (pack.purchased) return;

      const price = site.isEnhanced ? 0 : pack.price;
      if (price > essence) return;

      if (price > 0) {
        mutations.changeEssence(-price, "pack_shop");
      }

      for (const card of pack.cards) {
        mutations.addToPool(card.cardNumber, "pack_shop");
      }

      logEvent("pack_shop_purchase", {
        packType: pack.packType,
        tide: pack.tide,
        price,
        cardCount: pack.cards.length,
        cardNames: pack.cards.map((c) => c.name),
      });

      setPacks((prev) =>
        prev.map((p, i) => (i === index ? { ...p, purchased: true } : p)),
      );
      setExpandedIndex(index);
    },
    [packs, essence, site.isEnhanced, mutations],
  );

  const handleDone = useCallback(() => {
    logEvent("site_completed", {
      siteType: "PackShop",
      isEnhanced: site.isEnhanced,
      packsBought: packs.filter((p) => p.purchased).length,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations, packs]);

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
          Pack Shop
        </h2>
        {site.isEnhanced && (
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(168, 85, 247, 0.15)",
              color: "#c084fc",
              border: "1px solid rgba(168, 85, 247, 0.3)",
            }}
          >
            Enhanced -- All Packs Free
          </span>
        )}
        <p className="mt-2 text-sm opacity-50">
          {String(essence)} Essence available
        </p>
      </div>

      {/* Pack tiles */}
      <div className="mb-6 grid w-full max-w-4xl grid-cols-1 gap-5 md:grid-cols-3">
        {packs.map((pack, index) => (
          <PackTile
            key={`pack-${String(index)}`}
            pack={pack}
            index={index}
            isEnhanced={site.isEnhanced}
            canAfford={site.isEnhanced || pack.price <= essence}
            isExpanded={expandedIndex === index}
            onBuy={handleBuy}
            onToggleExpand={() =>
              setExpandedIndex(expandedIndex === index ? null : index)
            }
          />
        ))}
      </div>

      {/* Done button */}
      <motion.button
        className="rounded-lg px-8 py-3 text-lg font-bold text-white transition-opacity"
        style={{
          background: "linear-gradient(135deg, #7c3aedcc 0%, #7c3aed 100%)",
          boxShadow: "0 0 20px rgba(124, 58, 237, 0.25)",
        }}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.97 }}
        onClick={handleDone}
      >
        Done
      </motion.button>
    </motion.div>
  );
}

/** Props for a single pack tile. */
interface PackTileProps {
  pack: PackShopSlot;
  index: number;
  isEnhanced: boolean;
  canAfford: boolean;
  isExpanded: boolean;
  onBuy: (index: number) => void;
  onToggleExpand: () => void;
}

/** Renders a single pack offer tile. */
function PackTile({
  pack,
  index,
  isEnhanced,
  canAfford,
  isExpanded,
  onBuy,
  onToggleExpand,
}: PackTileProps) {
  const color = pack.tide
    ? TIDE_COLORS[pack.tide]
    : (PACK_TYPE_COLORS[pack.packType] ?? "#a855f7");
  const label = PACK_TYPE_LABELS[pack.packType] ?? "Pack";
  const displayPrice = isEnhanced ? 0 : pack.price;

  if (pack.purchased) {
    return (
      <motion.div
        className="flex flex-col rounded-xl p-4"
        style={{
          background: `linear-gradient(145deg, ${color}10 0%, ${color}05 100%)`,
          border: `1px solid ${color}30`,
          opacity: 0.8,
        }}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 0.8, scale: 1 }}
      >
        <div className="mb-3 flex items-center justify-between">
          <span className="text-sm font-bold" style={{ color }}>
            {label}
          </span>
          <span
            className="rounded-full px-2 py-0.5 text-xs font-bold"
            style={{
              background: `${color}20`,
              color,
            }}
          >
            Purchased
          </span>
        </div>

        {/* Show purchased cards */}
        <div className="grid grid-cols-2 gap-2">
          {pack.cards.map((card, ci) => (
            <motion.div
              key={`${String(card.cardNumber)}-${String(ci)}`}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: ci * 0.1 }}
            >
              <CardDisplay card={card} />
            </motion.div>
          ))}
        </div>
      </motion.div>
    );
  }

  return (
    <motion.div
      className="flex flex-col rounded-xl p-4"
      style={{
        background: `linear-gradient(145deg, ${color}15 0%, ${color}08 100%)`,
        border: `1px solid ${color}40`,
      }}
      whileHover={{ scale: 1.02 }}
      transition={{ duration: 0.2 }}
    >
      {/* Pack header */}
      <div className="mb-3 flex items-center gap-2">
        {pack.tide && (
          <img
            src={tideIconUrl(pack.tide)}
            alt={pack.tide}
            className="h-6 w-6 rounded-full object-contain"
            style={{ border: `1px solid ${color}` }}
          />
        )}
        <span className="text-base font-bold" style={{ color }}>
          {label}
        </span>
      </div>

      {/* Tide / alliance info */}
      {pack.tide && (
        <span
          className="mb-2 inline-block w-fit rounded-full px-2.5 py-0.5 text-xs font-bold uppercase tracking-wider"
          style={{
            background: `${color}20`,
            color,
            border: `1px solid ${color}40`,
          }}
        >
          {pack.alliance ?? pack.tide}
        </span>
      )}

      {/* Card count */}
      <p className="mb-3 text-xs opacity-50">
        {String(pack.cards.length)} cards
      </p>

      {/* Preview toggle for non-purchased packs */}
      {isExpanded && (
        <motion.div
          className="mb-3 grid grid-cols-2 gap-2"
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          transition={{ duration: 0.3 }}
        >
          {pack.cards.map((card, ci) => (
            <CardDisplay
              key={`preview-${String(card.cardNumber)}-${String(ci)}`}
              card={card}
            />
          ))}
        </motion.div>
      )}

      <button
        className="mb-2 text-xs underline opacity-40 transition-opacity hover:opacity-70"
        onClick={onToggleExpand}
      >
        {isExpanded ? "Hide cards" : "Preview cards"}
      </button>

      {/* Buy button */}
      <button
        className="mt-auto flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2.5 text-sm font-bold transition-opacity"
        style={{
          background: canAfford ? "#7c3aed" : "#4b5563",
          color: canAfford ? "#fbbf24" : "#9ca3af",
          opacity: canAfford ? 1 : 0.6,
          cursor: canAfford ? "pointer" : "not-allowed",
        }}
        disabled={!canAfford}
        onClick={() => onBuy(index)}
      >
        <span>Buy</span>
        <span>
          {displayPrice === 0 ? "FREE" : `${String(displayPrice)} Essence`}
        </span>
      </button>
    </motion.div>
  );
}
