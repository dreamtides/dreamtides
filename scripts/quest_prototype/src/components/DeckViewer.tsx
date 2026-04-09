import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type {
  DeckEntry,
  QuestState,
} from "../types/quest";
import { useQuest } from "../state/quest-context";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { CardDisplay } from "./CardDisplay";
import { CardOverlay } from "./CardOverlay";
import { logEvent } from "../logging";
import { TRANSFIGURATION_COLORS } from "../transfiguration/transfiguration-logic";
import { computeTideDistribution } from "./tide-distribution";

/** All tides including Neutral, used for filter toggles. */
const ALL_TIDES: readonly Tide[] = [...NAMED_TIDES, "Neutral"] as const;

/** Sort criteria options. */
type SortCriteria =
  | "acquisitionOrder"
  | "energyCost"
  | "name"
  | "tide"
  | "cardType";

/** Labels for sort criteria. */
const SORT_LABELS: Readonly<Record<SortCriteria, string>> = {
  acquisitionOrder: "Acquisition Order",
  energyCost: "Energy Cost",
  name: "Name",
  tide: "Tide",
  cardType: "Card Type",
};

/** Card type filter options. */
type CardTypeFilter = "All" | "Characters" | "Events";

/** Tide ordering for sort-by-tide. */
const TIDE_ORDER: Readonly<Record<Tide, number>> = {
  Bloom: 0,
  Arc: 1,
  Ignite: 2,
  Pact: 3,
  Umbra: 4,
  Rime: 5,
  Surge: 6,
  Neutral: 7,
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
 * sorting, and a sidebar for dreamcaller, dreamsigns, and tide crystals.
 */
export function DeckViewer({
  isOpen,
  onClose,
  cardDatabase,
}: DeckViewerProps) {
  const { state } = useQuest();

  const [tideFilters, setTideFilters] = useState<Record<Tide, boolean>>(
    () => {
      const filters: Partial<Record<Tide, boolean>> = {};
      for (const tide of ALL_TIDES) {
        filters[tide] = true;
      }
      return filters as Record<Tide, boolean>;
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
        tideFilters: Object.fromEntries(
          Object.entries(tideFilters).filter(([_, v]) => !v),
        ),
        sortCriteria,
        sortAscending,
        cardTypeFilter,
      });
    }
    prevOpenRef.current = isOpen;
  }, [isOpen, state.deck.length, tideFilters, sortCriteria, sortAscending, cardTypeFilter]);

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

  const tideDistribution = useMemo(
    () => computeTideDistribution(state.deck, cardDatabase),
    [state.deck, cardDatabase],
  );

  const activeFilterCount = useMemo(() => {
    const tideActive = ALL_TIDES.some((t) => !tideFilters[t]);
    const typeActive = cardTypeFilter !== "All";
    return tideActive || typeActive;
  }, [tideFilters, cardTypeFilter]);

  const filteredEntries = useMemo<ResolvedEntry[]>(() => {
    return resolvedEntries.filter((resolved) => {
      if (!tideFilters[resolved.card.tide]) return false;
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
  }, [resolvedEntries, tideFilters, cardTypeFilter]);

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
        case "tide":
          cmp = TIDE_ORDER[a.card.tide] - TIDE_ORDER[b.card.tide];
          break;
        case "cardType":
          cmp = a.card.cardType.localeCompare(b.card.cardType);
          break;
      }
      return sortAscending ? cmp : -cmp;
    });
    return sorted;
  }, [filteredEntries, sortCriteria, sortAscending]);

  const toggleTide = useCallback((tide: Tide) => {
    setTideFilters((prev) => ({ ...prev, [tide]: !prev[tide] }));
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

  const nonZeroCrystals = useMemo(() => {
    return Object.entries(state.tideCrystals).filter(
      ([_, count]) => count > 0,
    ) as Array<[Tide, number]>;
  }, [state.tideCrystals]);

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

          {/* Tide distribution bar */}
          {tideDistribution.total > 0 && (
            <div
              className="px-4 py-2 md:px-6"
              style={{
                borderBottom: "1px solid rgba(124, 58, 237, 0.15)",
                background: "rgba(10, 6, 18, 0.5)",
              }}
            >
              {/* Proportional bar */}
              <div
                className="mb-2 flex h-2 overflow-hidden rounded-full"
                style={{ background: "rgba(255, 255, 255, 0.05)" }}
              >
                {tideDistribution.tides
                  .filter((t) => t.count > 0)
                  .map((t) => (
                    <div
                      key={t.tide}
                      style={{
                        width: `${String(t.percentage)}%`,
                        background: TIDE_COLORS[t.tide],
                        opacity: 0.8,
                      }}
                      title={`${t.tide}: ${String(t.count)} (${String(t.percentage)}%)`}
                    />
                  ))}
              </div>

              {/* Tide badges row */}
              <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
                {tideDistribution.tides.map((t) => (
                  <div
                    key={t.tide}
                    className="flex items-center gap-1"
                    style={{
                      opacity: t.count > 0 ? 1 : 0.3,
                    }}
                  >
                    <img
                      src={tideIconUrl(t.tide)}
                      alt={t.tide}
                      className="h-4 w-4 rounded-full"
                      style={{
                        border: t.isDominant
                          ? `1.5px solid ${TIDE_COLORS[t.tide]}`
                          : "1px solid rgba(255, 255, 255, 0.15)",
                      }}
                    />
                    <span
                      className="text-[11px] font-medium"
                      style={{
                        color: t.count > 0 ? TIDE_COLORS[t.tide] : "#6b7280",
                      }}
                    >
                      {t.tide}
                    </span>
                    <span
                      className="text-[11px]"
                      style={{
                        color: t.count > 0 ? "#e2e8f0" : "#4b5563",
                        fontWeight: t.isDominant ? 700 : 400,
                      }}
                    >
                      {String(t.count)}
                    </span>
                    {t.count > 0 && (
                      <span
                        className="text-[10px]"
                        style={{ color: "#9ca3af" }}
                      >
                        ({String(t.percentage)}%)
                      </span>
                    )}
                  </div>
                ))}
                {activeFilterCount && (
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
            {/* Tide filter toggles */}
            <div className="flex flex-wrap items-center gap-1">
              <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">
                Tides
              </span>
              {ALL_TIDES.map((tide) => (
                <button
                  key={tide}
                  className="flex cursor-pointer items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
                  style={{
                    background: tideFilters[tide]
                      ? `${TIDE_COLORS[tide]}25`
                      : "rgba(255, 255, 255, 0.03)",
                    border: `1px solid ${tideFilters[tide] ? `${TIDE_COLORS[tide]}60` : "rgba(255, 255, 255, 0.1)"}`,
                    color: tideFilters[tide] ? TIDE_COLORS[tide] : "#6b7280",
                    opacity: tideFilters[tide] ? 1 : 0.5,
                  }}
                  onClick={() => {
                    toggleTide(tide);
                  }}
                >
                  <img
                    src={tideIconUrl(tide)}
                    alt={tide}
                    className="h-3 w-3 rounded-full"
                    style={{
                      opacity: tideFilters[tide] ? 1 : 0.4,
                    }}
                  />
                  <span className="hidden sm:inline">{tide}</span>
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
                        src={tideIconUrl(state.dreamcaller.tide)}
                        alt={state.dreamcaller.tide}
                        className="h-5 w-5 rounded-full"
                        style={{
                          border: `1px solid ${TIDE_COLORS[state.dreamcaller.tide]}`,
                        }}
                      />
                      <span
                        className="text-sm font-bold"
                        style={{
                          color:
                            TIDE_COLORS[state.dreamcaller.tide],
                        }}
                      >
                        {state.dreamcaller.name}
                      </span>
                    </div>
                    <p
                      className="mt-2 text-[11px] leading-relaxed opacity-70"
                      style={{ color: "#e2e8f0" }}
                    >
                      {state.dreamcaller.abilityDescription}
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

              {/* Tide Crystals section */}
              {nonZeroCrystals.length > 0 && (
                <div>
                  <h3
                    className="mb-2 text-xs font-bold uppercase tracking-wider"
                    style={{ color: "#a855f7" }}
                  >
                    Tide Crystals
                  </h3>
                  <div className="flex flex-col gap-1.5">
                    {nonZeroCrystals.map(([tide, count]) => (
                      <div
                        key={tide}
                        className="flex items-center gap-2"
                      >
                        <img
                          src={tideIconUrl(tide)}
                          alt={tide}
                          className="h-4 w-4 rounded-full"
                          style={{
                            border: `1px solid ${TIDE_COLORS[tide]}`,
                          }}
                        />
                        <span
                          className="text-xs font-medium"
                          style={{ color: TIDE_COLORS[tide] }}
                        >
                          {tide}
                        </span>
                        <span
                          className="ml-auto text-xs font-bold"
                          style={{ color: "#e2e8f0" }}
                        >
                          {String(count)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>

          {/* Mobile sidebar as tabs at bottom */}
          <MobileSidebar
            dreamcaller={state.dreamcaller}
            dreamsigns={state.dreamsigns}
            tideCrystals={nonZeroCrystals}
          />

          {/* Card overlay */}
          <CardOverlay card={overlayCard} onClose={handleCloseOverlay} />
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/** Mobile sidebar shown only on smaller screens as a collapsible section. */
function MobileSidebar({
  dreamcaller,
  dreamsigns,
  tideCrystals,
}: {
  dreamcaller: QuestState["dreamcaller"];
  dreamsigns: QuestState["dreamsigns"];
  tideCrystals: Array<[Tide, number]>;
}) {
  const [activeTab, setActiveTab] = useState<
    "dreamcaller" | "dreamsigns" | "crystals" | null
  >(null);

  const toggleTab = useCallback(
    (tab: "dreamcaller" | "dreamsigns" | "crystals") => {
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
        {tideCrystals.length > 0 && (
          <button
            className="flex-1 cursor-pointer px-2 py-1.5 text-[10px] font-medium transition-colors"
            style={{
              color: activeTab === "crystals" ? "#c084fc" : "#6b7280",
              background:
                activeTab === "crystals"
                  ? "rgba(168, 85, 247, 0.15)"
                  : "transparent",
              borderBottom: `2px solid ${activeTab === "crystals" ? "#a855f7" : "transparent"}`,
            }}
            onClick={() => {
              toggleTab("crystals");
            }}
          >
            Crystals
          </button>
        )}
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
                      src={tideIconUrl(dreamcaller.tide)}
                      alt={dreamcaller.tide}
                      className="mt-0.5 h-5 w-5 rounded-full"
                      style={{
                        border: `1px solid ${TIDE_COLORS[dreamcaller.tide]}`,
                      }}
                    />
                    <div>
                      <span
                        className="text-xs font-bold"
                        style={{
                          color: TIDE_COLORS[dreamcaller.tide],
                        }}
                      >
                        {dreamcaller.name}
                      </span>
                      <p className="mt-0.5 text-[10px] opacity-60">
                        {dreamcaller.abilityDescription}
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
            {activeTab === "crystals" && (
              <div className="flex flex-wrap gap-3">
                {tideCrystals.map(([tide, count]) => (
                  <div key={tide} className="flex items-center gap-1">
                    <img
                      src={tideIconUrl(tide)}
                      alt={tide}
                      className="h-3.5 w-3.5 rounded-full"
                    />
                    <span
                      className="text-[10px] font-bold"
                      style={{ color: TIDE_COLORS[tide] }}
                    >
                      {tide}: {String(count)}
                    </span>
                  </div>
                ))}
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}
