/** Algorithm used to seed initial packs from the card pool. */
export type SeedingAlgorithm = "random" | "balanced";

/** Configuration constants for the cube draft engine. */
export interface DraftConfig {
  /** Total number of seats in the draft (1 player + bots). */
  seatCount: number;
  /** Number of cards in each pack. */
  packSize: number;
  /** Number of rounds per pool. */
  roundsPerPool: number;
  /** Number of picks per round (one per seat). */
  picksPerRound: number;
  /** Number of tides in the fitness vector (excludes Wild). */
  tideCount: number;
  /** Scoring weight for preference alignment. */
  preferenceWeight: number;
  /** Scoring weight for openness/signal. */
  signalWeight: number;
  /** Scoring weight for rarity. */
  rarityWeight: number;
  /** Probability that the bot picks the highest-scored card. */
  aiOptimality: number;
  /** Multiplier applied to fitness when updating preference vector. */
  learningRate: number;
  /** Number of recent packs to consider for openness computation. */
  opennessWindow: number;
  /** Numeric value assigned to each rarity for scoring. */
  rarityValues: Readonly<Record<string, number>>;
  /** Algorithm for distributing cards into packs: "random" or "balanced" (even tide distribution). */
  seedingAlgorithm: SeedingAlgorithm;
}

/** Per-seat agent state, tracking preferences and picks. */
export interface AgentState {
  /** 7-element preference vector tracking affinity for each named tide. */
  preference: number[];
  /** History of supply signal vectors from recent packs (up to opennessWindow entries). */
  opennessHistory: number[][];
  /** Card numbers picked by this agent so far. */
  picks: number[];
}

/** Full persistent draft state, survives across dreamscapes. */
export interface DraftState {
  /** Card numbers remaining in the pool (consumed without replacement). */
  pool: number[];
  /** 10 packs, one per seat. Each pack is an array of card numbers. */
  packs: number[][];
  /** 10 agent states, one per seat. Seat 0 = player. */
  agents: AgentState[];
  /** Current round within the pool (0-2). */
  currentRound: number;
  /** Current pick within the round (0-9). */
  currentPick: number;
  /** Total picks completed in this pool (0-30). */
  totalPicks: number;
  /** Whether the draft is currently active (packs are dealt and picks are in progress). */
  isActive: boolean;
  /** Number of player picks completed in the current draft site visit. */
  sitePicksCompleted: number;
}
