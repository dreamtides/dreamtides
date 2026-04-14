import type { CardData, Tide } from "../types/cards";
import type { DraftConfig, DraftState, PackContext } from "../types/draft";
import { cardAccentTide, NAMED_TIDES } from "../data/card-database";
import { logEvent } from "../logging";

/** Default shared draft configuration. */
export const DEFAULT_DRAFT_CONFIG: Readonly<DraftConfig> = {
  packSize: 4,
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

/** Weight applied to cards that have appeared in previous packs but were not picked. */
const SEEN_CARD_WEIGHT = 0.3;

/** Fisher-Yates shuffle of an array in place. */
function shuffle<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
  return arr;
}

/**
 * Sample cards from weighted entries without replacement.
 * Returns the selected card numbers.
 */
function weightedSample(
  entries: Array<{ cardNumber: number; weight: number }>,
  count: number,
): number[] {
  const packSize = Math.min(count, entries.length);
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

/** Generate a pack using depletion-weighted random sampling. */
function depletionPack(ctx: PackContext, seenCards: Set<number>): number[] {
  const entries: Array<{ cardNumber: number; weight: number }> = [];
  for (const cardNumber of ctx.pool) {
    const weight = seenCards.has(cardNumber) ? SEEN_CARD_WEIGHT : 1.0;
    entries.push({ cardNumber, weight });
  }
  return weightedSample(entries, ctx.packSize);
}

/** Sort an array of cards by tide order without mutating the original. */
export function sortCardsByTide(cards: CardData[]): CardData[] {
  return [...cards].sort((a, b) => {
    const orderA = TIDE_ORDER[cardAccentTide(a)] ?? 8;
    const orderB = TIDE_ORDER[cardAccentTide(b)] ?? 8;
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
      const accentTide = cardAccentTide(card);
      counts[accentTide] = (counts[accentTide] ?? 0) + 1;
    }
  }
  return counts;
}

/** Create initial DraftState, filtering to the chosen tide + Neutral. */
export function initializeDraftState(
  cardDatabase: Map<number, CardData>,
  chosenTide: Tide,
): DraftState {
  const pool = Array.from(cardDatabase.keys()).filter((cardNum) => {
    const card = cardDatabase.get(cardNum);
    return card !== undefined &&
      (cardAccentTide(card) === chosenTide || cardAccentTide(card) === "Neutral");
  });

  logEvent("draft_pool_initialized", {
    poolSize: pool.length,
    chosenTide,
    cardCountByTide: countByTide(pool, cardDatabase),
    packStrategy: { type: "depletion" },
  });

  return {
    pool: shuffle([...pool]),
    draftedCards: [],
    currentPack: [],
    pickNumber: 1,
    sitePicksCompleted: 0,
    packStrategy: { type: "depletion" },
    seenCards: [],
  };
}

/** Prepare the state for a draft site visit. Draws the first pack. */
export function enterDraftSite(
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  state.sitePicksCompleted = 0;
  const seenSet = new Set(state.seenCards);
  state.currentPack = depletionPack(
    {
      pool: state.pool,
      cardDatabase,
      draftedCards: state.draftedCards,
      pickNumber: state.pickNumber,
      packSize: config.packSize,
    },
    seenSet,
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

  // Track unpicked cards as seen for depletion weighting
  for (const packCard of state.currentPack) {
    if (packCard !== cardNumber) {
      state.seenCards.push(packCard);
    }
  }

  // Remove all pack cards from the pool
  const packSet = new Set(state.currentPack);
  state.pool = state.pool.filter((num) => !packSet.has(num));

  logEvent("draft_pick_player", {
    pickNumber: state.pickNumber,
    cardNumber,
    cardName: card?.name ?? "Unknown",
    cardTide: card === undefined ? "Neutral" : cardAccentTide(card),
    packCards: state.currentPack,
    poolRemaining: state.pool.length,
  });

  state.pickNumber += 1;
  state.sitePicksCompleted += 1;

  if (state.sitePicksCompleted >= SITE_PICKS) {
    return true;
  }

  // Draw the next pack
  const seenSet = new Set(state.seenCards);
  state.currentPack = depletionPack(
    {
      pool: state.pool,
      cardDatabase,
      draftedCards: state.draftedCards,
      pickNumber: state.pickNumber,
      packSize: config.packSize,
    },
    seenSet,
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
