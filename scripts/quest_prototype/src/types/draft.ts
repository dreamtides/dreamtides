import type { CardData, Tide } from "./cards";

/** Configuration shared across all pack generation strategies. */
export interface DraftConfig {
  /** Number of cards shown per pick. */
  packSize: number;
}

/** Context provided to a pack generation strategy. */
export interface PackContext {
  /** Card numbers remaining in the pool. */
  pool: number[];
  /** Card database for resolving card data. */
  cardDatabase: Map<number, CardData>;
  /** Player's drafted card numbers, ordered newest first. */
  draftedCards: number[];
  /** 1-indexed pick counter across the entire quest. */
  pickNumber: number;
  /** Number of cards to include in the pack. */
  packSize: number;
}

/** Strategy for generating card packs during draft. */
export type PackStrategy =
  | { type: "tide_current" }
  | { type: "pool_bias"; featuredTides: Tide[]; featuredWeight: number };

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
  /** Strategy used to generate packs. */
  packStrategy: PackStrategy;
}
