import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { NAMED_TIDES, TIDE_COLORS, tideIconUrl, cardImageUrl } from "../data/card-database";
import { CardDisplay } from "./CardDisplay";
import { CardOverlay } from "./CardOverlay";
import { DeckViewerContent } from "./DeckViewer";
import { VisualDeckEditor } from "./VisualDeckEditor";
import { TRANSFIGURATION_COLORS } from "../transfiguration/transfiguration-logic";

/** All tides including Neutral, used for filter toggles. */
const ALL_TIDES: readonly Tide[] = [...NAMED_TIDES, "Neutral"] as const;

/** Sort criteria for the pool panel. */
type SortCriteria = "energyCost" | "name" | "tide" | "cardType";

const SORT_LABELS: Readonly<Record<SortCriteria, string>> = {
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

/** A resolved pool entry with card data. */
interface ResolvedEntry {
  entry: DeckEntry;
  card: CardData;
}

/** A grouped pool entry combining duplicates. */
interface PoolGroup {
  cardNumber: number;
  card: CardData;
  transfiguration: DeckEntry["transfiguration"];
  isBane: boolean;
  entryIds: string[];
  count: number;
}

/** A grouped deck row for the compact list. */
interface DeckRow {
  cardNumber: number;
  card: CardData;
  transfiguration: DeckEntry["transfiguration"];
  isBane: boolean;
  entryIds: string[];
  count: number;
}

/** Props for the DeckEditor component. */
interface DeckEditorProps {
  isOpen: boolean;
  onClose: () => void;
  cardDatabase: Map<number, CardData>;
}

/** Full-screen deck editor with pool grid and compact deck list. */
export function DeckEditor({
  isOpen,
  onClose,
  cardDatabase,
}: DeckEditorProps) {
  const { state, mutations } = useQuest();
  const config = useQuestConfig();
  const [validationMessage, setValidationMessage] = useState<string | null>(null);

  const [tideFilters, setTideFilters] = useState<Record<Tide, boolean>>(() => {
    const filters: Partial<Record<Tide, boolean>> = {};
    for (const tide of ALL_TIDES) {
      filters[tide] = true;
    }
    return filters as Record<Tide, boolean>;
  });
  const [cardTypeFilter, setCardTypeFilter] = useState<CardTypeFilter>("All");
  const [sortCriteria, setSortCriteria] = useState<SortCriteria>("energyCost");
  const [sortAscending, setSortAscending] = useState(true);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);
  const [showSortDropdown, setShowSortDropdown] = useState(false);
  const [hoverPreview, setHoverPreview] = useState<{ card: CardData; top: number } | null>(null);
  const hoverTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const deckScrollRef = useRef<HTMLDivElement>(null);
  const [viewMode, setViewMode] = useState<"edit" | "visual" | "view">("edit");

  const handleDone = useCallback(() => {
    setValidationMessage(null);
    onClose();
  }, [onClose]);

  useEffect(() => {
    if (!isOpen) return undefined;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        if (overlayCard !== null) return;
        handleDone();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, overlayCard, handleDone]);

  // Clear hover preview on scroll or when editor closes
  useEffect(() => {
    if (!isOpen) {
      setHoverPreview(null);
      return undefined;
    }
    const scrollEl = deckScrollRef.current;
    if (!scrollEl) return undefined;
    function handleScroll() {
      setHoverPreview(null);
    }
    scrollEl.addEventListener("scroll", handleScroll);
    return () => {
      scrollEl.removeEventListener("scroll", handleScroll);
    };
  }, [isOpen]);

  useEffect(() => {
    if (!showSortDropdown) return undefined;
    function handleClick() {
      setShowSortDropdown(false);
    }
    window.addEventListener("click", handleClick);
    return () => {
      window.removeEventListener("click", handleClick);
    };
  }, [showSortDropdown]);

  // Resolve pool entries
  const resolvedPool = useMemo<ResolvedEntry[]>(() => {
    return state.pool
      .map((entry) => {
        const card = cardDatabase.get(entry.cardNumber);
        if (!card) return null;
        return { entry, card };
      })
      .filter((e): e is ResolvedEntry => e !== null);
  }, [state.pool, cardDatabase]);

  // Filter pool entries
  const filteredPool = useMemo<ResolvedEntry[]>(() => {
    return resolvedPool.filter((resolved) => {
      if (!tideFilters[resolved.card.tide]) return false;
      if (cardTypeFilter === "Characters" && resolved.card.cardType !== "Character") return false;
      if (cardTypeFilter === "Events" && resolved.card.cardType !== "Event") return false;
      return true;
    });
  }, [resolvedPool, tideFilters, cardTypeFilter]);

  // Sort pool entries
  const sortedPool = useMemo<ResolvedEntry[]>(() => {
    const sorted = [...filteredPool];
    sorted.sort((a, b) => {
      let cmp = 0;
      switch (sortCriteria) {
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
      if (cmp === 0) cmp = (a.card.energyCost ?? 0) - (b.card.energyCost ?? 0);
      if (cmp === 0) cmp = a.card.name.localeCompare(b.card.name);
      return sortAscending ? cmp : -cmp;
    });
    return sorted;
  }, [filteredPool, sortCriteria, sortAscending]);

  // Group sorted pool entries by card identity for duplicate badges
  const poolGroups = useMemo<PoolGroup[]>(() => {
    const groups = new Map<string, PoolGroup>();
    const order: string[] = [];
    for (const resolved of sortedPool) {
      const key = `${String(resolved.entry.cardNumber)}-${resolved.entry.transfiguration ?? "none"}-${String(resolved.entry.isBane)}`;
      const existing = groups.get(key);
      if (existing) {
        existing.entryIds.push(resolved.entry.entryId);
        existing.count += 1;
      } else {
        order.push(key);
        groups.set(key, {
          cardNumber: resolved.entry.cardNumber,
          card: resolved.card,
          transfiguration: resolved.entry.transfiguration,
          isBane: resolved.entry.isBane,
          entryIds: [resolved.entry.entryId],
          count: 1,
        });
      }
    }
    return order.map((key) => groups.get(key)!);
  }, [sortedPool]);

  // Group deck entries into compact rows
  const deckRows = useMemo<DeckRow[]>(() => {
    const groups = new Map<string, DeckRow>();
    for (const entry of state.deck) {
      const card = cardDatabase.get(entry.cardNumber);
      if (!card) continue;
      const key = `${String(entry.cardNumber)}-${entry.transfiguration ?? "none"}-${String(entry.isBane)}`;
      const existing = groups.get(key);
      if (existing) {
        existing.entryIds.push(entry.entryId);
        existing.count += 1;
      } else {
        groups.set(key, {
          cardNumber: entry.cardNumber,
          card,
          transfiguration: entry.transfiguration,
          isBane: entry.isBane,
          entryIds: [entry.entryId],
          count: 1,
        });
      }
    }
    const rows = Array.from(groups.values());
    rows.sort((a, b) => {
      const costCmp = (a.card.energyCost ?? 0) - (b.card.energyCost ?? 0);
      if (costCmp !== 0) return costCmp;
      return a.card.name.localeCompare(b.card.name);
    });
    return rows;
  }, [state.deck, cardDatabase]);

  // Group deck rows by energy cost for dividers
  const deckRowsByEnergy = useMemo(() => {
    const groups: Array<{ cost: number; rows: DeckRow[] }> = [];
    let currentCost: number | null = null;
    for (const row of deckRows) {
      const cost = row.card.energyCost ?? 0;
      if (cost !== currentCost) {
        groups.push({ cost, rows: [row] });
        currentCost = cost;
      } else {
        groups[groups.length - 1].rows.push(row);
      }
    }
    return groups;
  }, [deckRows]);

  // Count copies of each cardNumber currently in the deck
  const deckCopyCounts = useMemo(() => {
    const counts = new Map<number, number>();
    for (const entry of state.deck) {
      counts.set(entry.cardNumber, (counts.get(entry.cardNumber) ?? 0) + 1);
    }
    return counts;
  }, [state.deck]);

  // Count bane vs chosen cards in deck
  const baneCount = useMemo(() => state.deck.filter((e) => e.isBane).length, [state.deck]);
  const chosenCount = state.deck.length - baneCount;

  // Deck size color coding
  const deckSizeColor =
    state.deck.length < config.minimumDeckSize || state.deck.length > config.maximumDeckSize
      ? "#ef4444"
      : "#10b981";

  const toggleTide = useCallback((tide: Tide) => {
    setTideFilters((prev) => ({ ...prev, [tide]: !prev[tide] }));
  }, []);

  const handlePoolGroupClick = useCallback(
    (group: PoolGroup) => {
      const currentCopies = deckCopyCounts.get(group.cardNumber) ?? 0;
      if (currentCopies >= config.maxCopies) return;
      const entryId = group.entryIds[group.entryIds.length - 1];
      mutations.moveToDeck(entryId);
    },
    [mutations, deckCopyCounts, config.maxCopies],
  );

  const handleDeckRowClick = useCallback(
    (row: DeckRow) => {
      if (row.isBane) return;
      const entryId = row.entryIds[row.entryIds.length - 1];
      mutations.moveToPool(entryId);
    },
    [mutations],
  );

  const handleCardOverlay = useCallback((card: CardData) => {
    setOverlayCard(card);
  }, []);

  const handleCloseOverlay = useCallback(() => {
    setOverlayCard(null);
  }, []);

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          key="deck-editor-backdrop"
          className="fixed inset-0 z-[60] flex flex-col"
          style={{ backgroundColor: "rgba(5, 2, 10, 0.95)" }}
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
            <div className="flex items-center gap-3">
              <h2 className="text-lg font-bold md:text-xl" style={{ color: "#e2e8f0" }}>
                Deck Editor
              </h2>
              <span
                className="rounded-full px-3 py-0.5 text-xs font-bold"
                style={{
                  background: "rgba(251, 191, 36, 0.15)",
                  border: "1px solid rgba(251, 191, 36, 0.3)",
                  color: deckSizeColor,
                }}
              >
                Deck: {String(state.deck.length)} / {String(config.minimumDeckSize)}-{String(config.maximumDeckSize)}
              </span>
              <span className="text-[11px] opacity-60" style={{ color: "#e2e8f0" }}>
                {String(state.deck.length)} cards ({String(baneCount)} bane, {String(chosenCount)} chosen)
              </span>
            </div>
            <div className="flex items-center gap-3">
              {validationMessage !== null && (
                <span className="text-xs font-medium" style={{ color: "#ef4444" }}>
                  {validationMessage}
                </span>
              )}
              <div
                className="flex overflow-hidden rounded-lg"
                style={{ border: "1px solid rgba(124, 58, 237, 0.4)" }}
              >
                <button
                  className="cursor-pointer px-3 py-1.5 text-xs font-medium transition-colors"
                  style={{
                    background: viewMode === "edit" ? "rgba(124, 58, 237, 0.3)" : "transparent",
                    color: viewMode === "edit" ? "#c084fc" : "#6b7280",
                  }}
                  onClick={() => { setViewMode("edit"); }}
                >
                  List
                </button>
                <button
                  className="cursor-pointer px-3 py-1.5 text-xs font-medium transition-colors"
                  style={{
                    background: viewMode === "visual" ? "rgba(124, 58, 237, 0.3)" : "transparent",
                    color: viewMode === "visual" ? "#c084fc" : "#6b7280",
                    borderLeft: "1px solid rgba(124, 58, 237, 0.4)",
                  }}
                  onClick={() => { setViewMode("visual"); }}
                >
                  Visual
                </button>
                <button
                  className="cursor-pointer px-3 py-1.5 text-xs font-medium transition-colors"
                  style={{
                    background: viewMode === "view" ? "rgba(124, 58, 237, 0.3)" : "transparent",
                    color: viewMode === "view" ? "#c084fc" : "#6b7280",
                    borderLeft: "1px solid rgba(124, 58, 237, 0.4)",
                  }}
                  onClick={() => { setViewMode("view"); }}
                >
                  View
                </button>
              </div>
              <button
                className="cursor-pointer rounded-lg px-5 py-2 text-sm font-semibold transition-colors"
                style={{
                  background: "rgba(251, 191, 36, 0.2)",
                  border: "1px solid rgba(251, 191, 36, 0.5)",
                  color: "#fbbf24",
                  textShadow: "0 0 8px rgba(251, 191, 36, 0.3)",
                }}
                onClick={handleDone}
              >
                Done
              </button>
            </div>
          </div>

          {/* Main content */}
          {viewMode === "view" ? (
            <DeckViewerContent cardDatabase={cardDatabase} />
          ) : viewMode === "visual" ? (
            <VisualDeckEditor cardDatabase={cardDatabase} />
          ) : (
          <div className="flex min-h-0 flex-1">
            {/* Left panel: Pool */}
            <div
              className="flex flex-1 flex-col"
              style={{ borderRight: "1px solid rgba(124, 58, 237, 0.4)" }}
            >
              {/* Pool header + filters (merged into one compact bar) */}
              <div
                className="px-4 py-2 md:px-6"
                style={{
                  borderBottom: "1px solid rgba(124, 58, 237, 0.15)",
                  background: "rgba(10, 6, 18, 0.6)",
                }}
              >
                {/* Row 1: Pool label + tide filters */}
                <div className="mb-1.5 flex flex-wrap items-center gap-1.5">
                  <span className="mr-1 text-sm font-bold" style={{ color: "#a855f7" }}>
                    Pool{" "}
                    <span className="font-normal opacity-60">
                      ({String(sortedPool.length)}
                      {sortedPool.length !== resolvedPool.length
                        ? ` / ${String(resolvedPool.length)}`
                        : ""}
                      {poolGroups.length !== sortedPool.length
                        ? `, ${String(poolGroups.length)} unique`
                        : ""})
                    </span>
                  </span>
                  <div
                    className="mx-1 h-4"
                    style={{ borderLeft: "1px solid rgba(124, 58, 237, 0.3)" }}
                  />
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
                        style={{ opacity: tideFilters[tide] ? 1 : 0.4 }}
                      />
                      <span className="hidden sm:inline">{tide}</span>
                    </button>
                  ))}
                </div>

                {/* Row 2: Type filter + Sort */}
                <div className="flex flex-wrap items-center gap-2">
                  <span className="mr-1 text-[11px] uppercase tracking-wider opacity-60" style={{ color: "#a855f7" }}>
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

                  <div
                    className="mx-1 h-4"
                    style={{ borderLeft: "1px solid rgba(255, 255, 255, 0.1)" }}
                  />

                  <span className="mr-1 text-[11px] uppercase tracking-wider opacity-60" style={{ color: "#a855f7" }}>
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
                        className="absolute top-full left-0 z-50 mt-1 rounded-lg py-1 shadow-xl"
                        style={{
                          background: "#1a1025",
                          border: "1px solid rgba(124, 58, 237, 0.3)",
                          minWidth: "160px",
                        }}
                      >
                        {(Object.keys(SORT_LABELS) as SortCriteria[]).map((criteria) => (
                          <button
                            key={criteria}
                            className="block w-full cursor-pointer px-3 py-1.5 text-left text-xs transition-colors"
                            style={{
                              color: sortCriteria === criteria ? "#c084fc" : "#e2e8f0",
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
                        ))}
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
                    aria-label={sortAscending ? "Sort descending" : "Sort ascending"}
                  >
                    {sortAscending ? "\u2191" : "\u2193"}
                  </button>

                  <span className="ml-auto hidden text-[10px] opacity-30 xl:inline">
                    Click to add to deck. Right-click for details.
                  </span>
                </div>
              </div>

              {/* Pool card grid */}
              <div
                className="flex-1 overflow-y-auto px-4 py-3 md:px-6"
                style={{ background: "radial-gradient(ellipse at center, rgba(124, 58, 237, 0.03) 0%, transparent 70%)" }}
              >
                {poolGroups.length === 0 ? (
                  <div className="flex h-full items-center justify-center">
                    <p className="text-sm opacity-40">
                      {resolvedPool.length === 0
                        ? "No cards in pool. Move cards from your deck to manage them here."
                        : "No cards match the current filters."}
                    </p>
                  </div>
                ) : (
                  <div className="grid grid-cols-3 gap-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                    <AnimatePresence>
                      {poolGroups.map((group) => (
                        <motion.div
                          key={`${String(group.cardNumber)}-${group.transfiguration ?? "none"}-${String(group.isBane)}`}
                          className="relative"
                          initial={{ opacity: 0, scale: 0.9 }}
                          animate={{ opacity: 1, scale: 1 }}
                          exit={{ opacity: 0, scale: 0.95 }}
                          transition={{ duration: 0.15 }}
                        >
                          {/* Count badge for duplicates */}
                          {group.count > 1 && (
                            <div
                              className="absolute -top-1.5 -right-1.5 z-30 rounded-md px-1.5 py-0.5 text-[10px] font-bold shadow-md"
                              style={{
                                background: "#000",
                                color: "#fff",
                                border: "1px solid rgba(255, 255, 255, 0.3)",
                              }}
                            >
                              {String(group.count)}{"\u00D7"}
                            </div>
                          )}
                          {/* Transfiguration indicator */}
                          {group.transfiguration !== null && (
                            <div
                              className="absolute -top-1 z-10 rounded-full px-1.5 py-0.5 text-[9px] font-bold shadow-md"
                              style={{
                                right: group.count > 1 ? "1.5rem" : "-0.25rem",
                                background: TRANSFIGURATION_COLORS[group.transfiguration],
                                color: "#fff",
                                boxShadow: `0 0 6px ${TRANSFIGURATION_COLORS[group.transfiguration]}80`,
                              }}
                            >
                              {group.transfiguration}
                            </div>
                          )}
                          {/* Bane indicator */}
                          {group.isBane && (
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
                          {/* MAX badge for cards at copy limit */}
                          {(deckCopyCounts.get(group.cardNumber) ?? 0) >= config.maxCopies && (
                            <div
                              className="absolute inset-0 z-20 flex items-center justify-center rounded-lg"
                              style={{
                                background: "rgba(0, 0, 0, 0.5)",
                                pointerEvents: "none",
                              }}
                            >
                              <span
                                className="rounded px-2 py-1 text-xs font-bold"
                                style={{
                                  background: "rgba(239, 68, 68, 0.8)",
                                  color: "#fff",
                                }}
                              >
                                MAX
                              </span>
                            </div>
                          )}
                          <div
                            className="cursor-pointer transition-transform hover:scale-[1.03]"
                            style={
                              group.transfiguration !== null
                                ? {
                                    boxShadow: `0 0 8px ${TRANSFIGURATION_COLORS[group.transfiguration]}40`,
                                    borderRadius: "0.5rem",
                                  }
                                : group.isBane
                                  ? {
                                      boxShadow: "0 0 8px rgba(239, 68, 68, 0.3)",
                                      borderRadius: "0.5rem",
                                    }
                                  : undefined
                            }
                            onClick={() => {
                              handlePoolGroupClick(group);
                            }}
                            onContextMenu={(e) => {
                              e.preventDefault();
                              handleCardOverlay(group.card);
                            }}
                          >
                            <CardDisplay card={group.card} />
                          </div>
                        </motion.div>
                      ))}
                    </AnimatePresence>
                  </div>
                )}
              </div>
            </div>

            {/* Right panel: Deck list */}
            <div
              className="flex w-72 shrink-0 flex-col lg:w-80 xl:w-96"
              style={{
                background: "rgba(5, 2, 10, 0.6)",
                boxShadow: "inset 4px 0 12px rgba(0, 0, 0, 0.3)",
              }}
            >
              {/* Deck header */}
              <div
                className="flex items-center justify-between px-4 py-2.5"
                style={{
                  borderBottom: "1px solid rgba(124, 58, 237, 0.2)",
                  background: "rgba(10, 6, 18, 0.6)",
                }}
              >
                <h3 className="text-sm font-bold" style={{ color: "#a855f7" }}>
                  Deck{" "}
                  <span className="font-normal" style={{ color: deckSizeColor }}>
                    ({String(state.deck.length)} / {String(config.minimumDeckSize)}-{String(config.maximumDeckSize)})
                  </span>
                </h3>
              </div>

              {/* Deck card list */}
              <div ref={deckScrollRef} className="flex-1 overflow-y-auto px-2 py-2">
                {deckRowsByEnergy.length === 0 ? (
                  <div className="flex h-full items-center justify-center">
                    <p className="px-4 text-center text-sm opacity-40">
                      Deck is empty. Click cards in the pool to add them.
                    </p>
                  </div>
                ) : (
                  <AnimatePresence mode="popLayout">
                    {deckRowsByEnergy.map((group, groupIndex) => (
                      <div key={group.cost} className={groupIndex === 0 ? "mb-2" : "mb-2 mt-3"}>
                        {/* Cost divider */}
                        <div className="mb-1.5 flex items-center gap-2 px-2 py-1">
                          <div className="h-px flex-1" style={{ background: "rgba(124, 58, 237, 0.5)" }} />
                          <span
                            className="flex items-center gap-1.5 text-[11px] font-bold uppercase tracking-wider"
                            style={{ color: "#fbbf24" }}
                          >
                            <span
                              className="flex h-4 w-4 items-center justify-center rounded-full text-[9px]"
                              style={{
                                background: "rgba(251, 191, 36, 0.2)",
                                border: "1px solid rgba(251, 191, 36, 0.4)",
                              }}
                            >
                              {String(group.cost)}
                            </span>
                            Cost
                          </span>
                          <div className="h-px flex-1" style={{ background: "rgba(124, 58, 237, 0.5)" }} />
                        </div>

                        {/* Card rows */}
                        {group.rows.map((row) => (
                          <motion.div
                            key={`${String(row.cardNumber)}-${row.transfiguration ?? "none"}-${String(row.isBane)}`}
                            layout
                            layoutId={row.entryIds[row.entryIds.length - 1]}
                            initial={{ opacity: 0, x: -20 }}
                            animate={{ opacity: 1, x: 0 }}
                            exit={{ opacity: 0, x: -30 }}
                            transition={{
                              duration: 0.15,
                              layout: { duration: 0.35, ease: [0.4, 0, 0.2, 1] },
                            }}
                          >
                            <DeckListRow
                              row={row}
                              onClick={() => {
                                handleDeckRowClick(row);
                              }}
                              onRightClick={() => {
                                handleCardOverlay(row.card);
                              }}
                              onMouseEnter={(e) => {
                                if (hoverTimeoutRef.current) clearTimeout(hoverTimeoutRef.current);
                                const rect = e.currentTarget.getBoundingClientRect();
                                setHoverPreview({ card: row.card, top: rect.top });
                              }}
                              onMouseLeave={() => {
                                hoverTimeoutRef.current = setTimeout(() => {
                                  setHoverPreview(null);
                                }, 50);
                              }}
                            />
                          </motion.div>
                        ))}
                      </div>
                    ))}
                  </AnimatePresence>
                )}
              </div>

              {/* Bulk action buttons */}
              <div
                className="flex gap-2 px-3 py-2.5"
                style={{ borderTop: "1px solid rgba(124, 58, 237, 0.2)" }}
              >
                <button
                  className="flex-1 cursor-pointer rounded-lg px-3 py-2 text-xs font-semibold transition-colors"
                  style={{
                    background: "rgba(124, 58, 237, 0.25)",
                    border: "1px solid rgba(124, 58, 237, 0.5)",
                    color: "#c084fc",
                    opacity: state.pool.length === 0 ? 0.4 : 1,
                  }}
                  disabled={state.pool.length === 0}
                  onClick={() => {
                    mutations.moveAllToDeck();
                  }}
                >
                  {"\u2192"} Add All to Deck
                </button>
                <button
                  className="flex-1 cursor-pointer rounded-lg px-3 py-2 text-xs font-medium transition-colors"
                  style={{
                    background: "rgba(239, 68, 68, 0.15)",
                    border: "1px solid rgba(239, 68, 68, 0.4)",
                    color: "#f87171",
                    opacity: state.deck.length === 0 ? 0.4 : 1,
                  }}
                  disabled={state.deck.length === 0}
                  onClick={() => {
                    mutations.moveAllToPool();
                  }}
                >
                  {"\u2190"} Move All to Pool
                  <span className="block text-[9px] font-normal opacity-60">(banes will remain in deck)</span>
                </button>
              </div>
            </div>
          </div>
          )}

          {/* Hover preview for deck rows */}
          {hoverPreview !== null && (
            <div
              className="pointer-events-none fixed z-[70] w-56"
              style={{
                right: "calc(288px + 1rem)",
                top: Math.max(16, Math.min(hoverPreview.top - 80, window.innerHeight - 340)),
              }}
            >
              <CardDisplay card={hoverPreview.card} />
            </div>
          )}

          {/* Card overlay */}
          <CardOverlay card={overlayCard} onClose={handleCloseOverlay} />
        </motion.div>
      )}
    </AnimatePresence>
  );
}

/** A single row in the compact deck list. */
function DeckListRow({
  row,
  onClick,
  onRightClick,
  onMouseEnter,
  onMouseLeave,
}: {
  row: DeckRow;
  onClick: () => void;
  onRightClick: () => void;
  onMouseEnter: (e: React.MouseEvent) => void;
  onMouseLeave: () => void;
}) {
  const tideColor = TIDE_COLORS[row.card.tide];

  return (
    <button
      className="relative mb-1 flex w-full items-center gap-2 overflow-hidden rounded-lg px-3 py-2 text-left transition-all"
      style={{
        background: `linear-gradient(90deg, ${tideColor}20 0%, rgba(10, 6, 18, 0.8) 70%)`,
        borderLeft: `3px solid ${tideColor}80`,
        borderTop: `1px solid ${tideColor}15`,
        borderRight: `1px solid ${tideColor}15`,
        borderBottom: `1px solid ${tideColor}15`,
        cursor: row.isBane ? "not-allowed" : "pointer",
      }}
      onClick={onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        onRightClick();
      }}
      onMouseEnter={onMouseEnter}
      onMouseLeave={onMouseLeave}
    >
      {/* Card art crop (Hearthstone-style background) */}
      <img
        src={cardImageUrl(row.cardNumber)}
        alt=""
        className="pointer-events-none absolute top-0 right-0 h-full object-cover"
        style={{
          width: "45%",
          maskImage: "linear-gradient(to right, transparent 0%, black 60%)",
          WebkitMaskImage: "linear-gradient(to right, transparent 0%, black 60%)",
          opacity: 0.35,
        }}
      />

      {/* Tide icons (one per copy) */}
      <div className="relative z-10 flex shrink-0 items-center gap-0.5">
        {Array.from({ length: row.count }, (_, i) => (
          <img
            key={i}
            src={tideIconUrl(row.card.tide)}
            alt={row.card.tide}
            className="h-4 w-4 rounded-full"
            style={{ border: `1px solid ${tideColor}60` }}
          />
        ))}
      </div>

      {/* Card name */}
      <span
        className="relative z-10 min-w-0 flex-1 truncate text-xs font-medium"
        style={{ color: "#e2e8f0" }}
      >
        {row.card.name}
      </span>

      {/* Transfiguration badge */}
      {row.transfiguration !== null && (
        <span
          className="relative z-10 shrink-0 rounded-full px-1 py-0.5 text-[8px] font-bold"
          style={{
            background: TRANSFIGURATION_COLORS[row.transfiguration],
            color: "#fff",
          }}
        >
          {row.transfiguration.charAt(0)}
        </span>
      )}

      {/* Bane lock indicator */}
      {row.isBane && (
        <span className="relative z-10 shrink-0 text-[10px]" style={{ color: "#ef4444" }} title="Bane - cannot be removed">
          {"\uD83D\uDD12"}
        </span>
      )}

      {/* Energy cost */}
      <span
        className="relative z-10 flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-[10px] font-bold"
        style={{
          background: "rgba(251, 191, 36, 0.2)",
          color: "#fbbf24",
          border: "1px solid rgba(251, 191, 36, 0.3)",
        }}
      >
        {String(row.card.energyCost ?? 0)}
      </span>
    </button>
  );
}
