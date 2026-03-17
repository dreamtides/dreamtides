import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { motion } from "framer-motion";
import type { SiteState, DeckEntry } from "../types/quest";
import type { CardData } from "../types/cards";
import { CardDisplay } from "../components/CardDisplay";
import { CardOverlay } from "../components/CardOverlay";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";
import {
  assignTransfiguration,
  TRANSFIGURATION_COLORS,
  transfigurationEffectDetails,
  type TransfigurationOffer,
} from "../transfiguration/transfiguration-logic";

/** Props for the TransfigurationSiteScreen component. */
interface TransfigurationSiteScreenProps {
  site: SiteState;
}

/** A deck entry paired with its assigned transfiguration offer. */
interface TransfigurationCandidate {
  entry: DeckEntry;
  card: CardData;
  offer: TransfigurationOffer;
}

/** Selects up to count random untransfigured cards that have eligible transfigurations. */
function selectCandidates(
  deck: DeckEntry[],
  cardDatabase: Map<number, CardData>,
  count: number,
): TransfigurationCandidate[] {
  const eligible: TransfigurationCandidate[] = [];
  const shuffled = [...deck].sort(() => Math.random() - 0.5);
  for (const entry of shuffled) {
    if (eligible.length >= count) break;
    if (entry.transfiguration !== null) continue;
    const card = cardDatabase.get(entry.cardNumber);
    if (!card) continue;
    const offer = assignTransfiguration(card, entry.transfiguration);
    if (offer) {
      eligible.push({ entry, card, offer });
    }
  }
  return eligible;
}

/** Renders the Transfiguration site screen. */
export function TransfigurationSiteScreen({
  site,
}: TransfigurationSiteScreenProps) {
  const { state, mutations, cardDatabase } = useQuest();
  const { deck } = state;

  const [candidates] = useState<TransfigurationCandidate[]>(() =>
    selectCandidates(deck, cardDatabase, 3),
  );
  const [acceptedEntryIds, setAcceptedEntryIds] = useState<Set<string>>(
    new Set(),
  );
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);

  // Enhanced mode: pick from full deck
  const [enhancedPickedEntry, setEnhancedPickedEntry] =
    useState<DeckEntry | null>(null);
  const [enhancedOffer, setEnhancedOffer] =
    useState<TransfigurationOffer | null>(null);
  const [enhancedAccepted, setEnhancedAccepted] = useState(false);

  const untransfiguredDeck = useMemo(
    () => deck.filter((e) => e.transfiguration === null),
    [deck],
  );

  const autoReturnTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (autoReturnTimer.current !== null) {
        clearTimeout(autoReturnTimer.current);
      }
    };
  }, []);

  const handleAccept = useCallback(
    (candidate: TransfigurationCandidate) => {
      if (acceptedEntryIds.size > 0) return;
      mutations.transfigureCard(
        candidate.entry.entryId,
        candidate.offer.type,
        candidate.offer.description,
        transfigurationEffectDetails(candidate.offer, candidate.card),
      );
      setAcceptedEntryIds(new Set([candidate.entry.entryId]));

      logEvent("site_completed", {
        siteType: "Transfiguration",
        outcome: "completed",
      });
      autoReturnTimer.current = setTimeout(() => {
        autoReturnTimer.current = null;
        mutations.markSiteVisited(site.id);
        mutations.setScreen({ type: "dreamscape" });
      }, 800);
    },
    [mutations, acceptedEntryIds.size, site.id],
  );

  const handleEnhancedPick = useCallback(
    (entry: DeckEntry) => {
      const card = cardDatabase.get(entry.cardNumber);
      if (!card) return;
      const offer = assignTransfiguration(card, entry.transfiguration);
      if (!offer) return;
      setEnhancedPickedEntry(entry);
      setEnhancedOffer(offer);
    },
    [cardDatabase],
  );

  const handleEnhancedAccept = useCallback(() => {
    if (!enhancedPickedEntry || !enhancedOffer) return;
    const card = cardDatabase.get(enhancedPickedEntry.cardNumber);
    if (!card) return;
    mutations.transfigureCard(
      enhancedPickedEntry.entryId,
      enhancedOffer.type,
      enhancedOffer.description,
      transfigurationEffectDetails(enhancedOffer, card),
    );
    setEnhancedAccepted(true);

    logEvent("site_completed", {
      siteType: "Transfiguration",
      outcome: "completed",
    });
    autoReturnTimer.current = setTimeout(() => {
      autoReturnTimer.current = null;
      mutations.markSiteVisited(site.id);
      mutations.setScreen({ type: "dreamscape" });
    }, 800);
  }, [enhancedPickedEntry, enhancedOffer, mutations, cardDatabase, site.id]);

  const handleEnhancedReject = useCallback(() => {
    setEnhancedPickedEntry(null);
    setEnhancedOffer(null);
  }, []);

  const handleClose = useCallback(() => {
    if (autoReturnTimer.current !== null) {
      clearTimeout(autoReturnTimer.current);
      autoReturnTimer.current = null;
    }
    if (acceptedEntryIds.size === 0 && !enhancedAccepted) {
      logEvent("site_completed", {
        siteType: "Transfiguration",
        outcome: "skipped",
      });
    }
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [mutations, site.id, acceptedEntryIds.size, enhancedAccepted]);

  // Enhanced mode: full deck browser for picking
  if (site.isEnhanced) {
    return (
      <motion.div
        className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -20 }}
        transition={{ duration: 0.4 }}
      >
        <div className="mb-6 text-center">
          <h2
            className="text-2xl font-bold tracking-wide md:text-3xl"
            style={{ color: "#a855f7" }}
          >
            Transfiguration
          </h2>
          <span
            className="mt-2 inline-block rounded-full px-3 py-1 text-sm font-bold"
            style={{
              background: "rgba(168, 85, 247, 0.15)",
              color: "#c084fc",
              border: "1px solid rgba(168, 85, 247, 0.3)",
            }}
          >
            Enhanced -- Choose any card
          </span>
          <p className="mt-2 text-sm opacity-60">
            {enhancedPickedEntry
              ? "Review the transfiguration preview below."
              : "Pick any untransfigured card from your deck to transfigure."}
          </p>
        </div>

        {!enhancedPickedEntry && (
          <div className="grid w-full max-w-5xl grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6">
            {untransfiguredDeck.map((entry, index) => {
              const card = cardDatabase.get(entry.cardNumber);
              if (!card) return null;
              return (
                <motion.div
                  key={entry.entryId}
                  initial={{ opacity: 0, scale: 0.9 }}
                  animate={{ opacity: 1, scale: 1 }}
                  transition={{ duration: 0.3, delay: index * 0.03 }}
                >
                  <CardDisplay
                    card={card}
                    onClick={() => handleEnhancedPick(entry)}
                    onHover={() => setOverlayCard(card)}
                  />
                </motion.div>
              );
            })}
          </div>
        )}

        {enhancedPickedEntry && enhancedOffer && !enhancedAccepted && (
          <EnhancedPreview
            entry={enhancedPickedEntry}
            offer={enhancedOffer}
            cardDatabase={cardDatabase}
            onAccept={handleEnhancedAccept}
            onReject={handleEnhancedReject}
          />
        )}

        {enhancedAccepted && (
          <motion.div
            className="mt-4 text-center"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
          >
            <p className="text-lg font-bold" style={{ color: "#10b981" }}>
              Transfiguration applied!
            </p>
          </motion.div>
        )}

        <button
          className="mt-8 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={handleClose}
        >
          Close
        </button>

        <CardOverlay card={overlayCard} onClose={() => setOverlayCard(null)} />
      </motion.div>
    );
  }

  // Normal mode: 3 random candidates
  return (
    <motion.div
      className="flex min-h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      <div className="mb-6 text-center">
        <h2
          className="text-2xl font-bold tracking-wide md:text-3xl"
          style={{ color: "#a855f7" }}
        >
          Transfiguration
        </h2>
        <p className="mt-2 text-sm opacity-60">
          Apply a magical enhancement to one of these cards.
        </p>
      </div>

      {candidates.length === 0 ? (
        <p className="mt-8 text-lg opacity-50">
          No eligible cards found for transfiguration.
        </p>
      ) : (
        <div className="flex w-full max-w-5xl flex-col gap-6 md:flex-row md:justify-center">
          {candidates.map((candidate, index) => {
            const isAccepted = acceptedEntryIds.has(candidate.entry.entryId);
            const color = TRANSFIGURATION_COLORS[candidate.offer.type];
            return (
              <motion.div
                key={candidate.entry.entryId}
                className="flex flex-col items-center gap-3 rounded-xl p-4"
                style={{
                  background: "rgba(15, 10, 24, 0.8)",
                  border: `1px solid ${color}40`,
                  boxShadow: `0 0 12px ${color}15`,
                  flex: "1 1 0",
                  maxWidth: "280px",
                }}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.4, delay: index * 0.15 }}
              >
                {/* Original card */}
                <div style={{ width: "100%" }}>
                  <CardDisplay card={candidate.card} />
                </div>

                {/* Arrow / becomes indicator */}
                <div
                  className="flex flex-col items-center gap-1 py-1"
                  style={{ color }}
                >
                  <span className="text-2xl">{"\u2193"}</span>
                  <span
                    className="rounded-full px-3 py-0.5 text-xs font-bold uppercase tracking-wider"
                    style={{
                      background: `${color}20`,
                      border: `1px solid ${color}40`,
                    }}
                  >
                    {candidate.offer.type}
                  </span>
                  <span className="mt-1 text-xs opacity-80">
                    {candidate.offer.description}
                  </span>
                </div>

                {/* Preview card */}
                <div
                  style={{
                    width: "100%",
                    filter: `drop-shadow(0 0 6px ${color}40)`,
                  }}
                >
                  <CardDisplay
                    card={candidate.offer.previewCard}
                    selected={true}
                    selectionColor={color}
                    tintColor={color}
                  />
                </div>

                {/* Accept button */}
                {isAccepted ? (
                  <div
                    className="w-full rounded-lg px-4 py-2 text-center text-sm font-bold"
                    style={{
                      background: `${color}20`,
                      color,
                      border: `1px solid ${color}40`,
                    }}
                  >
                    Applied!
                  </div>
                ) : acceptedEntryIds.size > 0 ? (
                  <div
                    className="w-full rounded-lg px-4 py-2 text-center text-sm font-bold opacity-40"
                    style={{
                      background: "#4b5563",
                      color: "#9ca3af",
                    }}
                  >
                    Unavailable
                  </div>
                ) : (
                  <button
                    className="w-full rounded-lg px-4 py-2 text-sm font-bold transition-opacity hover:opacity-90"
                    style={{
                      background: `linear-gradient(135deg, ${color} 0%, ${color}cc 100%)`,
                      color: "#ffffff",
                    }}
                    onClick={() => handleAccept(candidate)}
                  >
                    Accept {candidate.offer.type}
                  </button>
                )}
              </motion.div>
            );
          })}
        </div>
      )}

      <button
        className="mt-8 rounded-lg px-6 py-2.5 text-base font-medium transition-colors"
        style={{
          background: "rgba(107, 114, 128, 0.2)",
          border: "1px solid rgba(107, 114, 128, 0.4)",
          color: "#9ca3af",
        }}
        onClick={handleClose}
      >
        Close
      </button>

      <CardOverlay card={overlayCard} onClose={() => setOverlayCard(null)} />
    </motion.div>
  );
}

/** Renders the enhanced mode preview for a single picked card. */
function EnhancedPreview({
  entry,
  offer,
  cardDatabase,
  onAccept,
  onReject,
}: {
  entry: DeckEntry;
  offer: TransfigurationOffer;
  cardDatabase: Map<number, CardData>;
  onAccept: () => void;
  onReject: () => void;
}) {
  const card = cardDatabase.get(entry.cardNumber);
  if (!card) return null;

  const color = TRANSFIGURATION_COLORS[offer.type];

  return (
    <motion.div
      className="flex flex-col items-center gap-4 rounded-xl p-6"
      style={{
        background: "rgba(15, 10, 24, 0.8)",
        border: `1px solid ${color}40`,
        boxShadow: `0 0 16px ${color}20`,
        maxWidth: "320px",
      }}
      initial={{ opacity: 0, scale: 0.95 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ duration: 0.3 }}
    >
      <div style={{ width: "100%" }}>
        <CardDisplay card={card} />
      </div>

      <div className="flex flex-col items-center gap-1" style={{ color }}>
        <span className="text-2xl">{"\u2193"}</span>
        <span
          className="rounded-full px-3 py-0.5 text-xs font-bold uppercase tracking-wider"
          style={{
            background: `${color}20`,
            border: `1px solid ${color}40`,
          }}
        >
          {offer.type}
        </span>
        <span className="mt-1 text-xs opacity-80">{offer.description}</span>
      </div>

      <div
        style={{
          width: "100%",
          filter: `drop-shadow(0 0 6px ${color}40)`,
        }}
      >
        <CardDisplay
          card={offer.previewCard}
          selected={true}
          selectionColor={color}
          tintColor={color}
        />
      </div>

      <div className="flex w-full gap-2">
        <button
          className="flex-1 rounded-lg px-4 py-2 text-sm font-bold transition-opacity hover:opacity-90"
          style={{
            background: `linear-gradient(135deg, ${color} 0%, ${color}cc 100%)`,
            color: "#ffffff",
          }}
          onClick={onAccept}
        >
          Accept
        </button>
        <button
          className="flex-1 rounded-lg px-4 py-2 text-sm font-bold transition-opacity hover:opacity-90"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={onReject}
        >
          Reject
        </button>
      </div>
    </motion.div>
  );
}
