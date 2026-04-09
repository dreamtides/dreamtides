import { useCallback, useMemo, useState } from "react";
import { motion } from "framer-motion";
import type { CardData } from "../types/cards";
import type { SiteState } from "../types/quest";
import { CardDisplay } from "../components/CardDisplay";
import { CardOverlay } from "../components/CardOverlay";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import {
  generateShopInventory,
  effectivePrice,
  rerollCost,
  type ShopSlot,
} from "../shop/shop-generator";

/** Props for the ShopScreen component. */
interface ShopScreenProps {
  site: SiteState;
}

/** Renders the Shop site screen with a 2x3 item grid, purchasing, and rerolling. */
export function ShopScreen({ site }: ShopScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const { essence, deck } = state;

  const [slots, setSlots] = useState<ShopSlot[]>(() =>
    generateShopInventory(cardDatabase, deck, state.excludedTides),
  );
  const [rerollCount, setRerollCount] = useState(0);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);

  const currentRerollCost = useMemo(
    () => rerollCost(rerollCount, site.isEnhanced),
    [rerollCount, site.isEnhanced],
  );

  const handleBuy = useCallback(
    (index: number) => {
      const slot = slots[index];
      if (slot.purchased) return;

      const price = effectivePrice(slot);
      if (price > essence) return;

      mutations.changeEssence(-price, "shop_purchase");

      if (slot.itemType === "card" && slot.card) {
        mutations.addCard(slot.card.cardNumber, "shop");
        logEvent("shop_purchase", {
          itemType: "card",
          cardNumber: slot.card.cardNumber,
          cardName: slot.card.name,
          basePrice: slot.basePrice,
          discountedPrice: price,
          essenceRemaining: essence - price,
        });
      } else if (slot.itemType === "dreamsign" && slot.dreamsign) {
        mutations.addDreamsign(slot.dreamsign, "Shop");
        logEvent("shop_purchase", {
          itemType: "dreamsign",
          dreamsignName: slot.dreamsign.name,
          basePrice: slot.basePrice,
          discountedPrice: price,
          essenceRemaining: essence - price,
        });
      }

      setSlots((prev) =>
        prev.map((s, i) => (i === index ? { ...s, purchased: true } : s)),
      );
    },
    [slots, essence, mutations],
  );

  const handleReroll = useCallback(
    (index: number) => {
      if (currentRerollCost > essence) return;

      mutations.changeEssence(-currentRerollCost, "shop_reroll");
      logEvent("shop_reroll", {
        rerollCost: currentRerollCost,
        rerollCount: rerollCount + 1,
      });

      setRerollCount((prev) => prev + 1);

      // Regenerate unpurchased non-reroll slots
      const newInventory = generateShopInventory(cardDatabase, deck, state.excludedTides);
      // Collect only non-reroll replacement items to avoid introducing
      // a second reroll slot from the freshly generated inventory.
      const replacements = newInventory.filter(
        (s) => s.itemType !== "reroll",
      );
      let replacementIdx = 0;
      setSlots((prev) =>
        prev.map((s, i) => {
          if (s.purchased) return s;
          if (i === index) {
            // Keep the reroll slot but update its price
            return {
              ...s,
              basePrice:
                rerollCost(rerollCount + 1, site.isEnhanced),
            };
          }
          if (s.itemType === "reroll") return s;
          const replacement = replacements[replacementIdx];
          replacementIdx += 1;
          return replacement ?? s;
        }),
      );
    },
    [
      currentRerollCost,
      essence,
      rerollCount,
      cardDatabase,
      deck,
      site.isEnhanced,
      mutations,
    ],
  );

  const handleLeave = useCallback(() => {
    logEvent("site_completed", {
      siteType: "Shop",
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
          Shop
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

      {/* Item grid: 3 columns desktop, 2 tablet */}
      <div className="grid w-full max-w-4xl grid-cols-2 gap-4 lg:grid-cols-3 lg:gap-6">
        {slots.map((slot, index) => (
          <ShopSlotCard
            key={`shop-slot-${String(index)}`}
            slot={slot}
            index={index}
            canAfford={effectivePrice(slot) <= essence}
            rerollCost={
              slot.itemType === "reroll" ? currentRerollCost : undefined
            }
            onBuy={handleBuy}
            onReroll={handleReroll}
            onCardClick={setOverlayCard}
          />
        ))}
      </div>

      {/* Leave button */}
      <button
        className="mt-8 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
        style={{
          background: "rgba(107, 114, 128, 0.2)",
          border: "1px solid rgba(107, 114, 128, 0.4)",
          color: "#9ca3af",
        }}
        onClick={handleLeave}
      >
        Leave Shop
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
  rerollCost?: number;
  onBuy: (index: number) => void;
  onReroll: (index: number) => void;
  onCardClick: (card: CardData) => void;
}

/** Renders a single slot in the shop grid. */
function ShopSlotCard({
  slot,
  index,
  canAfford,
  rerollCost: rerollCostValue,
  onBuy,
  onReroll,
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

  if (slot.itemType === "reroll") {
    const displayCost = rerollCostValue ?? slot.basePrice;
    const canAffordReroll = displayCost <= 0 || canAfford;
    return (
      <div className="flex flex-col gap-2">
        <div
          className="flex flex-col items-center justify-center gap-3 rounded-lg p-4"
          style={{
            aspectRatio: "2 / 3",
            background:
              "linear-gradient(145deg, #1a1025 0%, #1a1030 60%, #0d0814 100%)",
            border: "1px solid rgba(168, 85, 247, 0.4)",
            boxShadow: "0 0 12px rgba(168, 85, 247, 0.15)",
          }}
        >
          <div className="text-4xl">{"\u267B\uFE0F"}</div>
          <h3
            className="text-center text-lg font-bold"
            style={{ color: "#a855f7" }}
          >
            Reroll Shop
          </h3>
          <p className="text-center text-xs opacity-50">
            Refresh all unsold items with new stock
          </p>
        </div>
        <button
          className="w-full rounded-lg px-3 py-2 text-sm font-bold transition-opacity"
          style={{
            background: canAffordReroll ? "#7c3aed" : "#4b5563",
            color: canAffordReroll ? "#fbbf24" : "#9ca3af",
            opacity: canAffordReroll ? 1 : 0.6,
            cursor: canAffordReroll ? "pointer" : "not-allowed",
          }}
          disabled={!canAffordReroll}
          onClick={() => onReroll(index)}
        >
          {displayCost === 0 ? "Reroll (FREE)" : `Reroll -- ${String(displayCost)} Essence`}
        </button>
      </div>
    );
  }

  if (slot.itemType === "dreamsign" && slot.dreamsign) {
    const ds = slot.dreamsign;
    const tideColor = TIDE_COLORS[ds.tide];
    return (
      <div className="flex flex-col gap-2">
        <div
          className="flex flex-col items-center justify-center gap-2 rounded-lg p-3"
          style={{
            aspectRatio: "2 / 3",
            background:
              "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
            border: `1px solid ${tideColor}60`,
            boxShadow: `0 0 8px ${tideColor}20`,
          }}
        >
          <img
            src={tideIconUrl(ds.tide)}
            alt={ds.tide}
            className="h-10 w-10 rounded-full object-contain"
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
            Dreamsign
          </span>
          <h3
            className="text-center text-sm font-bold"
            style={{ color: tideColor }}
          >
            {ds.name}
          </h3>
          <p
            className="text-center text-[10px] leading-tight opacity-70"
            style={{ color: "#e2e8f0" }}
          >
            {ds.effectDescription}
          </p>
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

  // Card slot
  if (slot.card) {
    return (
      <div className="flex flex-col gap-2">
        <CardDisplay
          card={slot.card}
          onClick={() => onCardClick(slot.card!)}
        />
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

  return null;
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
        background: canAfford ? "#7c3aed" : "#4b5563",
        color: canAfford ? "#fbbf24" : "#9ca3af",
        opacity: canAfford ? 1 : 0.6,
        cursor: canAfford ? "pointer" : "not-allowed",
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
