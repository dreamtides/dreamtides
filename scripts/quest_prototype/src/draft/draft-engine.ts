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

function spendShownOffer(
  remainingCopiesByCard: Record<string, number>,
  offer: number[],
): void {
  for (const cardNumber of offer) {
    const key = String(cardNumber);
    const remainingCopies = remainingCopiesByCard[key];
    if (remainingCopies === undefined) {
      continue;
    }

    if (remainingCopies <= 1) {
      delete remainingCopiesByCard[key];
    } else {
      remainingCopiesByCard[key] = remainingCopies - 1;
    }
  }
}

function revealOffer(
  state: DraftState,
  config: DraftConfig,
): boolean {
  const offer = buildOffer({
    remainingCopiesByCard: state.remainingCopiesByCard,
    pickNumber: state.pickNumber,
    packSize: config.packSize,
  });

  state.currentOffer = offer;
  if (offer.length < config.packSize) {
    return false;
  }

  spendShownOffer(state.remainingCopiesByCard, offer);
  logEvent("draft_offer_revealed", {
    pickNumber: state.pickNumber,
    offerCards: offer,
    poolRemaining: countRemainingCards(state.remainingCopiesByCard),
    uniqueCardsRemaining: countRemainingUniqueCards(state.remainingCopiesByCard),
  });
  return true;
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

export function countRemainingCards(
  remainingCopiesByCard: Record<string, number>,
): number {
  return Object.values(remainingCopiesByCard).reduce(
    (total, copies) => total + copies,
    0,
  );
}

export function countRemainingUniqueCards(
  remainingCopiesByCard: Record<string, number>,
): number {
  return Object.keys(remainingCopiesByCard).length;
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
    uniqueCardCount: countRemainingUniqueCards(remainingCopiesByCard),
    dreamcallerId: resolvedPackage.dreamcaller.id,
    selectedPackageTides: resolvedPackage.selectedTides,
    cardCountByTide: countByTide(expandedPool, cardDatabase),
  });

  return {
    remainingCopiesByCard,
    currentOffer: [],
    activeSiteId: null,
    pickNumber: 1,
    sitePicksCompleted: 0,
  };
}

/** Prepare the state for a draft site visit. Draws the first pack. */
export function enterDraftSite(
  state: DraftState,
  siteId: string,
  _cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  if (state.activeSiteId === siteId) {
    logEvent("draft_site_entered", {
      siteId,
      pickNumber: state.pickNumber,
      poolSize: countRemainingCards(state.remainingCopiesByCard),
      offerCards: state.currentOffer,
      offerAvailable: state.currentOffer.length === config.packSize,
      resumedExistingOffer: state.currentOffer.length > 0,
    });
    return;
  }

  state.activeSiteId = siteId;
  state.sitePicksCompleted = 0;
  const hasOffer = revealOffer(state, config);

  logEvent("draft_site_entered", {
    siteId,
    pickNumber: state.pickNumber,
    poolSize: countRemainingCards(state.remainingCopiesByCard),
    offerCards: state.currentOffer,
    offerAvailable: hasOffer,
    resumedExistingOffer: false,
  });
}

/** Return the current offer for display. */
export function getCurrentOffer(state: DraftState): number[] {
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
  const currentOffer = [...state.currentOffer];
  if (!currentOffer.includes(cardNumber)) {
    throw new Error(
      `Card ${String(cardNumber)} is not in the current offer`,
    );
  }

  const card = cardDatabase.get(cardNumber);

  logEvent("draft_pick_player", {
    siteId: state.activeSiteId,
    pickNumber: state.pickNumber,
    cardNumber,
    cardName: card?.name ?? "Unknown",
    cardTide: card === undefined ? "Neutral" : cardAccentTide(card),
    offerCards: currentOffer,
    poolRemaining: countRemainingCards(state.remainingCopiesByCard),
    uniqueCardsRemaining: countRemainingUniqueCards(state.remainingCopiesByCard),
  });

  state.pickNumber += 1;
  state.sitePicksCompleted += 1;

  if (state.sitePicksCompleted >= SITE_PICKS) {
    state.currentOffer = [];
    return true;
  }

  return !revealOffer(state, config);
}

/** Finalize a draft site visit. Log the cards drafted during this visit. */
export function completeDraftSite(
  state: DraftState,
  draftedCardNumbers: number[],
): void {
  logEvent("draft_site_completed", {
    siteId: state.activeSiteId,
    cardsDrafted: [...draftedCardNumbers],
    picksCompleted: state.sitePicksCompleted,
    poolRemaining: countRemainingCards(state.remainingCopiesByCard),
    uniqueCardsRemaining: countRemainingUniqueCards(state.remainingCopiesByCard),
  });
}
