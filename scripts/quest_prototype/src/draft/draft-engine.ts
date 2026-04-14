import type { CardData, Tide } from "../types/cards";
import type { ResolvedDreamcallerPackage } from "../types/content";
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

/**
 * Sample unique card numbers from weighted entries without replacement.
 * Returns the selected card numbers.
 */
function weightedSample(
  entries: Array<{ cardNumber: number; weight: number }>,
  count: number,
): number[] {
  const packSize = Math.min(count, entries.length);
  const selected: number[] = [];

  for (let i = 0; i < packSize; i++) {
    const totalWeight = entries.reduce((sum, entry) => sum + entry.weight, 0);
    if (totalWeight <= 0) {
      break;
    }

    let roll = Math.random() * totalWeight;
    let chosenIndex = entries.length - 1;
    for (let index = 0; index < entries.length; index += 1) {
      roll -= entries[index].weight;
      if (roll <= 0) {
        chosenIndex = index;
        break;
      }
    }

    selected.push(entries[chosenIndex].cardNumber);
    entries.splice(chosenIndex, 1);
  }

  return selected;
}

/** Build a 4-unique-card offer weighted by remaining copies. */
function buildOffer(ctx: PackContext): number[] {
  const entries: Array<{ cardNumber: number; weight: number }> = [];

  for (const [cardNumberText, copies] of Object.entries(
    ctx.remainingCopiesByCard,
  )) {
    const cardNumber = Number(cardNumberText);
    if (!Number.isInteger(cardNumber) || copies <= 0) {
      continue;
    }

    entries.push({ cardNumber, weight: copies });
  }

  if (entries.length < ctx.packSize) {
    return [];
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

  for (const cardNumber of cardNumbers) {
    const card = cardDatabase.get(cardNumber);
    if (card === undefined) {
      continue;
    }

    const accentTide = cardAccentTide(card);
    counts[accentTide] = (counts[accentTide] ?? 0) + 1;
  }

  return counts;
}

function sanitizeDraftPoolCopies(
  cardDatabase: Map<number, CardData>,
  draftPoolCopiesByCard: Record<string, number>,
): Record<string, number> {
  const remainingCopiesByCard: Record<string, number> = {};

  for (const [cardNumberText, copies] of Object.entries(draftPoolCopiesByCard)) {
    const cardNumber = Number(cardNumberText);
    if (
      !Number.isInteger(cardNumber) ||
      !cardDatabase.has(cardNumber) ||
      copies <= 0
    ) {
      continue;
    }

    remainingCopiesByCard[String(cardNumber)] = copies;
  }

  return remainingCopiesByCard;
}

function expandRemainingCopies(remainingCopiesByCard: Record<string, number>): number[] {
  return Object.entries(remainingCopiesByCard).flatMap(([cardNumberText, copies]) =>
    Array.from({ length: copies }, () => Number(cardNumberText)),
  );
}

function countRemainingCards(remainingCopiesByCard: Record<string, number>): number {
  return Object.values(remainingCopiesByCard).reduce(
    (total, copies) => total + copies,
    0,
  );
}

/** Create initial DraftState from the resolved Dreamcaller package. */
export function initializeDraftState(
  cardDatabase: Map<number, CardData>,
  resolvedPackage: ResolvedDreamcallerPackage,
): DraftState {
  const remainingCopiesByCard = sanitizeDraftPoolCopies(
    cardDatabase,
    resolvedPackage.draftPoolCopiesByCard,
  );
  const expandedPool = expandRemainingCopies(remainingCopiesByCard);

  logEvent("draft_pool_initialized", {
    poolSize: countRemainingCards(remainingCopiesByCard),
    uniqueCardCount: Object.keys(remainingCopiesByCard).length,
    dreamcallerId: resolvedPackage.dreamcaller.id,
    selectedPackageTides: resolvedPackage.selectedTides,
    cardCountByTide: countByTide(expandedPool, cardDatabase),
  });

  return {
    remainingCopiesByCard,
    currentOffer: [],
    pickNumber: 1,
    sitePicksCompleted: 0,
  };
}

/** Prepare the state for a draft site visit. Draws the first pack. */
export function enterDraftSite(
  state: DraftState,
  _cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  state.sitePicksCompleted = 0;
  state.currentOffer = buildOffer({
    remainingCopiesByCard: state.remainingCopiesByCard,
    pickNumber: state.pickNumber,
    packSize: config.packSize,
  });

  logEvent("draft_site_entered", {
    pickNumber: state.pickNumber,
    poolSize: countRemainingCards(state.remainingCopiesByCard),
    offerCards: state.currentOffer,
  });
}

/** Return the current pack for display. */
export function getPlayerPack(state: DraftState): number[] {
  return state.currentOffer;
}

/**
 * Process a player pick. The shown cards are spent from the fixed pool.
 * Returns whether the site visit is complete.
 */
export function processPlayerPick(
  cardNumber: number,
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): boolean {
  const offerIndex = state.currentOffer.indexOf(cardNumber);
  if (offerIndex === -1) {
    throw new Error(
      `Card ${String(cardNumber)} is not in the current offer`,
    );
  }

  const card = cardDatabase.get(cardNumber);

  for (const offeredCardNumber of state.currentOffer) {
    const key = String(offeredCardNumber);
    const remainingCopies = state.remainingCopiesByCard[key];
    if (remainingCopies === undefined) {
      continue;
    }

    if (remainingCopies <= 1) {
      delete state.remainingCopiesByCard[key];
    } else {
      state.remainingCopiesByCard[key] = remainingCopies - 1;
    }
  }

  logEvent("draft_pick_player", {
    pickNumber: state.pickNumber,
    cardNumber,
    cardName: card?.name ?? "Unknown",
    cardTide: card === undefined ? "Neutral" : cardAccentTide(card),
    offerCards: state.currentOffer,
    poolRemaining: countRemainingCards(state.remainingCopiesByCard),
    uniqueCardsRemaining: Object.keys(state.remainingCopiesByCard).length,
  });

  state.pickNumber += 1;
  state.sitePicksCompleted += 1;

  if (state.sitePicksCompleted >= SITE_PICKS) {
    return true;
  }

  state.currentOffer = buildOffer({
    remainingCopiesByCard: state.remainingCopiesByCard,
    pickNumber: state.pickNumber,
    packSize: config.packSize,
  });

  return state.currentOffer.length < config.packSize;
}

/** Finalize a draft site visit. Log the cards drafted during this visit. */
export function completeDraftSite(
  state: DraftState,
  draftedCardNumbers: readonly number[] = [],
): void {
  logEvent("draft_site_completed", {
    cardsDrafted: [...draftedCardNumbers],
    picksCompleted: state.sitePicksCompleted,
    poolRemaining: countRemainingCards(state.remainingCopiesByCard),
    uniqueCardsRemaining: Object.keys(state.remainingCopiesByCard).length,
  });
}
