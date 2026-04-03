/** Configuration for the Tide Current draft algorithm. */
export interface DraftConfig {
  /** Number of cards shown per pick. */
  packSize: number;
  /** Minimum affinity for any tide. */
  baseAffinity: number;
  /** First pick where focus > 0. */
  focusStartPick: number;
  /** Focus increase per pick after focusStartPick. */
  focusRate: number;
  /** Recency decay per pick position (1.0 = no decay). */
  decayFactor: number;
  /** Affinity contribution from allied-tide (distance 1) drafted cards. */
  allySimilarity: number;
  /** Affinity contribution from distance-2 drafted cards. */
  distance2Similarity: number;
  /** Affinity contribution from distance-3 drafted cards. */
  distance3Similarity: number;
  /** Affinity added to all core tides per Neutral card drafted. */
  neutralDraftContribution: number;
  /** Neutral's affinity as fraction of highest core tide affinity. */
  neutralAffinityFactor: number;
}

/** Persistent draft state, survives across dreamscape visits. */
export interface DraftState {
  /** Card numbers remaining in the pool (shrinks by packSize each pick). */
  pool: number[];
  /** Player's drafted card numbers, ordered newest first. */
  draftedCards: number[];
  /** Current pack of card numbers being offered. */
  currentPack: number[];
  /** 1-indexed pick counter across the entire quest. */
  pickNumber: number;
  /** Number of player picks completed in the current draft site visit. */
  sitePicksCompleted: number;
}
