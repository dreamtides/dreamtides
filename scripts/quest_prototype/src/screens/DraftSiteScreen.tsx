import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  sortCardsByTide,
  SITE_PICKS,
} from "../draft/draft-engine";
import type { DraftState } from "../types/draft";
import type { CardData } from "../types/cards";
import { cardAccentTide, cardImageUrl, TIDE_COLORS } from "../data/card-database";
import { logEvent } from "../logging";


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
        style={{ gridTemplateColumns: "repeat(3, minmax(0, 1fr))", alignItems: "start" }}
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

/** Compact deck sidebar showing all drafted cards sorted by energy cost. */
function DeckSidebar({
  cardDatabase,
}: {
  cardDatabase: Map<number, CardData>;
}) {
  const { state } = useQuest();

  const deckCards = useMemo(() => {
    const cards: CardData[] = [];
    for (const entry of state.deck) {
      const card = cardDatabase.get(entry.cardNumber);
      if (card) cards.push(card);
    }
    return cards.sort((a, b) => (a.energyCost ?? 0) - (b.energyCost ?? 0));
  }, [state.deck, cardDatabase]);

  if (deckCards.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-4">
        <p className="text-xs opacity-40">No cards drafted yet.</p>
      </div>
    );
  }

  // Group by energy cost
  let lastCost = -1;

  return (
    <div className="flex flex-col gap-0.5 overflow-y-auto p-2">
      {deckCards.map((card, i) => {
        const cost = card.energyCost ?? 0;
        const showDivider = cost !== lastCost;
        lastCost = cost;
        const tideColor = TIDE_COLORS[cardAccentTide(card)];

        return (
          <div key={`deck-${String(card.cardNumber)}-${String(i)}`}>
            {showDivider && (
              <div className="flex items-center gap-1.5 px-1 pt-1.5 pb-0.5">
                <span
                  className="flex h-4 w-4 items-center justify-center rounded-full text-[9px] font-bold"
                  style={{
                    background: "rgba(251, 191, 36, 0.2)",
                    color: "#fbbf24",
                    border: "1px solid rgba(251, 191, 36, 0.3)",
                  }}
                >
                  {String(cost)}
                </span>
                <div
                  className="h-px flex-1"
                  style={{ background: "rgba(251, 191, 36, 0.15)" }}
                />
              </div>
            )}
            <div
              className="relative flex items-center gap-2 overflow-hidden rounded px-2 py-1"
              style={{
                background: `linear-gradient(90deg, ${tideColor}15 0%, rgba(10, 6, 18, 0.7) 70%)`,
                borderLeft: `2px solid ${tideColor}60`,
              }}
            >
              <img
                src={cardImageUrl(card.cardNumber)}
                alt=""
                className="pointer-events-none absolute top-0 right-0 h-full object-cover"
                style={{
                  width: "40%",
                  maskImage: "linear-gradient(to right, transparent 0%, black 60%)",
                  WebkitMaskImage: "linear-gradient(to right, transparent 0%, black 60%)",
                  opacity: 0.25,
                }}
              />
              <span
                className="relative z-10 min-w-0 flex-1 truncate text-[11px] font-medium"
                style={{ color: "#e2e8f0" }}
              >
                {card.name}
              </span>
              <span
                className="relative z-10 flex h-4 w-4 shrink-0 items-center justify-center rounded-full text-[9px] font-bold"
                style={{
                  background: "rgba(251, 191, 36, 0.2)",
                  color: "#fbbf24",
                  border: "1px solid rgba(251, 191, 36, 0.3)",
                }}
              >
                {String(cost)}
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}

/** The draft site screen: 4-card pack display, card picking, and summary. */
export function DraftSiteScreen({ siteId }: { siteId: string }) {
  const { state, mutations, cardDatabase } = useQuest();
  const [pickPhase, setPickPhase] = useState<PickPhase>("idle");
  const [pickedCardNumber, setPickedCardNumber] = useState<number | null>(null);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);
  const [currentPackCards, setCurrentPackCards] = useState<CardData[]>([]);
  const [draftedCardNumbers, setDraftedCardNumbers] = useState<number[]>([]);
  const [isComplete, setIsComplete] = useState(false);
  const [packKey, setPackKey] = useState(0);
  const [showDeckSidebar, setShowDeckSidebar] = useState(false);
  const draftStateRef = useRef<DraftState | null>(null);
  const initializedRef = useRef(false);

  // Initialize draft state on mount
  useEffect(() => {
    if (initializedRef.current) return;
    if (cardDatabase.size === 0) return;
    initializedRef.current = true;

    let ds = state.draftState;
    if (ds === null) {
      if (state.resolvedPackage === null) {
        return;
      }
      ds = initializeDraftState(cardDatabase, state.resolvedPackage);
    }

    // Deep clone to avoid mutating React state directly
    const cloned = JSON.parse(JSON.stringify(ds)) as DraftState;
    enterDraftSite(cloned, cardDatabase);
    draftStateRef.current = cloned;
    mutations.setDraftState(cloned, "draft_site_enter");

    // Load initial pack, sorted by tide
    const packNums = getPlayerPack(cloned);
    const cards = packNums
      .map((num) => cardDatabase.get(num))
      .filter((c): c is CardData => c !== undefined);
    setCurrentPackCards(sortCardsByTide(cards));
    setDraftedCardNumbers([...cloned.draftedCardNumbers]);
  }, [state.draftState, state.resolvedPackage, cardDatabase, mutations]);

  // Resolve the current pack from draft state, sorted by tide
  const refreshPack = useCallback(() => {
    const ds = draftStateRef.current;
    if (!ds) return;
    const packNums = getPlayerPack(ds);
    const cards = packNums
      .map((num) => cardDatabase.get(num))
      .filter((c): c is CardData => c !== undefined);
    setCurrentPackCards(sortCardsByTide(cards));
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
        mutations.setDraftState(cloned, "draft_pick");

        // Add picked card to deck
        mutations.addCard(cardNumber, "draft_pick");

        setDraftedCardNumbers([...cloned.draftedCardNumbers]);

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

  const handleCardInspect = useCallback(
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
      cardsDrafted: draftedCardNumbers,
    });

    mutations.markSiteVisited(siteId);
    mutations.setScreen({ type: "dreamscape" });
  }, [siteId, draftedCardNumbers, mutations]);

  const pickNumber = draftedCardNumbers.length + 1;

  if (cardDatabase.size === 0) {
    return (
      <div className="flex min-h-screen flex-col items-center justify-center gap-4 px-4">
        <p className="text-lg opacity-60">
          Card database unavailable. Cannot start draft.
        </p>
        <button
          className="rounded-lg px-6 py-3 font-bold text-white transition-colors"
          style={{
            background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
            border: "1px solid rgba(168, 85, 247, 0.5)",
          }}
          onClick={() => {
            mutations.setScreen({ type: "dreamscape" });
          }}
        >
          Return to Dreamscape
        </button>
      </div>
    );
  }

  if (isComplete) {
    return (
      <DraftSummary
        draftedCardNumbers={draftedCardNumbers}
        cardDatabase={cardDatabase}
        onContinue={handleContinue}
      />
    );
  }

  {/*
    Layout: full viewport minus HUD (48px). Cards use viewport-relative
    heights so the 2x2 grid fills the screen. Each card is ~42vh tall
    (two rows + gap + header ≈ 100vh - 48px). Width follows from the
    2:3 aspect ratio.
  */}
  return (
    <div
      className="flex"
      style={{ height: "calc(100vh - 48px)" }}
    >
      {/* Main draft area */}
      <div className="flex flex-1 flex-col items-center justify-center">
        {/* Compact header */}
        <div className="flex w-full items-center justify-between px-4 py-1 md:px-8">
          <div className="flex items-center gap-3">
            <h2
              className="text-lg font-bold tracking-wide"
              style={{ color: "#a855f7" }}
            >
              Draft
            </h2>
            <span className="text-xs opacity-50">
              Pick {String(Math.min(pickNumber, SITE_PICKS))}/{String(SITE_PICKS)}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <div
              className="h-1.5 w-24 overflow-hidden rounded-full md:w-32"
              style={{ background: "rgba(124, 58, 237, 0.2)" }}
            >
              <motion.div
                className="h-full rounded-full"
                style={{ background: "#f97316" }}
                initial={false}
                animate={{
                  width: `${String((draftedCardNumbers.length / SITE_PICKS) * 100)}%`,
                }}
                transition={{ duration: 0.3 }}
              />
            </div>
            <button
              className="cursor-pointer rounded px-2 py-1 text-xs font-medium transition-colors"
              style={{
                background: showDeckSidebar
                  ? "rgba(124, 58, 237, 0.3)"
                  : "rgba(124, 58, 237, 0.15)",
                border: `1px solid ${showDeckSidebar ? "rgba(124, 58, 237, 0.6)" : "rgba(124, 58, 237, 0.3)"}`,
                color: showDeckSidebar ? "#c084fc" : "#9ca3af",
              }}
              onClick={() => {
                setShowDeckSidebar((prev) => !prev);
              }}
            >
              {"\uD83C\uDCCF"} {String(state.deck.length)}
            </button>
          </div>
        </div>

        {/* 2x2 card grid, centered */}
        <AnimatePresence mode="wait">
          <motion.div
            key={`pack-${String(packKey)}`}
            className="grid gap-3 md:gap-4"
            style={{
              gridTemplateColumns: "repeat(2, auto)",
              gridTemplateRows: "repeat(2, auto)",
            }}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.25 }}
          >
            {currentPackCards.map((card) => {
              const isPicked = pickedCardNumber === card.cardNumber;
              const isOther = pickedCardNumber !== null && !isPicked;

              return (
                <motion.div
                  key={`card-${String(card.cardNumber)}`}
                  initial={{ opacity: 0 }}
                  animate={
                    isPicked && pickPhase !== "idle"
                      ? { opacity: 0, scale: 0.9 }
                      : isOther && pickPhase !== "idle"
                        ? { opacity: 0, scale: 0.95 }
                        : { opacity: 1, scale: 1 }
                  }
                  transition={{ duration: 0.3 }}
                >
                  <div
                    className="relative rounded-lg transition-shadow duration-200"
                    style={{
                      height: "calc((100vh - 48px - 80px) / 2)",
                      aspectRatio: "2 / 3",
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
                    onContextMenu={(e) => {
                      e.preventDefault();
                      handleCardInspect(card);
                    }}
                  >
                    <CardDisplay
                      card={card}
                      className="h-full w-full"
                      large
                      onClick={
                        pickPhase === "idle"
                          ? () => {
                              handleCardPick(card.cardNumber);
                            }
                          : undefined
                      }
                    />
                  </div>
                </motion.div>
              );
            })}
          </motion.div>
        </AnimatePresence>

        {/* Drafted cards row */}
        <DraftedCardsRow
          cardNumbers={draftedCardNumbers}
          cardDatabase={cardDatabase}
        />
      </div>

      {/* Deck sidebar */}
      {showDeckSidebar && (
        <div
          className="w-56 shrink-0 overflow-hidden border-l lg:w-64"
          style={{
            borderColor: "rgba(124, 58, 237, 0.2)",
            background: "rgba(5, 2, 10, 0.6)",
          }}
        >
          <div
            className="px-3 py-2"
            style={{ borderBottom: "1px solid rgba(124, 58, 237, 0.15)" }}
          >
            <span className="text-xs font-bold uppercase tracking-wider" style={{ color: "#a855f7" }}>
              Deck ({String(state.deck.length)})
            </span>
          </div>
          <DeckSidebar cardDatabase={cardDatabase} />
        </div>
      )}

      {/* Card overlay */}
      <CardOverlay card={overlayCard} onClose={handleOverlayClose} />
    </div>
  );
}
