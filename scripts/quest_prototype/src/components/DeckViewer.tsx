import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Rarity } from "../types/cards";
import type {
  DeckEntry,
  QuestState,
} from "../types/quest";
import { useQuest } from "../state/quest-context";
import {
  RARITY_COLORS,
  TIDE_COLORS,
  tideIconUrl,
} from "../data/card-database";
import { CardDisplay } from "./CardDisplay";
import { CardOverlay } from "./CardOverlay";
import { logEvent } from "../logging";
import { TRANSFIGURATION_COLORS } from "../transfiguration/transfiguration-logic";
import { ALL_RARITIES, computeDeckSummary } from "./deck-summary";

/** Sort criteria options. */
type SortCriteria =
  | "acquisitionOrder"
  | "energyCost"
  | "name"
  | "rarity"
  | "cardType";

/** Labels for sort criteria. */
const SORT_LABELS: Readonly<Record<SortCriteria, string>> = {
  acquisitionOrder: "Acquisition Order",
  energyCost: "Energy Cost",
  name: "Name",
  rarity: "Rarity",
  cardType: "Card Type",
};

/** Card type filter options. */
type CardTypeFilter = "All" | "Characters" | "Events";

/** Rarity ordering for sorting. */
const RARITY_ORDER: Readonly<Record<Rarity, number>> = {
  Common: 0,
  Uncommon: 1,
  Rare: 2,
  Legendary: 3,
};

/** A deck entry paired with its resolved card data. */
interface ResolvedEntry {
  entry: DeckEntry;
  card: CardData;
  index: number;
}

/** Props for the DeckViewer component. */
interface DeckViewerProps {
  isOpen: boolean;
  onClose: () => void;
  cardDatabase: Map<number, CardData>;
}

/**
 * Full-screen overlay showing the player's complete deck, with filtering,
 * sorting, and a sidebar for dreamcaller and dreamsign data.
 */
export function DeckViewer({
  isOpen,
  onClose,
  cardDatabase,
}: DeckViewerProps) {
  const { state } = useQuest();

  const [rarityFilters, setRarityFilters] = useState<Record<Rarity, boolean>>(
    () => {
      const filters: Partial<Record<Rarity, boolean>> = {};
      for (const rarity of ALL_RARITIES) {
        filters[rarity] = true;
      }
      return filters as Record<Rarity, boolean>;
    },
  );
  const [cardTypeFilter, setCardTypeFilter] =
    useState<CardTypeFilter>("All");
  const [sortCriteria, setSortCriteria] =
    useState<SortCriteria>("acquisitionOrder");
  const [sortAscending, setSortAscending] = useState(true);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);
  const [showSortDropdown, setShowSortDropdown] = useState(false);
  const openTimestampRef = useRef<number>(0);
  const prevOpenRef = useRef(false);

  useEffect(() => {
    if (isOpen && !prevOpenRef.current) {
      openTimestampRef.current = Date.now();
      logEvent("deck_viewer_opened", {
        cardCount: state.deck.length,
        disabledRarities: ALL_RARITIES.filter((rarity) => !rarityFilters[rarity]),
        sortCriteria,
        sortAscending,
        cardTypeFilter,
      });
    }
    prevOpenRef.current = isOpen;
  }, [
    isOpen,
    state.deck.length,
    rarityFilters,
    sortCriteria,
    sortAscending,
    cardTypeFilter,
  ]);

  const handleClose = useCallback(() => {
    const duration = Date.now() - openTimestampRef.current;
    logEvent("deck_viewer_closed", { durationMs: duration });
    onClose();
  }, [onClose]);

  useEffect(() => {
    if (!isOpen) return undefined;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        if (overlayCard !== null) {
          return;
        }
        handleClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, overlayCard, handleClose]);

  const resolvedEntries = useMemo<ResolvedEntry[]>(() => {
    return state.deck
      .map((entry, index) => {
        const card = cardDatabase.get(entry.cardNumber);
        if (!card) return null;
        return { entry, card, index };
      })
      .filter((e): e is ResolvedEntry => e !== null);
  }, [state.deck, cardDatabase]);

  const deckSummary = useMemo(
    () => computeDeckSummary(state.deck, cardDatabase),
    [state.deck, cardDatabase],
  );

  const hasActiveFilters = useMemo(
    () =>
      ALL_RARITIES.some((rarity) => !rarityFilters[rarity])
      || cardTypeFilter !== "All",
    [rarityFilters, cardTypeFilter],
  );

  const filteredEntries = useMemo<ResolvedEntry[]>(() => {
    return resolvedEntries.filter((resolved) => {
      if (!rarityFilters[resolved.card.rarity]) return false;
      if (
        cardTypeFilter === "Characters" &&
        resolved.card.cardType !== "Character"
      )
        return false;
      if (
        cardTypeFilter === "Events" &&
        resolved.card.cardType !== "Event"
      )
        return false;
      return true;
    });
  }, [resolvedEntries, rarityFilters, cardTypeFilter]);

  const sortedEntries = useMemo<ResolvedEntry[]>(() => {
    const sorted = [...filteredEntries];
    sorted.sort((a, b) => {
      let cmp = 0;
      switch (sortCriteria) {
        case "acquisitionOrder":
          cmp = a.index - b.index;
          break;
        case "energyCost":
          cmp = (a.card.energyCost ?? 0) - (b.card.energyCost ?? 0);
          break;
        case "name":
          cmp = a.card.name.localeCompare(b.card.name);
          break;
        case "rarity":
          cmp = RARITY_ORDER[a.card.rarity] - RARITY_ORDER[b.card.rarity];
          break;
        case "cardType":
          cmp = a.card.cardType.localeCompare(b.card.cardType);
          break;
      }
      return sortAscending ? cmp : -cmp;
    });
    return sorted;
  }, [filteredEntries, sortCriteria, sortAscending]);

  const toggleRarity = useCallback((rarity: Rarity) => {
    setRarityFilters((prev) => ({ ...prev, [rarity]: !prev[rarity] }));
  }, []);

  const handleCardClick = useCallback((card: CardData) => {
    setOverlayCard(card);
  }, []);

  const handleCloseOverlay = useCallback(() => {
    setOverlayCard(null);
  }, []);

  const closeSortDropdown = useCallback(() => {
    setShowSortDropdown(false);
  }, []);

  useEffect(() => {
    if (!showSortDropdown) return undefined;
    function handleClick() {
      closeSortDropdown();
    }
    window.addEventListener("click", handleClick);
    return () => {
      window.removeEventListener("click", handleClick);
    };
  }, [showSortDropdown, closeSortDropdown]);

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          key="deck-viewer-backdrop"
          className="fixed inset-0 z-[60] flex flex-col"
          style={{ backgroundColor: "rgba(5, 2, 10, 0.92)" }}
          initial={{ opacity: 0, y: 40 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 40 }}
          transition={{ duration: 0.3 }}
        >
          {/* Header */}
          <div
            className="flex items-center justify-between px-4 py-3 md:px-6"
            style={{
              borderBottom: "1px solid rgba(124, 58, 237, 0.3)",
              background:
                "linear-gradient(180deg, rgba(10, 6, 18, 0.95) 0%, rgba(10, 6, 18, 0.8) 100%)",
            }}
          >
            <h2 className="text-lg font-bold md:text-xl" style={{ color: "#e2e8f0" }}>
              Deck{" "}
              <span className="text-sm font-normal opacity-60 md:text-base">
                ({String(sortedEntries.length)}
                {sortedEntries.length !== resolvedEntries.length
                  ? ` / ${String(resolvedEntries.length)}`
                  : ""}{" "}
                cards)
              </span>
            </h2>
            <button
              className="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full text-lg transition-colors"
              style={{
                background: "rgba(255, 255, 255, 0.1)",
                color: "#e2e8f0",
              }}
              onClick={handleClose}
              aria-label="Close deck viewer"
            >
              {"\u2715"}
            </button>
          </div>

          {/* Deck summary */}
          {deckSummary.total > 0 && (
            <div
              className="px-4 py-2 md:px-6"
              style={{
                borderBottom: "1px solid rgba(124, 58, 237, 0.15)",
                background: "rgba(10, 6, 18, 0.5)",
              }}
            >
              <div
                className="mb-2 grid gap-2 md:grid-cols-3"
                style={{ color: "#e2e8f0" }}
              >
                <SummaryBadge
                  label="Characters"
                  value={String(deckSummary.characterCount)}
                />
                <SummaryBadge
                  label="Events"
                  value={String(deckSummary.eventCount)}
                />
                <SummaryBadge
                  label="Avg Cost"
                  value={
                    deckSummary.averageEnergyCost === null
                      ? "--"
                      : deckSummary.averageEnergyCost.toFixed(1)
                  }
                />
              </div>

              <div
                className="mb-2 flex h-2 overflow-hidden rounded-full"
                style={{ background: "rgba(255, 255, 255, 0.05)" }}
              >
                {deckSummary.rarities
                  .filter((rarity) => rarity.count > 0)
                  .map((rarity) => (
                    <div
                      key={rarity.rarity}
                      style={{
                        width: `${String(rarity.percentage)}%`,
                        background: RARITY_COLORS[rarity.rarity],
                        opacity: 0.85,
                      }}
                      title={`${rarity.rarity}: ${String(rarity.count)} (${String(rarity.percentage)}%)`}
                    />
                  ))}
              </div>

              <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
                {deckSummary.rarities.map((rarity) => (
                  <div
                    key={rarity.rarity}
                    className="flex items-center gap-1"
                    style={{
                      opacity: rarity.count > 0 ? 1 : 0.35,
                    }}
                  >
                    <span
                      className="h-3 w-3 rounded-full"
                      style={{
                        background: RARITY_COLORS[rarity.rarity],
                        boxShadow:
                          rarity.count > 0
                            ? `0 0 8px ${RARITY_COLORS[rarity.rarity]}55`
                            : "none",
                      }}
                    />
                    <span
                      className="text-[11px] font-medium"
                      style={{
                        color:
                          rarity.count > 0
                            ? RARITY_COLORS[rarity.rarity]
                            : "#6b7280",
                      }}
                    >
                      {rarity.rarity}
                    </span>
                    <span
                      className="text-[11px]"
                      style={{
                        color: rarity.count > 0 ? "#e2e8f0" : "#4b5563",
                      }}
                    >
                      {String(rarity.count)}
                    </span>
                    {rarity.count > 0 && (
                      <span
                        className="text-[10px]"
                        style={{ color: "#9ca3af" }}
                      >
                        ({String(rarity.percentage)}%)
                      </span>
                    )}
                  </div>
                ))}
                {hasActiveFilters && (
                  <span
                    className="ml-auto text-[10px] italic"
                    style={{ color: "#6b7280" }}
                  >
                    (showing full deck)
                  </span>
                )}
              </div>
            </div>
          )}

          {/* Controls row */}
          <div
            className="flex flex-wrap items-center gap-2 px-4 py-2 md:px-6"
            style={{
              borderBottom: "1px solid rgba(124, 58, 237, 0.15)",
              background: "rgba(10, 6, 18, 0.6)",
            }}
          >
            {/* Rarity filter toggles */}
            <div className="flex flex-wrap items-center gap-1">
              <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">
                Rarity
              </span>
              {ALL_RARITIES.map((rarity) => (
                <button
                  key={rarity}
                  className="flex cursor-pointer items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
                  style={{
                    background: rarityFilters[rarity]
                      ? `${RARITY_COLORS[rarity]}25`
                      : "rgba(255, 255, 255, 0.03)",
                    border: `1px solid ${rarityFilters[rarity] ? `${RARITY_COLORS[rarity]}60` : "rgba(255, 255, 255, 0.1)"}`,
                    color: rarityFilters[rarity]
                      ? RARITY_COLORS[rarity]
                      : "#6b7280",
                    opacity: rarityFilters[rarity] ? 1 : 0.5,
                  }}
                  onClick={() => {
                    toggleRarity(rarity);
                  }}
                >
                  <span
                    className="h-2.5 w-2.5 rounded-full"
                    style={{
                      background: RARITY_COLORS[rarity],
                      opacity: rarityFilters[rarity] ? 1 : 0.4,
                    }}
                  />
                  <span className="hidden sm:inline">{rarity}</span>
                </button>
              ))}
            </div>

            {/* Divider */}
            <div
              className="mx-1 hidden h-5 md:block"
              style={{ borderLeft: "1px solid rgba(255, 255, 255, 0.1)" }}
            />

            {/* Card type filter */}
            <div className="flex items-center gap-1">
              <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">
                Type
              </span>
              {(["All", "Characters", "Events"] as const).map((filter) => (
                <button
                  key={filter}
                  className="cursor-pointer rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
                  style={{
                    background:
                      cardTypeFilter === filter
                        ? "rgba(168, 85, 247, 0.25)"
                        : "rgba(255, 255, 255, 0.03)",
                    border: `1px solid ${cardTypeFilter === filter ? "rgba(168, 85, 247, 0.5)" : "rgba(255, 255, 255, 0.1)"}`,
                    color: cardTypeFilter === filter ? "#c084fc" : "#6b7280",
                  }}
                  onClick={() => {
                    setCardTypeFilter(filter);
                  }}
                >
                  {filter}
                </button>
              ))}
            </div>

            {/* Divider */}
            <div
              className="mx-1 hidden h-5 md:block"
              style={{ borderLeft: "1px solid rgba(255, 255, 255, 0.1)" }}
            />

            {/* Sort controls */}
            <div className="flex items-center gap-1">
              <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">
                Sort
              </span>
              <div className="relative">
                <button
                  className="flex cursor-pointer items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
                  style={{
                    background: "rgba(255, 255, 255, 0.05)",
                    border: "1px solid rgba(255, 255, 255, 0.15)",
                    color: "#e2e8f0",
                  }}
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowSortDropdown((prev) => !prev);
                  }}
                >
                  {SORT_LABELS[sortCriteria]}
                  <span className="opacity-50">{"\u25BE"}</span>
                </button>
                {showSortDropdown && (
                  <div
                    className="absolute top-full left-0 z-10 mt-1 rounded-lg py-1 shadow-xl"
                    style={{
                      background: "#1a1025",
                      border: "1px solid rgba(124, 58, 237, 0.3)",
                      minWidth: "160px",
                    }}
                  >
                    {(Object.keys(SORT_LABELS) as SortCriteria[]).map(
                      (criteria) => (
                        <button
                          key={criteria}
                          className="block w-full cursor-pointer px-3 py-1.5 text-left text-xs transition-colors"
                          style={{
                            color:
                              sortCriteria === criteria
                                ? "#c084fc"
                                : "#e2e8f0",
                            background:
                              sortCriteria === criteria
                                ? "rgba(168, 85, 247, 0.15)"
                                : "transparent",
                          }}
                          onClick={(e) => {
                            e.stopPropagation();
                            setSortCriteria(criteria);
                            setShowSortDropdown(false);
                          }}
                        >
                          {SORT_LABELS[criteria]}
                        </button>
                      ),
                    )}
                  </div>
                )}
              </div>
              <button
                className="flex cursor-pointer items-center rounded-full px-1.5 py-0.5 text-[11px] transition-all"
                style={{
                  background: "rgba(255, 255, 255, 0.05)",
                  border: "1px solid rgba(255, 255, 255, 0.15)",
                  color: "#e2e8f0",
                }}
                onClick={() => {
                  setSortAscending((prev) => !prev);
                }}
                aria-label={
                  sortAscending ? "Sort descending" : "Sort ascending"
                }
              >
                {sortAscending ? "\u2191" : "\u2193"}
              </button>
            </div>
          </div>

          {/* Main content area */}
          <div className="flex min-h-0 flex-1">
            {/* Card grid */}
            <div className="flex-1 overflow-y-auto px-4 py-3 md:px-6">
              {sortedEntries.length === 0 ? (
                <div className="flex h-full items-center justify-center">
                  <p className="text-sm opacity-40">
                    {resolvedEntries.length === 0
                      ? "No cards in deck yet."
                      : "No cards match the current filters."}
                  </p>
                </div>
              ) : (
                <div className="grid grid-cols-3 gap-3 xl:grid-cols-5">
                  {sortedEntries.map((resolved) => (
                    <div
                      key={resolved.entry.entryId}
                      className="relative"
                    >
                      {/* Transfiguration indicator */}
                      {resolved.entry.transfiguration !== null && (
                        <div
                          className="absolute -top-1 -right-1 z-10 rounded-full px-1.5 py-0.5 text-[9px] font-bold shadow-md"
                          style={{
                            background:
                              TRANSFIGURATION_COLORS[
                                resolved.entry.transfiguration
                              ],
                            color: "#fff",
                            boxShadow: `0 0 6px ${TRANSFIGURATION_COLORS[resolved.entry.transfiguration]}80`,
                          }}
                        >
                          {resolved.entry.transfiguration}
                        </div>
                      )}
                      {/* Bane indicator */}
                      {resolved.entry.isBane && (
                        <div
                          className="absolute -top-1 -left-1 z-10 flex h-5 w-5 items-center justify-center rounded-full text-[10px] shadow-md"
                          style={{
                            background: "#7f1d1d",
                            border: "1px solid #ef4444",
                            color: "#fca5a5",
                          }}
                          title="Bane"
                        >
                          {"\u2620"}
                        </div>
                      )}
                      <div
                        style={
                          resolved.entry.transfiguration !== null
                            ? {
                                boxShadow: `0 0 8px ${TRANSFIGURATION_COLORS[resolved.entry.transfiguration]}40`,
                                borderRadius: "0.5rem",
                              }
                            : resolved.entry.isBane
                              ? {
                                  boxShadow:
                                    "0 0 8px rgba(239, 68, 68, 0.3)",
                                  borderRadius: "0.5rem",
                                }
                              : undefined
                        }
                      >
                        <CardDisplay
                          card={resolved.card}
                          onClick={() => {
                            handleCardClick(resolved.card);
                          }}
                        />
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Sidebar */}
            <div
              className="hidden w-64 shrink-0 overflow-y-auto border-l px-4 py-3 lg:block"
              style={{
                borderColor: "rgba(124, 58, 237, 0.2)",
                background: "rgba(10, 6, 18, 0.4)",
              }}
            >
              {/* Dreamcaller section */}
              <div className="mb-4">
                <h3
                  className="mb-2 text-xs font-bold uppercase tracking-wider"
                  style={{ color: "#a855f7" }}
                >
                  Dreamcaller
                </h3>
                {state.dreamcaller !== null ? (
                  <div
                    className="rounded-lg p-3"
                    style={{
                      background: "rgba(124, 58, 237, 0.1)",
                      border: "1px solid rgba(124, 58, 237, 0.2)",
                    }}
                  >
                    <div className="flex items-center gap-2">
                      <img
                        src={tideIconUrl(state.dreamcaller.accentTide)}
                        alt={state.dreamcaller.accentTide}
                        className="h-5 w-5 rounded-full"
                        style={{
                          border: `1px solid ${TIDE_COLORS[state.dreamcaller.accentTide]}`,
                        }}
                      />
                      <span
                        className="text-sm font-bold"
                        style={{
                          color:
                            TIDE_COLORS[state.dreamcaller.accentTide],
                        }}
                      >
                        {state.dreamcaller.name}
                      </span>
                    </div>
                    <p
                      className="mt-2 text-[11px] leading-relaxed opacity-70"
                      style={{ color: "#e2e8f0" }}
                    >
                      {state.dreamcaller.renderedText}
                    </p>
                  </div>
                ) : (
                  <p className="text-xs opacity-40">
                    No dreamcaller selected.
                  </p>
                )}
              </div>

              {/* Dreamsigns section */}
              <div className="mb-4">
                <h3
                  className="mb-2 text-xs font-bold uppercase tracking-wider"
                  style={{ color: "#a855f7" }}
                >
                  Dreamsigns ({String(state.dreamsigns.length)}/12)
                </h3>
                {state.dreamsigns.length === 0 ? (
                  <p className="text-xs opacity-40">
                    No dreamsigns acquired.
                  </p>
                ) : (
                  <div className="flex flex-col gap-2">
                    {state.dreamsigns.map((sign, i) => (
                      <div
                        key={`${sign.name}-${String(i)}`}
                        className="rounded-lg p-2"
                        style={{
                          background: sign.isBane
                            ? "rgba(239, 68, 68, 0.1)"
                            : "rgba(124, 58, 237, 0.08)",
                          border: `1px solid ${sign.isBane ? "rgba(239, 68, 68, 0.25)" : "rgba(124, 58, 237, 0.15)"}`,
                        }}
                      >
                        <div className="flex items-center gap-1.5">
                          <img
                            src={tideIconUrl(sign.tide)}
                            alt={sign.tide}
                            className="h-3.5 w-3.5 rounded-full"
                            style={{
                              border: `1px solid ${TIDE_COLORS[sign.tide]}`,
                            }}
                          />
                          <span
                            className="text-[11px] font-bold"
                            style={{
                              color: TIDE_COLORS[sign.tide],
                            }}
                          >
                            {sign.name}
                          </span>
                          {sign.isBane && (
                            <span
                              className="ml-auto text-[10px]"
                              style={{ color: "#ef4444" }}
                              title="Bane"
                            >
                              {"\u2620"}
                            </span>
                          )}
                        </div>
                        <p
                          className="mt-1 text-[10px] leading-snug opacity-60"
                          style={{ color: "#e2e8f0" }}
                        >
                          {sign.effectDescription}
                        </p>
                      </div>
                    ))}
                  </div>
                )}
              </div>

            </div>
          </div>

          {/* Mobile sidebar as tabs at bottom */}
          <MobileSidebar
            dreamcaller={state.dreamcaller}
            dreamsigns={state.dreamsigns}
          />

          {/* Card overlay */}
          <CardOverlay card={overlayCard} onClose={handleCloseOverlay} />
        </motion.div>
      )}
    </AnimatePresence>
  );
}

function SummaryBadge({ label, value }: { label: string; value: string }) {
  return (
    <div
      className="rounded-lg px-3 py-2"
      style={{
        background: "rgba(255, 255, 255, 0.03)",
        border: "1px solid rgba(255, 255, 255, 0.08)",
      }}
    >
      <span className="text-[10px] uppercase tracking-wider opacity-45">
        {label}
      </span>
      <div className="text-sm font-bold" style={{ color: "#e2e8f0" }}>
        {value}
      </div>
    </div>
  );
}

/** Mobile sidebar shown only on smaller screens as a collapsible section. */
function MobileSidebar({
  dreamcaller,
  dreamsigns,
}: {
  dreamcaller: QuestState["dreamcaller"];
  dreamsigns: QuestState["dreamsigns"];
}) {
  const [activeTab, setActiveTab] = useState<
    "dreamcaller" | "dreamsigns" | null
  >(null);

  const toggleTab = useCallback(
    (tab: "dreamcaller" | "dreamsigns") => {
      setActiveTab((prev) => (prev === tab ? null : tab));
    },
    [],
  );

  return (
    <div
      className="lg:hidden"
      style={{
        borderTop: "1px solid rgba(124, 58, 237, 0.2)",
        background: "rgba(10, 6, 18, 0.9)",
      }}
    >
      {/* Tab buttons */}
      <div className="flex">
        <button
          className="flex-1 cursor-pointer px-2 py-1.5 text-[10px] font-medium transition-colors"
          style={{
            color: activeTab === "dreamcaller" ? "#c084fc" : "#6b7280",
            background:
              activeTab === "dreamcaller"
                ? "rgba(168, 85, 247, 0.15)"
                : "transparent",
            borderBottom: `2px solid ${activeTab === "dreamcaller" ? "#a855f7" : "transparent"}`,
          }}
          onClick={() => {
            toggleTab("dreamcaller");
          }}
        >
          Dreamcaller
        </button>
        <button
          className="flex-1 cursor-pointer px-2 py-1.5 text-[10px] font-medium transition-colors"
          style={{
            color: activeTab === "dreamsigns" ? "#c084fc" : "#6b7280",
            background:
              activeTab === "dreamsigns"
                ? "rgba(168, 85, 247, 0.15)"
                : "transparent",
            borderBottom: `2px solid ${activeTab === "dreamsigns" ? "#a855f7" : "transparent"}`,
          }}
          onClick={() => {
            toggleTab("dreamsigns");
          }}
        >
          Signs ({String(dreamsigns.length)}/12)
        </button>
      </div>

      {/* Tab content */}
      <AnimatePresence>
        {activeTab !== null && (
          <motion.div
            key={activeTab}
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="overflow-hidden px-4 py-2"
          >
            {activeTab === "dreamcaller" && (
              <div>
                {dreamcaller !== null ? (
                  <div className="flex items-start gap-2">
                    <img
                      src={tideIconUrl(dreamcaller.accentTide)}
                      alt={dreamcaller.accentTide}
                      className="mt-0.5 h-5 w-5 rounded-full"
                      style={{
                        border: `1px solid ${TIDE_COLORS[dreamcaller.accentTide]}`,
                      }}
                    />
                    <div>
                      <span
                        className="text-xs font-bold"
                        style={{
                          color: TIDE_COLORS[dreamcaller.accentTide],
                        }}
                      >
                        {dreamcaller.name}
                      </span>
                      <p className="mt-0.5 text-[10px] opacity-60">
                        {dreamcaller.renderedText}
                      </p>
                    </div>
                  </div>
                ) : (
                  <p className="text-xs opacity-40">
                    No dreamcaller selected.
                  </p>
                )}
              </div>
            )}
            {activeTab === "dreamsigns" && (
              <div className="flex flex-col gap-1.5">
                {dreamsigns.length === 0 ? (
                  <p className="text-xs opacity-40">
                    No dreamsigns acquired.
                  </p>
                ) : (
                  dreamsigns.map((sign, i) => (
                    <div
                      key={`mobile-${sign.name}-${String(i)}`}
                      className="rounded-lg p-2"
                      style={{
                        background: sign.isBane
                          ? "rgba(239, 68, 68, 0.1)"
                          : "rgba(124, 58, 237, 0.08)",
                        border: `1px solid ${sign.isBane ? "rgba(239, 68, 68, 0.25)" : "rgba(124, 58, 237, 0.15)"}`,
                      }}
                    >
                      <div className="flex items-center gap-1.5">
                        <img
                          src={tideIconUrl(sign.tide)}
                          alt={sign.tide}
                          className="h-3 w-3 rounded-full"
                        />
                        <span
                          className="text-[10px] font-medium"
                          style={{ color: TIDE_COLORS[sign.tide] }}
                        >
                          {sign.name}
                        </span>
                        {sign.isBane && (
                          <span
                            className="ml-auto text-[9px]"
                            style={{ color: "#ef4444" }}
                          >
                            {"\u2620"}
                          </span>
                        )}
                      </div>
                      <p
                        className="mt-1 text-[10px] leading-snug opacity-60"
                        style={{ color: "#e2e8f0" }}
                      >
                        {sign.effectDescription}
                      </p>
                    </div>
                  ))
                )}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
