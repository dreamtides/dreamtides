import { useCallback, useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { SiteState } from "../types/quest";
import { CardDisplay } from "../components/CardDisplay";
import { CardOverlay } from "../components/CardOverlay";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { logEvent } from "../logging";
import {
  generateCardShopInventory,
  generateTideCrystalSlots,
  effectivePrice,
  rerollCost,
  type ShopSlot,
  type TideCrystalSlot,
} from "../shop/shop-generator";
import { startingTideSeedTides } from "../data/tide-weights";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";

/** Props for the ShopScreen component. */
interface ShopScreenProps {
  site: SiteState;
}

/** Computes the set of tides the player can play based on their tide crystals. */
function playableTidesFromCrystals(tideCrystals: Record<Tide, number>): Set<Tide> {
  const result = new Set<Tide>();
  for (const [tide, count] of Object.entries(tideCrystals)) {
    if (count > 0) {
      result.add(tide as Tide);
    }
  }
  return result;
}

/** Renders the Card Shop site screen with a card grid, purchasing, and rerolling. */
export function ShopScreen({ site }: ShopScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const config = useQuestConfig();
  const { essence } = state;

  const playableTides = useMemo(
    () => playableTidesFromCrystals(state.tideCrystals),
    [state.tideCrystals],
  );

  const [slots, setSlots] = useState<ShopSlot[]>(() =>
    generateCardShopInventory(cardDatabase, state.pool, startingTideSeedTides(state.startingTide), config, playableTides),
  );
  const [crystalSlots, setCrystalSlots] = useState<TideCrystalSlot[]>(() =>
    generateTideCrystalSlots(state.tideCrystals, state.startingTide),
  );
  const [rerollCount, setRerollCount] = useState(0);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);

  const currentRerollCost = useMemo(
    () => rerollCost(rerollCount, site.isEnhanced, config),
    [rerollCount, site.isEnhanced, config],
  );

  const handleBuy = useCallback(
    (index: number) => {
      const slot = slots[index];
      if (slot.purchased) return;

      const price = effectivePrice(slot);
      if (price > essence) return;

      mutations.changeEssence(-price, "shop_purchase");
      mutations.addToPool(slot.card.cardNumber, "card_shop");

      logEvent("shop_purchase", {
        cardNumber: slot.card.cardNumber,
        cardName: slot.card.name,
        basePrice: slot.basePrice,
        discountedPrice: price,
        essenceRemaining: essence - price,
      });

      setSlots((prev) =>
        prev.map((s, i) => (i === index ? { ...s, purchased: true } : s)),
      );
    },
    [slots, essence, mutations],
  );

  const handleReroll = useCallback(() => {
    if (currentRerollCost > essence) return;

    if (currentRerollCost > 0) {
      mutations.changeEssence(-currentRerollCost, "shop_reroll");
    }
    logEvent("shop_reroll", {
      rerollCost: currentRerollCost,
      rerollCount: rerollCount + 1,
    });

    setRerollCount((prev) => prev + 1);
    setSlots(
      generateCardShopInventory(cardDatabase, state.pool, startingTideSeedTides(state.startingTide), config, playableTides),
    );
  }, [currentRerollCost, essence, rerollCount, cardDatabase, state.pool, state.startingTide, config, mutations, playableTides]);

  const handleBuyCrystal = useCallback(
    (index: number) => {
      const slot = crystalSlots[index];
      if (slot.purchased || slot.price > essence) return;

      mutations.changeEssence(-slot.price, "crystal_purchase");
      mutations.addTideCrystal(slot.tide, 1);

      logEvent("crystal_purchase", {
        tide: slot.tide,
        price: slot.price,
        essenceRemaining: essence - slot.price,
      });

      setCrystalSlots((prev) =>
        prev.map((s, i) => (i === index ? { ...s, purchased: true } : s)),
      );
    },
    [crystalSlots, essence, mutations],
  );

  const handleLeave = useCallback(() => {
    logEvent("site_completed", {
      siteType: "CardShop",
      outcome: "left",
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site.id, mutations]);

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
          Card Shop
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
            Enhanced -- Free Rerolls
          </span>
        )}
      </div>

      {/* Tide Crystal slots */}
      {crystalSlots.length > 0 && (
        <div className="mb-6 w-full max-w-4xl">
          <h3
            className="mb-3 text-center text-sm font-bold uppercase tracking-wider"
            style={{ color: "#c084fc" }}
          >
            Tide Crystals
          </h3>
          <div className="flex flex-wrap justify-center gap-3">
            {crystalSlots.map((slot, index) => (
              <button
                key={slot.tide}
                className="flex cursor-pointer items-center gap-2 rounded-lg px-4 py-2.5 transition-opacity"
                style={{
                  background: slot.purchased
                    ? "rgba(107, 114, 128, 0.1)"
                    : `rgba(124, 58, 237, 0.1)`,
                  border: slot.purchased
                    ? "1px dashed rgba(107, 114, 128, 0.2)"
                    : `1px solid ${TIDE_COLORS[slot.tide]}40`,
                  opacity: slot.purchased ? 0.3 : slot.price <= essence ? 1 : 0.5,
                  cursor: slot.purchased || slot.price > essence ? "not-allowed" : "pointer",
                }}
                disabled={slot.purchased || slot.price > essence}
                onClick={() => handleBuyCrystal(index)}
              >
                <img
                  src={tideIconUrl(slot.tide)}
                  alt={slot.tide}
                  className="h-6 w-6 rounded-full"
                  style={{ border: `2px solid ${TIDE_COLORS[slot.tide]}` }}
                />
                <span
                  className="text-sm font-bold"
                  style={{ color: TIDE_COLORS[slot.tide] }}
                >
                  {slot.tide}
                </span>
                {!slot.purchased && (
                  <span
                    className="ml-1 text-xs font-medium"
                    style={{ color: "#fbbf24" }}
                  >
                    {String(slot.price)} Essence
                  </span>
                )}
                {slot.purchased && (
                  <span className="ml-1 text-xs opacity-50">Purchased</span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Card grid: 2 columns on small screens, 4 on large */}
      <div className="grid w-full max-w-4xl grid-cols-2 gap-4 lg:grid-cols-4 lg:gap-6">
        {slots.map((slot, index) => (
          <ShopSlotCard
            key={`shop-slot-${String(index)}`}
            slot={slot}
            index={index}
            canAfford={effectivePrice(slot) <= essence}
            onBuy={handleBuy}
            onCardClick={setOverlayCard}
          />
        ))}
      </div>

      {/* Reroll button */}
      <div className="mt-6">
        <button
          className="rounded-lg px-6 py-2.5 text-base font-bold transition-opacity"
          style={{
            background: currentRerollCost <= essence
              ? "linear-gradient(135deg, #d4a017 0%, #b8860b 100%)"
              : "#4b5563",
            color: currentRerollCost <= essence ? "#ffffff" : "#9ca3af",
            opacity: currentRerollCost <= essence ? 1 : 0.6,
            cursor: currentRerollCost <= essence ? "pointer" : "not-allowed",
            border: currentRerollCost <= essence ? "1px solid rgba(251, 191, 36, 0.5)" : undefined,
          }}
          disabled={currentRerollCost > essence}
          onClick={handleReroll}
        >
          {currentRerollCost === 0
            ? "Reroll (FREE)"
            : `Reroll -- ${String(currentRerollCost)} Essence`}
        </button>
      </div>

      {/* Leave button */}
      <button
        className="mt-4 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
        style={{
          background: "rgba(107, 114, 128, 0.2)",
          border: "1px solid rgba(107, 114, 128, 0.4)",
          color: "#9ca3af",
        }}
        onClick={handleLeave}
      >
        Leave Card Shop
      </button>

      <CardOverlay card={overlayCard} onClose={() => setOverlayCard(null)} />
    </motion.div>
  );
}

/** Props for a single shop slot card. */
interface ShopSlotCardProps {
  slot: ShopSlot;
  index: number;
  canAfford: boolean;
  onBuy: (index: number) => void;
  onCardClick: (card: CardData) => void;
}

/** Renders a single slot in the shop grid. */
function ShopSlotCard({
  slot,
  index,
  canAfford,
  onBuy,
  onCardClick,
}: ShopSlotCardProps) {
  if (slot.purchased) {
    return (
      <div
        className="rounded-lg opacity-20"
        style={{
          aspectRatio: "2 / 3",
          background:
            "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
          border: "1px dashed rgba(107, 114, 128, 0.15)",
        }}
      />
    );
  }

  const price = effectivePrice(slot);
  const hasDiscount = slot.discountPercent > 0;

  return (
    <div className="flex flex-col gap-2">
      <div className="relative">
        <CardDisplay
          card={slot.card}
          onClick={() => onCardClick(slot.card)}
        />
        {hasDiscount && (
          <span
            className="absolute right-1 top-1 rounded-full px-2 py-0.5 text-xs font-bold"
            style={{
              background: "rgba(239, 68, 68, 0.9)",
              color: "#fff",
            }}
          >
            -{String(slot.discountPercent)}%
          </span>
        )}
      </div>
      <PriceButton
        basePrice={slot.basePrice}
        price={price}
        hasDiscount={hasDiscount}
        canAfford={canAfford}
        onClick={() => onBuy(index)}
      />
    </div>
  );
}

/** Renders the Buy button with price display, including discount styling. */
function PriceButton({
  basePrice,
  price,
  hasDiscount,
  canAfford,
  onClick,
}: {
  basePrice: number;
  price: number;
  hasDiscount: boolean;
  canAfford: boolean;
  onClick: () => void;
}) {
  return (
    <button
      className="flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm font-bold transition-opacity"
      style={{
        background: canAfford
          ? "linear-gradient(135deg, #d4a017 0%, #b8860b 100%)"
          : "#4b5563",
        color: canAfford ? "#ffffff" : "#9ca3af",
        opacity: canAfford ? 1 : 0.6,
        cursor: canAfford ? "pointer" : "not-allowed",
        border: canAfford ? "1px solid rgba(251, 191, 36, 0.5)" : undefined,
      }}
      disabled={!canAfford}
      onClick={onClick}
    >
      <span>Buy</span>
      {hasDiscount && (
        <span
          className="text-xs line-through opacity-50"
          style={{ color: "#9ca3af" }}
        >
          {String(basePrice)}
        </span>
      )}
      <span>{String(price)}</span>
      <span className="text-xs opacity-70">Essence</span>
    </button>
  );
}
