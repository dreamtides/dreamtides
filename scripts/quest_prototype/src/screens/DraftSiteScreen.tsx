import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { CardDisplay } from "../components/CardDisplay";
import { CardOverlay } from "../components/CardOverlay";
import {
  enterDraftSite,
  getPlayerPack,
  initializeDraftState,
  processPlayerPick,
  completeDraftSite,
} from "../draft/draft-engine";
import type { DraftState } from "../types/draft";
import type { CardData } from "../types/cards";
import { logEvent } from "../logging";

/** Number of player picks per draft site visit. */
const SITE_PICKS = 5;

/** Delay in ms before showing the next pack after a pick. */
const NEXT_PACK_DELAY = 500;

/** Animation phases during a pick. */
type PickPhase = "idle" | "animating" | "waiting";

/** Shows the drafted cards as a row of mini-cards at the bottom. */
function DraftedCardsRow({ cardNumbers, cardDatabase }: {
  cardNumbers: number[];
  cardDatabase: Map<number, CardData>;
}) {
  if (cardNumbers.length === 0) {
    return null;
  }

  return (
    <div className="flex flex-col items-center gap-1.5 pt-2">
      <span className="text-xs font-medium opacity-50">Drafted</span>
      <div className="flex gap-2">
        {cardNumbers.map((num, i) => {
          const card = cardDatabase.get(num);
          if (!card) return null;
          return (
            <motion.div
              key={`drafted-${String(i)}-${String(num)}`}
              initial={{ opacity: 0, scale: 0.5, y: 20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              transition={{ duration: 0.3, delay: 0.1 }}
              className="rounded px-2 py-1 text-[10px] font-medium"
              style={{
                background: "rgba(26, 16, 37, 0.8)",
                border: "1px solid rgba(124, 58, 237, 0.3)",
                color: "#c084fc",
                maxWidth: "100px",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {card.name}
            </motion.div>
          );
        })}
      </div>
    </div>
  );
}

/** Summary screen shown after all 5 picks are complete. */
function DraftSummary({
  draftedCardNumbers,
  cardDatabase,
  onContinue,
}: {
  draftedCardNumbers: number[];
  cardDatabase: Map<number, CardData>;
  onContinue: () => void;
}) {
  const draftedCards = draftedCardNumbers
    .map((num) => cardDatabase.get(num))
    .filter((c): c is CardData => c !== undefined);

  return (
    <motion.div
      className="flex flex-col items-center gap-6 px-4 py-6"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
    >
      <h2
        className="text-2xl font-bold tracking-wide"
        style={{ color: "#a855f7" }}
      >
        Draft Complete
      </h2>
      <p className="text-sm opacity-60">
        {String(draftedCards.length)} cards added to your deck
      </p>

      <div
        className="draft-summary-grid grid w-full max-w-4xl gap-3 md:gap-4"
        style={{ gridTemplateColumns: "repeat(3, 1fr)" }}
      >
        {draftedCards.map((card, i) => (
          <motion.div
            key={`summary-${String(i)}-${String(card.cardNumber)}`}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.3, delay: i * 0.1 }}
          >
            <CardDisplay card={card} />
          </motion.div>
        ))}
      </div>

      <button
        className="mt-4 rounded-lg px-6 py-3 font-bold text-white transition-colors"
        style={{
          background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
          border: "1px solid rgba(168, 85, 247, 0.5)",
        }}
        onClick={onContinue}
      >
        Continue
      </button>
    </motion.div>
  );
}

/** The draft site screen: pack display, card picking, and summary. */
export function DraftSiteScreen({ siteId }: { siteId: string }) {
  const { state, mutations, cardDatabase } = useQuest();
  const [pickPhase, setPickPhase] = useState<PickPhase>("idle");
  const [pickedCardNumber, setPickedCardNumber] = useState<number | null>(null);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);
  const [currentPackCards, setCurrentPackCards] = useState<CardData[]>([]);
  const [draftedThisSite, setDraftedThisSite] = useState<number[]>([]);
  const [isComplete, setIsComplete] = useState(false);
  const [packKey, setPackKey] = useState(0);
  const draftStateRef = useRef<DraftState | null>(null);
  const initializedRef = useRef(false);

  // Initialize draft state on mount
  useEffect(() => {
    if (initializedRef.current) return;
    initializedRef.current = true;

    let ds = state.draftState;
    if (ds === null) {
      ds = initializeDraftState(cardDatabase);
    }

    // Deep clone to avoid mutating React state directly
    const cloned = JSON.parse(JSON.stringify(ds)) as DraftState;
    enterDraftSite(cloned, cardDatabase);
    draftStateRef.current = cloned;
    mutations.setDraftState(cloned);

    // Load initial pack
    const packNums = getPlayerPack(cloned);
    const cards = packNums
      .map((num) => cardDatabase.get(num))
      .filter((c): c is CardData => c !== undefined);
    setCurrentPackCards(cards);
  }, [state.draftState, cardDatabase, mutations]);

  // Resolve the current pack from draft state
  const refreshPack = useCallback(() => {
    const ds = draftStateRef.current;
    if (!ds) return;
    const packNums = getPlayerPack(ds);
    const cards = packNums
      .map((num) => cardDatabase.get(num))
      .filter((c): c is CardData => c !== undefined);
    setCurrentPackCards(cards);
    setPackKey((prev) => prev + 1);
  }, [cardDatabase]);

  const handleCardPick = useCallback(
    (cardNumber: number) => {
      if (pickPhase !== "idle") return;
      const ds = draftStateRef.current;
      if (!ds) return;

      setPickedCardNumber(cardNumber);
      setPickPhase("animating");

      // Close overlay if open
      setOverlayCard(null);

      // After animation, process the pick
      setTimeout(() => {
        setPickPhase("waiting");

        // Clone to avoid direct mutation
        const cloned = JSON.parse(JSON.stringify(ds)) as DraftState;
        const siteComplete = processPlayerPick(
          cardNumber,
          cloned,
          cardDatabase,
        );
        draftStateRef.current = cloned;
        mutations.setDraftState(cloned);

        // Add picked card to deck
        mutations.addCard(cardNumber, "draft_pick");

        // Track drafted cards for this site
        setDraftedThisSite((prev) => [...prev, cardNumber]);

        if (siteComplete) {
          completeDraftSite(cloned);
          setTimeout(() => {
            setIsComplete(true);
            setPickPhase("idle");
            setPickedCardNumber(null);
          }, NEXT_PACK_DELAY);
        } else {
          // Show next pack after a brief pause
          setTimeout(() => {
            refreshPack();
            setPickPhase("idle");
            setPickedCardNumber(null);
          }, NEXT_PACK_DELAY);
        }
      }, 300);
    },
    [pickPhase, cardDatabase, mutations, refreshPack],
  );

  const handleCardHover = useCallback(
    (card: CardData) => {
      if (pickPhase === "idle") {
        setOverlayCard(card);
      }
    },
    [pickPhase],
  );

  const handleOverlayClose = useCallback(() => {
    setOverlayCard(null);
  }, []);

  const handleContinue = useCallback(() => {
    // Log completion
    logEvent("draft_site_completed_ui", {
      siteId,
      cardsDrafted: draftedThisSite,
    });

    mutations.markSiteVisited(siteId);
    mutations.setScreen({ type: "dreamscape" });
  }, [siteId, draftedThisSite, mutations]);

  const pickNumber = draftedThisSite.length + 1;
  const packSize = currentPackCards.length;

  if (isComplete) {
    return (
      <DraftSummary
        draftedCardNumbers={draftedThisSite}
        cardDatabase={cardDatabase}
        onContinue={handleContinue}
      />
    );
  }

  return (
    <div className="flex min-h-screen flex-col items-center pb-16">
      {/* Header with progress info */}
      <div className="flex w-full items-center justify-between px-4 py-4 md:px-8">
        <div className="flex flex-col gap-1">
          <h2
            className="text-xl font-bold tracking-wide md:text-2xl"
            style={{ color: "#a855f7" }}
          >
            Draft
          </h2>
          <span className="text-xs opacity-50">
            {String(packSize)} cards in pack
          </span>
        </div>

        {/* Progress indicator */}
        <div className="flex flex-col items-end gap-1">
          <span
            className="text-sm font-bold md:text-base"
            style={{ color: "#f97316" }}
          >
            Pick {String(Math.min(pickNumber, SITE_PICKS))} of{" "}
            {String(SITE_PICKS)}
          </span>
          {/* Progress bar */}
          <div
            className="h-1.5 w-24 overflow-hidden rounded-full md:w-32"
            style={{ background: "rgba(124, 58, 237, 0.2)" }}
          >
            <motion.div
              className="h-full rounded-full"
              style={{ background: "#f97316" }}
              initial={false}
              animate={{
                width: `${String((draftedThisSite.length / SITE_PICKS) * 100)}%`,
              }}
              transition={{ duration: 0.3 }}
            />
          </div>
        </div>
      </div>

      {/* Pack grid */}
      <div className="w-full max-w-6xl flex-1">
        <AnimatePresence mode="wait">
          <motion.div
            key={`pack-${String(packKey)}`}
            initial={{ opacity: 0, x: 40 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -40 }}
            transition={{ duration: 0.3 }}
          >
            <div
              className="draft-pack-grid grid w-full gap-3 px-2 md:gap-4 md:px-4"
              style={{
                gridTemplateColumns: "repeat(3, 1fr)",
              }}
            >
              <AnimatePresence>
                {currentPackCards.map((card) => {
                  const isPicked =
                    pickedCardNumber === card.cardNumber;
                  const isOther =
                    pickedCardNumber !== null && !isPicked;

                  return (
                    <motion.div
                      key={`card-${String(card.cardNumber)}`}
                      layout
                      initial={{ opacity: 0, y: 30 }}
                      animate={
                        isPicked && pickPhase === "animating"
                          ? { opacity: 0, y: 80, scale: 0.9 }
                          : isOther && pickPhase === "animating"
                            ? { opacity: 0, scale: 0.95 }
                            : { opacity: 1, y: 0, scale: 1 }
                      }
                      exit={{ opacity: 0, scale: 0.95 }}
                      transition={{ duration: 0.3 }}
                    >
                      <div
                        className="relative rounded-lg transition-shadow duration-200"
                        style={{
                          boxShadow: "none",
                        }}
                        onMouseEnter={(e) => {
                          if (pickPhase === "idle") {
                            e.currentTarget.style.boxShadow =
                              "0 0 0 3px #f97316, 0 0 16px rgba(249, 115, 22, 0.4)";
                          }
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.boxShadow = "none";
                        }}
                      >
                        <CardDisplay
                          card={card}
                          onClick={
                            pickPhase === "idle"
                              ? () => {
                                  handleCardPick(card.cardNumber);
                                }
                              : undefined
                          }
                          onHover={() => {
                            handleCardHover(card);
                          }}
                        />
                      </div>
                    </motion.div>
                  );
                })}
              </AnimatePresence>
            </div>
          </motion.div>
        </AnimatePresence>
      </div>

      {/* Drafted cards row */}
      <DraftedCardsRow
        cardNumbers={draftedThisSite}
        cardDatabase={cardDatabase}
      />

      {/* Card overlay */}
      <CardOverlay card={overlayCard} onClose={handleOverlayClose} />
    </div>
  );
}
