import type { CardData, Tide } from "../types/cards";
import type { DraftConfig, DraftState } from "../types/draft";
import { NAMED_TIDES } from "../data/card-database";
import { logEvent } from "../logging";

/** Default configuration for the Tide Current draft algorithm. */
export const DEFAULT_DRAFT_CONFIG: Readonly<DraftConfig> = {
  packSize: 4,
  baseAffinity: 1.0,
  focusStartPick: 3,
  focusRate: 0.35,
  decayFactor: 0.85,
  allySimilarity: 0.5,
  distance2Similarity: 0.15,
  distance3Similarity: 0.05,
  neutralDraftContribution: 0.4,
  neutralAffinityFactor: 0.5,
};

/** Number of player picks per draft site visit. */
export const SITE_PICKS = 5;

/** Tide ordering for display sorting. */
const TIDE_ORDER: Readonly<Record<string, number>> = {
  Bloom: 0,
  Arc: 1,
  Ignite: 2,
  Pact: 3,
  Umbra: 4,
  Rime: 5,
  Surge: 6,
  Neutral: 7,
};

/**
 * Tide circle: Bloom(0) -> Arc(1) -> Ignite(2) -> Pact(3) -> Umbra(4) -> Rime(5) -> Surge(6) -> Bloom
 * Distance is shortest path on the circle of 7 named tides.
 */
const TIDE_CIRCLE_ORDER: readonly string[] = [
  "Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge",
];

/** Compute shortest distance on the 7-tide circle. */
function tideCircleDistance(a: string, b: string): number {
  const idxA = TIDE_CIRCLE_ORDER.indexOf(a);
  const idxB = TIDE_CIRCLE_ORDER.indexOf(b);
  if (idxA === -1 || idxB === -1) return -1; // Neutral has no circle distance
  const diff = Math.abs(idxA - idxB);
  return Math.min(diff, 7 - diff);
}

/** Returns circle similarity for a given distance. */
function circleSimilarity(dist: number, config: DraftConfig): number {
  switch (dist) {
    case 0: return 1.0;
    case 1: return config.allySimilarity;
    case 2: return config.distance2Similarity;
    case 3: return config.distance3Similarity;
    default: return 0;
  }
}

/**
 * Compute tide affinity for each core tide plus Neutral based on
 * the player's drafted cards with recency decay.
 */
export function computeTideAffinity(
  draftedCards: number[],
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): Map<string, number> {
  const affinity = new Map<string, number>();
  for (const tide of NAMED_TIDES) {
    affinity.set(tide, config.baseAffinity);
  }
  affinity.set("Neutral", config.baseAffinity);

  let neutralCount = 0;

  // draftedCards is ordered newest first (index 0 = most recent)
  for (let position = 0; position < draftedCards.length; position++) {
    const card = cardDatabase.get(draftedCards[position]);
    if (!card) continue;

    const decay = Math.pow(config.decayFactor, position);

    if (card.tide === "Neutral") {
      neutralCount++;
      // Neutral cards contribute to all core tides
      for (const tide of NAMED_TIDES) {
        affinity.set(
          tide,
          affinity.get(tide)! + config.neutralDraftContribution * decay,
        );
      }
    } else {
      // Named tide card: contribute similarity-weighted affinity to all core tides
      for (const tide of NAMED_TIDES) {
        const dist = tideCircleDistance(card.tide, tide);
        const sim = circleSimilarity(dist, config);
        if (sim > 0) {
          affinity.set(tide, affinity.get(tide)! + sim * decay);
        }
      }
    }
  }

  // Neutral affinity: max(base + neutral_count, factor * max_core)
  const maxCoreAffinity = Math.max(
    ...NAMED_TIDES.map((t) => affinity.get(t)!),
  );
  const neutralAffinity = Math.max(
    config.baseAffinity + neutralCount,
    config.neutralAffinityFactor * maxCoreAffinity,
  );
  affinity.set("Neutral", neutralAffinity);

  return affinity;
}

/** Compute focus value for a given pick number. */
export function computeFocus(
  pickNumber: number,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): number {
  return Math.max(0, (pickNumber - config.focusStartPick) * config.focusRate);
}

/** Compute sampling weight for a card given tide affinities and focus. */
function computeCardWeight(
  card: CardData,
  affinity: Map<string, number>,
  focus: number,
): number {
  const tideAffinity = affinity.get(card.tide) ?? 1.0;
  return Math.pow(tideAffinity, focus);
}

/** Fisher-Yates shuffle of an array in place. */
function shuffle<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
  return arr;
}

/**
 * Draw a pack of cards from the pool using weighted sampling without
 * replacement. Cards are sampled proportional to their affinity-based weight.
 */
export function drawPack(
  pool: number[],
  cardDatabase: Map<number, CardData>,
  draftedCards: number[],
  pickNumber: number,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): number[] {
  const affinity = computeTideAffinity(draftedCards, cardDatabase, config);
  const focus = computeFocus(pickNumber, config);

  // Build weighted entries for the pool
  const entries: Array<{ cardNumber: number; weight: number }> = [];
  for (const cardNumber of pool) {
    const card = cardDatabase.get(cardNumber);
    if (!card) continue;
    entries.push({ cardNumber, weight: computeCardWeight(card, affinity, focus) });
  }

  // Sample packSize cards without replacement, proportional to weight
  const packSize = Math.min(config.packSize, entries.length);
  const selected: number[] = [];

  for (let i = 0; i < packSize; i++) {
    const totalWeight = entries.reduce((sum, e) => sum + e.weight, 0);
    if (totalWeight <= 0) break;

    let roll = Math.random() * totalWeight;
    let chosenIdx = entries.length - 1;
    for (let j = 0; j < entries.length; j++) {
      roll -= entries[j].weight;
      if (roll <= 0) {
        chosenIdx = j;
        break;
      }
    }

    selected.push(entries[chosenIdx].cardNumber);
    entries.splice(chosenIdx, 1);
  }

  return selected;
}

/** Sort an array of cards by tide order without mutating the original. */
export function sortCardsByTide(cards: CardData[]): CardData[] {
  return [...cards].sort((a, b) => {
    const orderA = TIDE_ORDER[a.tide] ?? 8;
    const orderB = TIDE_ORDER[b.tide] ?? 8;
    return orderA - orderB;
  });
}

/** Count cards per tide in a collection of card numbers. */
function countByTide(
  cardNumbers: number[],
  cardDatabase: Map<number, CardData>,
): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const tide of [...NAMED_TIDES, "Neutral" as Tide]) {
    counts[tide] = 0;
  }
  for (const num of cardNumbers) {
    const card = cardDatabase.get(num);
    if (card) {
      counts[card.tide] = (counts[card.tide] ?? 0) + 1;
    }
  }
  return counts;
}

/** Create initial DraftState, excluding cards from specified tides. */
export function initializeDraftState(
  cardDatabase: Map<number, CardData>,
  excludedTides: Tide[],
): DraftState {
  const excludedSet = new Set(excludedTides);
  const pool = Array.from(cardDatabase.keys()).filter((cardNum) => {
    const card = cardDatabase.get(cardNum);
    return card !== undefined && !excludedSet.has(card.tide);
  });

  logEvent("draft_pool_initialized", {
    poolSize: pool.length,
    excludedTides,
    cardCountByTide: countByTide(pool, cardDatabase),
  });

  return {
    pool: shuffle([...pool]),
    draftedCards: [],
    currentPack: [],
    pickNumber: 1,
    sitePicksCompleted: 0,
  };
}

/** Prepare the state for a draft site visit. Draws the first pack. */
export function enterDraftSite(
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  state.sitePicksCompleted = 0;
  state.currentPack = drawPack(
    state.pool,
    cardDatabase,
    state.draftedCards,
    state.pickNumber,
    config,
  );

  logEvent("draft_site_entered", {
    pickNumber: state.pickNumber,
    poolSize: state.pool.length,
    packCards: state.currentPack,
  });
}

/** Return the current pack for display. */
export function getPlayerPack(state: DraftState): number[] {
  return state.currentPack;
}

/**
 * Process a player pick. The picked card is added to draftedCards,
 * and all pack cards are removed from the pool. Returns whether
 * the site visit is complete.
 */
export function processPlayerPick(
  cardNumber: number,
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): boolean {
  const packIndex = state.currentPack.indexOf(cardNumber);
  if (packIndex === -1) {
    throw new Error(
      `Card ${String(cardNumber)} is not in the current pack`,
    );
  }

  const card = cardDatabase.get(cardNumber);

  // Add picked card to drafted cards (newest first)
  state.draftedCards.unshift(cardNumber);

  // Remove all pack cards from the pool
  const packSet = new Set(state.currentPack);
  state.pool = state.pool.filter((num) => !packSet.has(num));

  logEvent("draft_pick_player", {
    pickNumber: state.pickNumber,
    cardNumber,
    cardName: card?.name ?? "Unknown",
    cardTide: card?.tide ?? "Neutral",
    packCards: state.currentPack,
    poolRemaining: state.pool.length,
  });

  state.pickNumber += 1;
  state.sitePicksCompleted += 1;

  if (state.sitePicksCompleted >= SITE_PICKS) {
    return true;
  }

  // Draw the next pack
  state.currentPack = drawPack(
    state.pool,
    cardDatabase,
    state.draftedCards,
    state.pickNumber,
    config,
  );

  return false;
}

/** Finalize a draft site visit. Log the cards drafted during this visit. */
export function completeDraftSite(state: DraftState): void {
  const sitePicks = state.draftedCards.slice(0, state.sitePicksCompleted);

  logEvent("draft_site_completed", {
    cardsDrafted: sitePicks,
    totalDrafted: state.draftedCards.length,
    poolRemaining: state.pool.length,
  });
}
