import type { Tide, CardData } from "../types/cards";
import type { DraftState, PackStrategy } from "../types/draft";
import { NAMED_TIDES } from "../data/card-database";
import { computeTideAffinity, computeFocus } from "../draft/draft-engine";

/** Mono-tide convergence benchmarks from draft_algorithm.md simulation data. */
const CONVERGENCE_TABLE: ReadonlyArray<[number, number]> = [
  [1, 0.67],
  [5, 1.0],
  [10, 2.15],
  [15, 3.14],
  [20, 3.53],
  [25, 3.54],
];

/** Decay factor used by the Tide Current algorithm. */
const DECAY_FACTOR = 0.85;

/** Interpolate expected dominant cards per pack for a given pick number. */
export function expectedDominantAtPick(pickNumber: number): number {
  if (pickNumber <= CONVERGENCE_TABLE[0][0]) {
    return CONVERGENCE_TABLE[0][1];
  }
  const last = CONVERGENCE_TABLE[CONVERGENCE_TABLE.length - 1];
  if (pickNumber >= last[0]) {
    return last[1];
  }
  for (let i = 0; i < CONVERGENCE_TABLE.length - 1; i++) {
    const [p0, v0] = CONVERGENCE_TABLE[i];
    const [p1, v1] = CONVERGENCE_TABLE[i + 1];
    if (pickNumber >= p0 && pickNumber <= p1) {
      const t = (pickNumber - p0) / (p1 - p0);
      return v0 + t * (v1 - v0);
    }
  }
  return last[1];
}

/** Entry in the recency decay profile. */
export interface DecayEntry {
  cardName: string;
  tide: Tide;
  decay: number;
}

/** Debug info for the player's draft state. */
export interface DraftDebugInfo {
  draftedCards: CardData[];
  cardsByTide: Record<string, number>;
  totalCards: number;
  pickNumber: number;
  poolSize: number;
  focus: number;
  tideAffinities: Record<string, number>;
  primaryTide: Tide | null;
  secondaryTide: Tide | null;
  packStrategy: PackStrategy;
  excludedTides: Tide[];
  effectiveWeights: Record<string, number>;
  tideProbabilities: Record<string, number>;
  expectedDominant: number;
  decayProfile: DecayEntry[];
}

/** Compute featured multiplier for a tide given a pack strategy. */
function featuredMultiplier(tide: string, strategy: PackStrategy): number {
  if (strategy.type === "pool_bias" && strategy.featuredTides.includes(tide as Tide)) {
    return strategy.featuredWeight;
  }
  return 1.0;
}

/** Extract debug info from the current draft state. */
export function extractDraftDebugInfo(
  draftState: DraftState | null,
  cardDatabase: Map<number, CardData>,
  excludedTides: Tide[] = [],
): DraftDebugInfo | null {
  if (draftState === null) {
    return null;
  }

  const draftedCards: CardData[] = [];
  const cardsByTide: Record<string, number> = {};
  for (const tide of [...NAMED_TIDES, "Neutral" as Tide]) {
    cardsByTide[tide] = 0;
  }

  for (const cardNum of draftState.draftedCards) {
    const card = cardDatabase.get(cardNum);
    if (card) {
      draftedCards.push(card);
      cardsByTide[card.tide] = (cardsByTide[card.tide] ?? 0) + 1;
    }
  }

  const affinity = computeTideAffinity(
    draftState.draftedCards,
    cardDatabase,
  );
  const tideAffinities: Record<string, number> = {};
  for (const [tide, value] of affinity.entries()) {
    tideAffinities[tide] = Math.round(value * 100) / 100;
  }

  const focus = computeFocus(draftState.pickNumber);
  const roundedFocus = Math.round(focus * 100) / 100;

  const sorted = Object.entries(tideAffinities)
    .filter(([tide]) => tide !== "Neutral")
    .sort(([, a], [, b]) => b - a);
  const primaryTide: Tide | null =
    sorted.length > 0 && sorted[0][1] > 1.0 ? (sorted[0][0] as Tide) : null;
  const secondaryTide: Tide | null =
    sorted.length > 1 && sorted[1][1] > 1.0 ? (sorted[1][0] as Tide) : null;

  // Effective weights: affinity^focus * featured_multiplier
  const effectiveWeights: Record<string, number> = {};
  for (const tide of [...NAMED_TIDES, "Neutral" as Tide]) {
    const aff = affinity.get(tide) ?? 1.0;
    const weight = Math.pow(aff, focus) * featuredMultiplier(tide, draftState.packStrategy);
    effectiveWeights[tide] = Math.round(weight * 1000) / 1000;
  }

  // Tide probabilities: normalize to percentages
  const totalWeight = Object.values(effectiveWeights).reduce((s, w) => s + w, 0);
  const tideProbabilities: Record<string, number> = {};
  for (const [tide, weight] of Object.entries(effectiveWeights)) {
    tideProbabilities[tide] = totalWeight > 0
      ? Math.round((weight / totalWeight) * 1000) / 10
      : 0;
  }

  // Decay profile: last 10 cards with decay multiplier
  const decayProfile: DecayEntry[] = [];
  const maxDecay = Math.min(10, draftState.draftedCards.length);
  for (let i = 0; i < maxDecay; i++) {
    const card = cardDatabase.get(draftState.draftedCards[i]);
    if (card) {
      decayProfile.push({
        cardName: card.name,
        tide: card.tide,
        decay: Math.round(Math.pow(DECAY_FACTOR, i) * 1000) / 1000,
      });
    }
  }

  return {
    draftedCards,
    cardsByTide,
    totalCards: draftedCards.length,
    pickNumber: draftState.pickNumber,
    poolSize: draftState.pool.length,
    focus: roundedFocus,
    tideAffinities,
    primaryTide,
    secondaryTide,
    packStrategy: draftState.packStrategy,
    excludedTides,
    effectiveWeights,
    tideProbabilities,
    expectedDominant: Math.round(expectedDominantAtPick(draftState.pickNumber) * 100) / 100,
    decayProfile,
  };
}
