import { useCallback, useEffect, useMemo, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import type { CardData, Tide } from "../types/cards";
import type { DeckEntry } from "../types/quest";
import { useQuest } from "../state/quest-context";
import { useQuestConfig } from "../state/quest-config";
import { TIDE_COLORS, tideIconUrl } from "../data/card-database";
import { CardDisplay } from "./CardDisplay";
import { CardOverlay } from "./CardOverlay";
import { TRANSFIGURATION_COLORS } from "../transfiguration/transfiguration-logic";
import { computeTideDistribution } from "./tide-distribution";
import {
  ALL_TIDES,
  SORT_LABELS,
  SIZE_PRESETS,
  TIDE_ORDER,
  type CardSizePreset,
  type SortCriteria,
} from "./DeckViewer";

/** Card type filter options. */
type CardTypeFilter = "All" | "Characters" | "Events";

/** A resolved entry with card data. */
interface ResolvedEntry {
  entry: DeckEntry;
  card: CardData;
}

/** A group of identical cards for display. */
interface CardGroup {
  card: CardData;
  entries: ResolvedEntry[];
  count: number;
  representative: ResolvedEntry;
  transfiguration: DeckEntry["transfiguration"];
  isBane: boolean;
}

/** LocalStorage key for persisting card size preference. */
const CARD_SIZE_STORAGE_KEY = "quest-card-size";

function getPersistedCardSize(fallback: CardSizePreset): CardSizePreset {
  try {
    const stored = localStorage.getItem(CARD_SIZE_STORAGE_KEY);
    if (stored === "small" || stored === "medium" || stored === "large") return stored;
  } catch {
    // ignore
  }
  return fallback;
}

function persistCardSize(size: CardSizePreset): void {
  try {
    localStorage.setItem(CARD_SIZE_STORAGE_KEY, size);
  } catch {
    // ignore
  }
}

/** Props for the VisualDeckEditor component. */
interface VisualDeckEditorProps {
  cardDatabase: Map<number, CardData>;
}

function resolveAndGroup(
  entries: DeckEntry[],
  cardDatabase: Map<number, CardData>,
  tideFilters: Record<Tide, boolean>,
  cardTypeFilter: CardTypeFilter,
  sortCriteria: SortCriteria,
  sortAscending: boolean,
): { groups: CardGroup[]; totalResolved: number; totalFiltered: number } {
  const resolved: ResolvedEntry[] = [];
  for (const entry of entries) {
    const card = cardDatabase.get(entry.cardNumber);
    if (card) resolved.push({ entry, card });
  }
  const totalResolved = resolved.length;

  const filtered = resolved.filter((r) => {
    if (!tideFilters[r.card.tide]) return false;
    if (cardTypeFilter === "Characters" && r.card.cardType !== "Character") return false;
    if (cardTypeFilter === "Events" && r.card.cardType !== "Event") return false;
    return true;
  });
  const totalFiltered = filtered.length;

  filtered.sort((a, b) => {
    let cmp = 0;
    switch (sortCriteria) {
      case "acquisitionOrder":
        cmp = 0; // no index tracking for pool
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
    if (cmp === 0) cmp = (a.card.energyCost ?? 0) - (b.card.energyCost ?? 0);
    if (cmp === 0) cmp = a.card.name.localeCompare(b.card.name);
    return sortAscending ? cmp : -cmp;
  });

  const groups: CardGroup[] = [];
  const seen = new Map<string, CardGroup>();
  for (const r of filtered) {
    const key = `${String(r.entry.cardNumber)}-${r.entry.transfiguration ?? "none"}-${String(r.entry.isBane)}`;
    const existing = seen.get(key);
    if (existing) {
      existing.entries.push(r);
      existing.count += 1;
    } else {
      const group: CardGroup = {
        card: r.card,
        entries: [r],
        count: 1,
        representative: r,
        transfiguration: r.entry.transfiguration,
        isBane: r.entry.isBane,
      };
      groups.push(group);
      seen.set(key, group);
    }
  }

  return { groups, totalResolved, totalFiltered };
}

export function VisualDeckEditor({ cardDatabase }: VisualDeckEditorProps) {
  const { state, mutations } = useQuest();
  const config = useQuestConfig();

  const [tideFilters, setTideFilters] = useState<Record<Tide, boolean>>(() => {
    const filters: Partial<Record<Tide, boolean>> = {};
    for (const tide of ALL_TIDES) {
      filters[tide] = true;
    }
    return filters as Record<Tide, boolean>;
  });
  const [cardTypeFilter, setCardTypeFilter] = useState<CardTypeFilter>("All");
  const [sortCriteria, setSortCriteria] = useState<SortCriteria>("tide");
  const [sortAscending, setSortAscending] = useState(true);
  const [overlayCard, setOverlayCard] = useState<CardData | null>(null);
  const [showSortDropdown, setShowSortDropdown] = useState(false);
  const [cardSize, setCardSizeState] = useState<CardSizePreset>(() =>
    getPersistedCardSize("medium"),
  );

  const setCardSize = useCallback((size: CardSizePreset) => {
    setCardSizeState(size);
    persistCardSize(size);
  }, []);

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

  const tideDistribution = useMemo(
    () => computeTideDistribution(state.deck, cardDatabase),
    [state.deck, cardDatabase],
  );

  const deckResult = useMemo(
    () =>
      resolveAndGroup(state.deck, cardDatabase, tideFilters, cardTypeFilter, sortCriteria, sortAscending),
    [state.deck, cardDatabase, tideFilters, cardTypeFilter, sortCriteria, sortAscending],
  );

  const poolResult = useMemo(
    () =>
      resolveAndGroup(state.pool, cardDatabase, tideFilters, cardTypeFilter, sortCriteria, sortAscending),
    [state.pool, cardDatabase, tideFilters, cardTypeFilter, sortCriteria, sortAscending],
  );

  const deckCopyCounts = useMemo(() => {
    const counts = new Map<number, number>();
    for (const entry of state.deck) {
      counts.set(entry.cardNumber, (counts.get(entry.cardNumber) ?? 0) + 1);
    }
    return counts;
  }, [state.deck]);

  const toggleTide = useCallback((tide: Tide) => {
    setTideFilters((prev) => ({ ...prev, [tide]: !prev[tide] }));
  }, []);

  const handleDeckGroupClick = useCallback(
    (group: CardGroup) => {
      if (group.isBane) return;
      const entryId = group.entries[group.entries.length - 1].entry.entryId;
      mutations.moveToPool(entryId);
    },
    [mutations],
  );

  const handlePoolGroupClick = useCallback(
    (group: CardGroup) => {
      const currentCopies = deckCopyCounts.get(group.card.cardNumber) ?? 0;
      if (currentCopies >= config.maxCopies) return;
      const entryId = group.entries[group.entries.length - 1].entry.entryId;
      mutations.moveToDeck(entryId);
    },
    [mutations, deckCopyCounts, config.maxCopies],
  );

  const handleCardOverlay = useCallback((card: CardData) => {
    setOverlayCard(card);
  }, []);

  const handleCloseOverlay = useCallback(() => {
    setOverlayCard(null);
  }, []);

  const deckSizeColor =
    state.deck.length < config.minimumDeckSize || state.deck.length > config.maximumDeckSize
      ? "#ef4444"
      : "#10b981";

  return (
    <>
      {/* Tide distribution bar */}
      {tideDistribution.total > 0 && (
        <div
          className="px-4 py-2 md:px-6"
          style={{
            borderBottom: "1px solid rgba(124, 58, 237, 0.15)",
            background: "rgba(10, 6, 18, 0.5)",
          }}
        >
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
          <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
            {tideDistribution.tides.map((t) => (
              <div
                key={t.tide}
                className="flex items-center gap-1"
                style={{ opacity: t.count > 0 ? 1 : 0.3 }}
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
                  style={{ color: t.count > 0 ? TIDE_COLORS[t.tide] : "#6b7280" }}
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
                  <span className="text-[10px]" style={{ color: "#9ca3af" }}>
                    ({String(t.percentage)}%)
                  </span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Controls row */}
      <div
        className="relative z-20 flex flex-wrap items-center gap-2 px-4 py-2 md:px-6"
        style={{
          borderBottom: "1px solid rgba(124, 58, 237, 0.15)",
          background: "rgba(10, 6, 18, 0.6)",
        }}
      >
        {/* Tide filter toggles */}
        <div className="flex flex-wrap items-center gap-1">
          <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">Tides</span>
          {ALL_TIDES.map((tide) => (
            <button
              key={tide}
              className="flex cursor-pointer items-center gap-1 rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
              style={{
                background: tideFilters[tide] ? `${TIDE_COLORS[tide]}25` : "rgba(255, 255, 255, 0.03)",
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

        <div
          className="mx-1 hidden h-5 md:block"
          style={{ borderLeft: "1px solid rgba(255, 255, 255, 0.1)" }}
        />

        {/* Card type filter */}
        <div className="flex items-center gap-1">
          <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">Type</span>
          {(["All", "Characters", "Events"] as const).map((filter) => (
            <button
              key={filter}
              className="cursor-pointer rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
              style={{
                background: cardTypeFilter === filter ? "rgba(168, 85, 247, 0.25)" : "rgba(255, 255, 255, 0.03)",
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

        <div
          className="mx-1 hidden h-5 md:block"
          style={{ borderLeft: "1px solid rgba(255, 255, 255, 0.1)" }}
        />

        {/* Sort controls */}
        <div className="flex items-center gap-1">
          <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">Sort</span>
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
                      background: sortCriteria === criteria ? "rgba(168, 85, 247, 0.15)" : "transparent",
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
        </div>

        <div
          className="mx-1 hidden h-5 md:block"
          style={{ borderLeft: "1px solid rgba(255, 255, 255, 0.1)" }}
        />

        {/* Card size controls */}
        <div className="flex items-center gap-1">
          <span className="mr-1 text-[10px] uppercase tracking-wider opacity-40">Size</span>
          {(Object.keys(SIZE_PRESETS) as CardSizePreset[]).map((preset) => (
            <button
              key={preset}
              className="cursor-pointer rounded-full px-2 py-0.5 text-[11px] font-medium transition-all"
              style={{
                background: cardSize === preset ? "rgba(168, 85, 247, 0.25)" : "rgba(255, 255, 255, 0.03)",
                border: `1px solid ${cardSize === preset ? "rgba(168, 85, 247, 0.5)" : "rgba(255, 255, 255, 0.1)"}`,
                color: cardSize === preset ? "#c084fc" : "#6b7280",
              }}
              onClick={() => {
                setCardSize(preset);
              }}
            >
              {SIZE_PRESETS[preset].label}
            </button>
          ))}
        </div>

        <span className="ml-auto hidden text-[10px] opacity-30 xl:inline">
          Click to move cards. Right-click for details.
        </span>
      </div>

      {/* Scrollable content: deck grid, divider, pool grid */}
      <div
        className="flex-1 overflow-y-auto px-4 py-3 md:px-6"
        style={{ background: "radial-gradient(ellipse at center, rgba(124, 58, 237, 0.03) 0%, transparent 70%)" }}
      >
        {/* Deck section */}
        <div className="mb-2 flex items-center gap-2">
          <h3 className="text-sm font-bold" style={{ color: "#a855f7" }}>
            Deck
          </h3>
          <span
            className="rounded-full px-2 py-0.5 text-[11px] font-bold"
            style={{
              background: "rgba(251, 191, 36, 0.15)",
              border: "1px solid rgba(251, 191, 36, 0.3)",
              color: deckSizeColor,
            }}
          >
            {String(state.deck.length)} / {String(config.minimumDeckSize)}-{String(config.maximumDeckSize)}
          </span>
          {deckResult.totalFiltered !== deckResult.totalResolved && (
            <span className="text-[10px] opacity-40">
              (showing {String(deckResult.totalFiltered)} of {String(deckResult.totalResolved)})
            </span>
          )}
        </div>

        {deckResult.groups.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-sm opacity-40">
              {deckResult.totalResolved === 0
                ? "Deck is empty. Click cards in the pool below to add them."
                : "No deck cards match the current filters."}
            </p>
          </div>
        ) : (
          <div
            style={{
              display: "grid",
              gridTemplateColumns: SIZE_PRESETS[cardSize].columns,
              gap: SIZE_PRESETS[cardSize].gap,
            }}
          >
            <AnimatePresence>
              {deckResult.groups.map((group) => (
                <CardGroupTile
                  key={`deck-${String(group.representative.entry.cardNumber)}-${group.transfiguration ?? "none"}-${String(group.isBane)}`}
                  group={group}
                  cardSize={cardSize}
                  onClick={() => {
                    handleDeckGroupClick(group);
                  }}
                  onRightClick={() => {
                    handleCardOverlay(group.card);
                  }}
                  dimmed={false}
                />
              ))}
            </AnimatePresence>
          </div>
        )}

        {/* Divider */}
        <div className="my-4 flex items-center gap-3">
          <div className="h-px flex-1" style={{ background: "rgba(124, 58, 237, 0.4)" }} />
          <span
            className="text-[11px] font-bold uppercase tracking-wider"
            style={{ color: "#a855f7", opacity: 0.6 }}
          >
            Pool
          </span>
          <span className="text-[10px] opacity-40">
            {String(poolResult.totalFiltered)}
            {poolResult.totalFiltered !== poolResult.totalResolved
              ? ` / ${String(poolResult.totalResolved)}`
              : ""}{" "}
            cards
          </span>
          <div className="h-px flex-1" style={{ background: "rgba(124, 58, 237, 0.4)" }} />
        </div>

        {/* Pool section */}
        {poolResult.groups.length === 0 ? (
          <div className="flex items-center justify-center py-8">
            <p className="text-sm opacity-40">
              {poolResult.totalResolved === 0
                ? "No cards in pool."
                : "No pool cards match the current filters."}
            </p>
          </div>
        ) : (
          <div
            style={{
              display: "grid",
              gridTemplateColumns: SIZE_PRESETS[cardSize].columns,
              gap: SIZE_PRESETS[cardSize].gap,
            }}
          >
            <AnimatePresence>
              {poolResult.groups.map((group) => {
                const atMax = (deckCopyCounts.get(group.card.cardNumber) ?? 0) >= config.maxCopies;
                return (
                  <CardGroupTile
                    key={`pool-${String(group.representative.entry.cardNumber)}-${group.transfiguration ?? "none"}-${String(group.isBane)}`}
                    group={group}
                    cardSize={cardSize}
                    onClick={() => {
                      handlePoolGroupClick(group);
                    }}
                    onRightClick={() => {
                      handleCardOverlay(group.card);
                    }}
                    dimmed={atMax}
                  />
                );
              })}
            </AnimatePresence>
          </div>
        )}
      </div>

      <CardOverlay card={overlayCard} onClose={handleCloseOverlay} />
    </>
  );
}

/** A single card tile in the visual grid, with badges for count/transfig/bane. */
function CardGroupTile({
  group,
  cardSize,
  onClick,
  onRightClick,
  dimmed,
}: {
  group: CardGroup;
  cardSize: CardSizePreset;
  onClick: () => void;
  onRightClick: () => void;
  dimmed: boolean;
}) {
  const rep = group.representative;

  return (
    <motion.div
      className="relative"
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.95 }}
      transition={{ duration: 0.15 }}
    >
      {/* Copy count badge */}
      {group.count > 1 && (
        <div
          className="absolute -top-1.5 -right-1.5 z-20 flex items-center justify-center rounded-full px-1.5 py-0.5 text-[10px] font-bold shadow-lg"
          style={{
            background: "rgba(124, 58, 237, 0.9)",
            color: "#fff",
            border: "1.5px solid rgba(168, 85, 247, 0.6)",
            minWidth: "22px",
            textAlign: "center",
          }}
        >
          {String(group.count)}x
        </div>
      )}
      {/* Transfiguration indicator */}
      {rep.entry.transfiguration !== null && (
        <div
          className={`absolute ${group.count > 1 ? "top-4 -right-1" : "-top-1 -right-1"} z-10 rounded-full px-1.5 py-0.5 text-[9px] font-bold shadow-md`}
          style={{
            background: TRANSFIGURATION_COLORS[rep.entry.transfiguration],
            color: "#fff",
            boxShadow: `0 0 6px ${TRANSFIGURATION_COLORS[rep.entry.transfiguration]}80`,
          }}
        >
          {rep.entry.transfiguration}
        </div>
      )}
      {/* Bane indicator */}
      {rep.entry.isBane && (
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
      {/* MAX overlay */}
      {dimmed && (
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
          rep.entry.transfiguration !== null
            ? {
                boxShadow: `0 0 8px ${TRANSFIGURATION_COLORS[rep.entry.transfiguration]}40`,
                borderRadius: "0.5rem",
              }
            : rep.entry.isBane
              ? {
                  boxShadow: "0 0 8px rgba(239, 68, 68, 0.3)",
                  borderRadius: "0.5rem",
                }
              : undefined
        }
        onClick={onClick}
        onContextMenu={(e) => {
          e.preventDefault();
          onRightClick();
        }}
      >
        <CardDisplay card={group.card} compact={cardSize === "small"} />
      </div>
    </motion.div>
  );
}
