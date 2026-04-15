import { useCallback, useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { SiteState } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import {
  TEMPTING_OFFERS,
  type OfferEffect,
  type TemptingOffer,
} from "../data/tempting-offers";
import { DREAMSIGNS } from "../data/dreamsigns";

/** Props for the TemptingOfferScreen component. */
interface TemptingOfferScreenProps {
  site: SiteState;
}

/** Picks N random unique elements from an array. */
function pickRandom<T>(arr: readonly T[], count: number): T[] {
  const shuffled = [...arr].sort(() => Math.random() - 0.5);
  return shuffled.slice(0, count);
}

/** Describes an offer effect for logging. */
function describeOfferEffect(effect: OfferEffect): string {
  switch (effect.type) {
    case "addEssence":
      return `+${String(effect.amount)} essence`;
    case "addRandomCards":
      return `+${String(effect.count)} ${effect.rarity} card${effect.count === 1 ? "" : "s"}`;
    case "addTideCrystal":
      return "bonus removed";
    case "addMultipleTideCrystals":
      return "bonus removed";
    case "addBaneCards":
      return `+${String(effect.count)} bane card${effect.count === 1 ? "" : "s"}`;
    case "removeEssence":
      return `-${String(effect.amount)} essence`;
    case "removeDreamsign":
      return "Lose 1 random dreamsign";
    case "reduceMaxDreamsigns":
      return `-${String(effect.amount)} max dreamsign capacity`;
    case "removeRandomCards":
      return `-${String(effect.count)} random card${effect.count === 1 ? "" : "s"}`;
    case "addDreamsign":
      return "+1 random dreamsign";
  }
}

/** Shows 2 (or 3 enhanced) benefit/cost pairs. Accept one or skip. */
export function TemptingOfferScreen({ site }: TemptingOfferScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const { deck, dreamsigns: currentDreamsigns } = state;

  const pairCount = site.isEnhanced ? 3 : 2;

  const offers = useMemo<TemptingOffer[]>(
    () => pickRandom(TEMPTING_OFFERS, pairCount),
    [pairCount],
  );

  const [resultMessage, setResultMessage] = useState<string | null>(null);

  useEffect(() => {
    logEvent("site_entered", {
      siteType: "TemptingOffer",
      isEnhanced: site.isEnhanced,
      pairCount,
    });
  }, [site.isEnhanced, pairCount]);

  const completeSite = useCallback(() => {
    logEvent("site_completed", {
      siteType: "TemptingOffer",
      isEnhanced: site.isEnhanced,
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations]);

  /** Removes N random non-bane cards from the deck. */
  const removeRandomCards = useCallback(
    (count: number) => {
      const nonBaneEntries = deck.filter((e) => !e.isBane);
      const shuffled = [...nonBaneEntries].sort(() => Math.random() - 0.5);
      const toRemove = shuffled.slice(0, count);
      for (const entry of toRemove) {
        mutations.removeCard(entry.entryId, "tempting_offer");
      }
    },
    [deck, mutations],
  );

  /** Adds N random cards of a given rarity. */
  const addRandomCards = useCallback(
    (count: number, rarity: string) => {
      const cardsOfRarity = Array.from(cardDatabase.values()).filter(
        (c) => c.rarity === rarity,
      );
      if (cardsOfRarity.length === 0) return;
      for (let i = 0; i < count; i++) {
        const card =
          cardsOfRarity[Math.floor(Math.random() * cardsOfRarity.length)];
        mutations.addCard(card.cardNumber, "tempting_offer");
      }
    },
    [cardDatabase, mutations],
  );

  /** Adds N bane cards (random common cards flagged as bane). */
  const addBaneCards = useCallback(
    (count: number) => {
      const commonCards = Array.from(cardDatabase.values()).filter(
        (c) => c.rarity === "Common",
      );
      if (commonCards.length === 0) return;
      for (let i = 0; i < count; i++) {
        const card =
          commonCards[Math.floor(Math.random() * commonCards.length)];
        mutations.addBaneCard(card.cardNumber, "tempting_offer");
      }
    },
    [cardDatabase, mutations],
  );

  const applyEffect = useCallback(
    (effect: OfferEffect) => {
      switch (effect.type) {
        case "addEssence":
          mutations.changeEssence(effect.amount, "tempting_offer");
          break;
        case "addRandomCards":
          addRandomCards(effect.count, effect.rarity);
          break;
        case "addTideCrystal":
          logEvent("legacy_reward_skipped", {
            sourceSiteType: "TemptingOffer",
            legacyType: "tide_crystal",
            tide: effect.tide,
            count: effect.count,
          });
          break;
        case "addMultipleTideCrystals":
          for (const crystal of effect.crystals) {
            logEvent("legacy_reward_skipped", {
              sourceSiteType: "TemptingOffer",
              legacyType: "tide_crystal",
              tide: crystal.tide,
              count: crystal.count,
            });
          }
          break;
        case "addBaneCards":
          addBaneCards(effect.count);
          break;
        case "removeEssence":
          mutations.changeEssence(-effect.amount, "tempting_offer");
          break;
        case "removeDreamsign":
          if (currentDreamsigns.length > 0) {
            const index = Math.floor(
              Math.random() * currentDreamsigns.length,
            );
            mutations.removeDreamsign(index, "tempting_offer_cost");
          }
          break;
        case "reduceMaxDreamsigns":
          // Log the reduction; actual max enforcement happens at dreamsign acquisition
          logEvent("max_dreamsigns_reduced", { amount: effect.amount });
          break;
        case "removeRandomCards":
          removeRandomCards(effect.count);
          break;
        case "addDreamsign": {
          const template =
            DREAMSIGNS[Math.floor(Math.random() * DREAMSIGNS.length)];
          mutations.addDreamsign(
            { ...template, isBane: false },
            "TemptingOffer",
          );
          break;
        }
      }
    },
    [
      mutations,
      currentDreamsigns,
      addRandomCards,
      addBaneCards,
      removeRandomCards,
    ],
  );

  const handleAccept = useCallback(
    (offer: TemptingOffer) => {
      // Apply benefit first, then cost
      applyEffect(offer.benefit);
      applyEffect(offer.cost);

      const baneCount =
        offer.cost.type === "addBaneCards" ? offer.cost.count : 0;

      logEvent("tempting_offer_accepted", {
        benefitDescription: offer.benefitDescription,
        costDescription: offer.costDescription,
        benefitEffect: describeOfferEffect(offer.benefit),
        costEffect: describeOfferEffect(offer.cost),
        baneCardsAdded: baneCount,
      });

      setResultMessage(
        `${describeOfferEffect(offer.benefit)} / ${describeOfferEffect(offer.cost)}`,
      );

      setTimeout(completeSite, 1500);
    },
    [applyEffect, completeSite],
  );

  const handleSkip = useCallback(() => {
    completeSite();
  }, [completeSite]);

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
          style={{ color: "#ef4444" }}
        >
          Tempting Offer
        </h2>
        <p className="mt-1 text-sm opacity-50">
          Accept a bargain or walk away
        </p>
        {site.isEnhanced && (
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(168, 85, 247, 0.15)",
              color: "#c084fc",
              border: "1px solid rgba(168, 85, 247, 0.3)",
            }}
          >
            Enhanced -- 3 Offers
          </span>
        )}
      </div>

      {/* Result overlay */}
      <AnimatePresence>
        {resultMessage !== null && (
          <motion.div
            className="fixed inset-0 z-50 flex items-center justify-center"
            style={{ background: "rgba(0, 0, 0, 0.7)" }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <motion.div
              className="rounded-xl px-8 py-6 text-center"
              style={{
                background:
                  "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
                border: "1px solid rgba(239, 68, 68, 0.4)",
                boxShadow: "0 0 30px rgba(239, 68, 68, 0.2)",
              }}
              initial={{ scale: 0.8, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.8, opacity: 0 }}
            >
              <p className="text-xl font-bold" style={{ color: "#fca5a5" }}>
                {resultMessage}
              </p>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Offer pairs */}
      <div className="flex w-full max-w-3xl flex-col gap-4">
        {offers.map((offer, index) => (
          <motion.div
            key={`offer-${String(index)}`}
            className="flex flex-col gap-3 rounded-xl p-4"
            style={{
              background:
                "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
              border: "1px solid rgba(107, 114, 128, 0.2)",
            }}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.15, duration: 0.4 }}
          >
            <div className="flex flex-col gap-3 md:flex-row md:items-stretch">
              {/* Benefit */}
              <div
                className="flex flex-1 flex-col gap-2 rounded-lg p-3"
                style={{
                  background: "rgba(16, 185, 129, 0.05)",
                  border: "1px solid rgba(16, 185, 129, 0.2)",
                }}
              >
                <span
                  className="text-[10px] font-bold uppercase tracking-wider"
                  style={{ color: "#10b981" }}
                >
                  Benefit
                </span>
                <p
                  className="text-sm leading-relaxed"
                  style={{ color: "#a7f3d0" }}
                >
                  {offer.benefitDescription}
                </p>
              </div>

              {/* Cost */}
              <div
                className="flex flex-1 flex-col gap-2 rounded-lg p-3"
                style={{
                  background: "rgba(239, 68, 68, 0.05)",
                  border: "1px solid rgba(239, 68, 68, 0.2)",
                }}
              >
                <span
                  className="text-[10px] font-bold uppercase tracking-wider"
                  style={{ color: "#ef4444" }}
                >
                  Cost
                </span>
                <p
                  className="text-sm leading-relaxed"
                  style={{ color: "#fca5a5" }}
                >
                  {offer.costDescription}
                </p>
              </div>
            </div>

            {/* Accept button */}
            <button
              className="w-full rounded-lg px-5 py-2.5 font-bold text-white transition-opacity"
              style={{
                background:
                  "linear-gradient(135deg, #7c3aed 0%, #ef4444 100%)",
              }}
              disabled={resultMessage !== null}
              onClick={() => handleAccept(offer)}
            >
              Accept Offer
            </button>
          </motion.div>
        ))}
      </div>

      {/* Skip */}
      <button
        className="mt-8 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
        style={{
          background: "rgba(107, 114, 128, 0.2)",
          border: "1px solid rgba(107, 114, 128, 0.4)",
          color: "#9ca3af",
        }}
        disabled={resultMessage !== null}
        onClick={handleSkip}
      >
        Skip
      </button>
    </motion.div>
  );
}
