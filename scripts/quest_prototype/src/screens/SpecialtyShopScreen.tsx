import { useCallback, useState } from "react";
import { motion } from "framer-motion";
import type { CardData } from "../types/cards";
import type { SiteState } from "../types/quest";
import { CardDisplay } from "../components/CardDisplay";
import { CardOverlay } from "../components/CardOverlay";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import {
  generateSpecialtyShopInventory,
  effectivePrice,
  type ShopSlot,
} from "../shop/shop-generator";
import { computeQuestTideProfile } from "../data/quest-tide-profile";

/** Props for the SpecialtyShopScreen component. */
interface SpecialtyShopScreenProps {
  site: SiteState;
}

/** Renders the Specialty Shop site screen with 4 rare cards. */
export function SpecialtyShopScreen({ site }: SpecialtyShopScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const { essence, deck } = state;

  const [slots, setSlots] = useState<ShopSlot[]>(() => {
    const profile = computeQuestTideProfile({
      startingTide: state.startingTide,
      deck: state.deck,
      cardDatabase,
      dreamcaller: state.dreamcaller,
      tideCrystals: state.tideCrystals,
      recentDraftPicks: state.draftState?.draftedCards ?? [],
    });
    const inventory = generateSpecialtyShopInventory(cardDatabase, deck, state.excludedTides, profile);
    if (site.isEnhanced) {
      return inventory.map((s) => ({
        ...s,
        basePrice: 0,
        discountPercent: 0,
      }));
    }
    return inventory;
  });
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);

  const handleBuy = useCallback(
    (index: number) => {
      const slot = slots[index];
      if (slot.purchased || !slot.card) return;

      const price = effectivePrice(slot);
      if (price > essence && !site.isEnhanced) return;

      if (price > 0) {
        mutations.changeEssence(-price, "specialty_shop_purchase");
      }

      mutations.addCard(slot.card.cardNumber, "specialty_shop");

      logEvent("shop_purchase", {
        itemType: "card",
        cardNumber: slot.card.cardNumber,
        cardName: slot.card.name,
        basePrice: slot.basePrice,
        discountedPrice: price,
        essenceRemaining: essence - price,
        isSpecialtyShop: true,
        isEnhanced: site.isEnhanced,
      });

      setSlots((prev) =>
        prev.map((s, i) => (i === index ? { ...s, purchased: true } : s)),
      );
    },
    [slots, essence, site.isEnhanced, mutations],
  );

  const handleLeave = useCallback(() => {
    logEvent("site_completed", {
      siteType: "SpecialtyShop",
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
          Specialty Shop
        </h2>
        <p className="mt-1 text-sm opacity-50">
          Rare cards for the discerning collector
        </p>
        {site.isEnhanced && (
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(251, 191, 36, 0.15)",
              color: "#fbbf24",
              border: "1px solid rgba(251, 191, 36, 0.3)",
            }}
          >
            Enhanced -- Take All For Free!
          </span>
        )}
      </div>

      {/* Card grid: 4 columns desktop, 2 tablet */}
      <div className="grid w-full max-w-5xl grid-cols-2 gap-4 lg:grid-cols-4 lg:gap-6">
        {slots.map((slot, index) => (
          <SpecialtySlotCard
            key={`specialty-slot-${String(index)}`}
            slot={slot}
            index={index}
            isEnhanced={site.isEnhanced}
            canAfford={effectivePrice(slot) <= essence || site.isEnhanced}
            onBuy={handleBuy}
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

/** Props for a specialty shop slot card. */
interface SpecialtySlotCardProps {
  slot: ShopSlot;
  index: number;
  isEnhanced: boolean;
  canAfford: boolean;
  onBuy: (index: number) => void;
  onCardClick: (card: CardData) => void;
}

/** Renders a single slot in the specialty shop grid. */
function SpecialtySlotCard({
  slot,
  index,
  isEnhanced,
  canAfford,
  onBuy,
  onCardClick,
}: SpecialtySlotCardProps) {
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

  if (!slot.card) return null;

  const price = effectivePrice(slot);

  return (
    <div className="flex flex-col gap-2">
      <CardDisplay
        card={slot.card}
        onClick={() => onCardClick(slot.card!)}
      />
      <button
        className="flex w-full items-center justify-center gap-2 rounded-lg px-3 py-2 text-sm font-bold transition-opacity"
        style={{
          background: canAfford ? "#7c3aed" : "#4b5563",
          color: canAfford ? "#fbbf24" : "#9ca3af",
          opacity: canAfford ? 1 : 0.6,
          cursor: canAfford ? "pointer" : "not-allowed",
        }}
        disabled={!canAfford}
        onClick={() => onBuy(index)}
      >
        {isEnhanced ? (
          <span style={{ color: "#fbbf24" }}>FREE</span>
        ) : (
          <>
            <span>Buy</span>
            <span>{String(price)}</span>
            <span className="text-xs opacity-70">Essence</span>
          </>
        )}
      </button>
    </div>
  );
}
