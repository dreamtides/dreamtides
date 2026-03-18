import type { CardData, Tide } from "../types/cards";
import type { DraftConfig, DraftState, AgentState } from "../types/draft";
import { NAMED_TIDES } from "../data/card-database";
import { logEvent } from "../logging";

/** Ordered tide indices matching the fitness vector layout. */
const TIDE_INDEX: Readonly<Record<string, number>> = {
  Bloom: 0,
  Arc: 1,
  Ignite: 2,
  Pact: 3,
  Umbra: 4,
  Rime: 5,
  Surge: 6,
};

/** Default configuration for the cube draft engine. */
export const DEFAULT_DRAFT_CONFIG: Readonly<DraftConfig> = {
  seatCount: 10,
  packSize: 15,
  roundsPerPool: 3,
  picksPerRound: 10,
  tideCount: 7,
  preferenceWeight: 0.6,
  signalWeight: 0.2,
  rarityWeight: 0.2,
  aiOptimality: 0.8,
  learningRate: 3.0,
  opennessWindow: 3,
  rarityValues: {
    Common: 0.0,
    Uncommon: 0.33,
    Rare: 0.67,
    Legendary: 1.0,
  },
  seedingAlgorithm: "balanced",
};

/** Tide ordering for display sorting. */
const TIDE_ORDER: Readonly<Record<string, number>> = {
  Bloom: 0,
  Arc: 1,
  Ignite: 2,
  Pact: 3,
  Umbra: 4,
  Rime: 5,
  Surge: 6,
  Wild: 7,
};

/** Sort an array of cards by tide order without mutating the original. */
export function sortCardsByTide(cards: CardData[]): CardData[] {
  return [...cards].sort((a, b) => {
    const orderA = TIDE_ORDER[a.tide] ?? 8;
    const orderB = TIDE_ORDER[b.tide] ?? 8;
    return orderA - orderB;
  });
}

/** Number of player picks per draft site visit. */
export const SITE_PICKS = 5;

/** Compute the 7-element fitness vector for a card based on its tide. */
export function computeFitness(card: CardData): number[] {
  const fitness = new Array<number>(7).fill(0);
  if (card.tide === "Wild") {
    fitness.fill(0.15);
  } else {
    const idx = TIDE_INDEX[card.tide];
    if (idx !== undefined) {
      fitness[idx] = 1.0;
    }
  }
  return fitness;
}

/** Normalize a vector to sum to 1. Returns uniform [1/7, ...] for all-zero input. */
export function normalize(w: number[]): number[] {
  const sum = w.reduce((s, v) => s + v, 0);
  if (sum <= 0) {
    return new Array<number>(w.length).fill(1 / w.length);
  }
  return w.map((v) => v / sum);
}

/**
 * Compute the openness vector from an agent's history of supply signals.
 * Returns the element-wise mean of the stored signal vectors.
 * If history is empty, returns uniform [1/7, ...].
 */
function computeOpenness(agent: AgentState, tideCount: number): number[] {
  if (agent.opennessHistory.length === 0) {
    return new Array<number>(tideCount).fill(1 / tideCount);
  }
  const result = new Array<number>(tideCount).fill(0);
  for (const signal of agent.opennessHistory) {
    for (let i = 0; i < tideCount; i++) {
      result[i] += signal[i];
    }
  }
  const count = agent.opennessHistory.length;
  return result.map((v) => v / count);
}

/**
 * Compute the supply signal vector for a pack.
 * The signal is the normalized sum of fitness vectors of all cards in the pack.
 */
function computeSupplySignal(
  pack: number[],
  cardDatabase: Map<number, CardData>,
  tideCount: number,
): number[] {
  const signal = new Array<number>(tideCount).fill(0);
  for (const cardNum of pack) {
    const card = cardDatabase.get(cardNum);
    if (card) {
      const fitness = computeFitness(card);
      for (let i = 0; i < tideCount; i++) {
        signal[i] += fitness[i];
      }
    }
  }
  return normalize(signal);
}

/** Score a card for an AI agent using the adaptive scoring formula. */
export function scoreCard(
  card: CardData,
  agent: AgentState,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): number {
  const fitness = computeFitness(card);
  const normalizedPref = normalize(agent.preference);
  const openness = computeOpenness(agent, config.tideCount);

  const prefScore = dot(fitness, normalizedPref);
  const signalScore = dot(fitness, openness);
  const rarityValue = config.rarityValues[card.rarity] ?? 0;

  return (
    config.preferenceWeight * prefScore +
    config.signalWeight * signalScore +
    config.rarityWeight * rarityValue
  );
}

/** Dot product of two arrays. */
function dot(a: number[], b: number[]): number {
  let sum = 0;
  const len = Math.min(a.length, b.length);
  for (let i = 0; i < len; i++) {
    sum += a[i] * b[i];
  }
  return sum;
}

/** Fisher-Yates shuffle of an array in place. */
function shuffle<T>(arr: T[]): T[] {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
  return arr;
}

/** Create a fresh agent state with zero preference vector. */
function createAgentState(tideCount: number): AgentState {
  return {
    preference: new Array<number>(tideCount).fill(0),
    opennessHistory: [],
    picks: [],
  };
}

/** Count cards per tide in a collection of card numbers. */
function countByTide(
  cardNumbers: number[],
  cardDatabase: Map<number, CardData>,
): Record<string, number> {
  const counts: Record<string, number> = {};
  for (const tide of [...NAMED_TIDES, "Wild" as Tide]) {
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

/** Create initial DraftState from all non-Special cards. */
export function initializeDraftState(
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): DraftState {
  const pool = Array.from(cardDatabase.keys());

  logEvent("draft_pool_initialized", {
    poolSize: pool.length,
    cardCountByTide: countByTide(pool, cardDatabase),
  });

  return {
    pool: shuffle([...pool]),
    packs: Array.from({ length: config.seatCount }, () => []),
    agents: Array.from({ length: config.seatCount }, () =>
      createAgentState(config.tideCount),
    ),
    currentRound: 0,
    currentPick: 0,
    totalPicks: 0,
    isActive: false,
    sitePicksCompleted: 0,
  };
}

/** Create a fresh pool from all cards. Reset round/pick counters. Preserve bot preference vectors but clear stale openness history. */
export function refreshPool(
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  const pool = Array.from(cardDatabase.keys());
  state.pool = shuffle([...pool]);
  state.packs = Array.from({ length: config.seatCount }, () => []);
  state.currentRound = 0;
  state.currentPick = 0;
  state.totalPicks = 0;
  state.isActive = false;

  // Clear stale openness history from the previous pool while preserving
  // preference vectors so bots maintain their tide affinities.
  for (const agent of state.agents) {
    agent.opennessHistory = [];
  }

  logEvent("draft_pool_refreshed", {
    poolSize: state.pool.length,
    roundNumber: state.currentRound,
  });
}

/** Draw 10 packs of 15 from the pool (uniform random, without replacement). */
export function dealRound(
  state: DraftState,
  cardDatabase?: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  if (config.seedingAlgorithm === "balanced" && cardDatabase) {
    dealRoundBalanced(state, cardDatabase, config);
  } else {
    dealRoundRandom(state, config);
  }

  state.isActive = true;

  logEvent("draft_round_started", {
    roundNumber: state.currentRound,
    packsDealCount: config.seatCount,
  });
}

/** Random deal: draw sequentially from shuffled pool and distribute evenly. */
function dealRoundRandom(
  state: DraftState,
  config: DraftConfig,
): void {
  const totalNeeded = config.seatCount * config.packSize;
  const available = Math.min(totalNeeded, state.pool.length);

  const drawn = state.pool.splice(0, available);

  const packSize = Math.floor(available / config.seatCount);
  const remainder = available % config.seatCount;
  state.packs = [];
  let offset = 0;
  for (let i = 0; i < config.seatCount; i++) {
    const size = packSize + (i < remainder ? 1 : 0);
    state.packs.push(drawn.slice(offset, offset + size));
    offset += size;
  }
}

/** Balanced deal: distribute cards evenly across tides within each pack. */
function dealRoundBalanced(
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig,
): void {
  const totalNeeded = config.seatCount * config.packSize;
  const available = Math.min(totalNeeded, state.pool.length);

  const drawn = state.pool.splice(0, available);

  // Group drawn cards by tide
  const byTide = new Map<string, number[]>();
  const ungrouped: number[] = [];
  for (const cardNum of drawn) {
    const card = cardDatabase.get(cardNum);
    if (!card) {
      ungrouped.push(cardNum);
      continue;
    }
    const tide = card.tide;
    if (!byTide.has(tide)) {
      byTide.set(tide, []);
    }
    byTide.get(tide)!.push(cardNum);
  }

  // Shuffle each tide group
  for (const group of byTide.values()) {
    shuffle(group);
  }
  shuffle(ungrouped);

  const tideKeys = [...byTide.keys()];
  shuffle(tideKeys);
  const targetPackSize = Math.floor(available / config.seatCount);
  const extraRemainder = available % config.seatCount;

  const numPacks = config.seatCount;
  const packTargets = Array.from({ length: numPacks }, (_, i) =>
    targetPackSize + (i < extraRemainder ? 1 : 0),
  );

  // Pre-compute allocations: for each tide, compute how many cards go
  // into each pack. Distribute floor(tideCount/numPacks) to each pack,
  // then add remainders one at a time to the packs with the most room.
  const allocations: number[][] = Array.from(
    { length: numPacks },
    () => new Array<number>(tideKeys.length).fill(0),
  );

  for (let t = 0; t < tideKeys.length; t++) {
    const tideTotal = byTide.get(tideKeys[t])!.length;
    const base = Math.floor(tideTotal / numPacks);
    let rem = tideTotal - base * numPacks;

    for (let p = 0; p < numPacks; p++) {
      allocations[p][t] = base;
    }

    // Distribute remainder cards to packs that have the most capacity left
    while (rem > 0) {
      // Find pack with most remaining capacity
      let bestPack = -1;
      let bestRoom = -1;
      for (let p = 0; p < numPacks; p++) {
        const allocated = allocations[p].reduce((s, v) => s + v, 0);
        const room = packTargets[p] - allocated;
        if (room > bestRoom) {
          bestRoom = room;
          bestPack = p;
        }
      }
      if (bestPack === -1 || bestRoom <= 0) break;
      allocations[bestPack][t] += 1;
      rem -= 1;
    }
  }

  // Build packs using the computed allocations
  state.packs = [];
  for (let p = 0; p < numPacks; p++) {
    const pack: number[] = [];
    for (let t = 0; t < tideKeys.length; t++) {
      const group = byTide.get(tideKeys[t])!;
      const count = allocations[p][t];
      for (let j = 0; j < count && group.length > 0; j++) {
        pack.push(group.pop()!);
      }
    }
    state.packs.push(pack);
  }

  // Fill any remaining slots from leftover cards
  const leftover = [...ungrouped, ...[...byTide.values()].flatMap((g) => g)];
  shuffle(leftover);
  for (let p = 0; p < numPacks; p++) {
    while (state.packs[p].length < packTargets[p] && leftover.length > 0) {
      state.packs[p].push(leftover.pop()!);
    }
  }
}

/** Player picks a specific card from seat 0's current pack. Returns true if the card was found and picked. */
export function playerPick(
  cardNumber: number,
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): boolean {
  const pack = state.packs[0];
  const cardIndex = pack.indexOf(cardNumber);
  if (cardIndex === -1) {
    return false;
  }

  const card = cardDatabase.get(cardNumber);
  const packContents = [...pack];

  // Remove card from pack
  pack.splice(cardIndex, 1);
  // Add to player's picks
  state.agents[0].picks.push(cardNumber);

  // Update player preference vector
  if (card) {
    const fitness = computeFitness(card);
    for (let i = 0; i < config.tideCount; i++) {
      state.agents[0].preference[i] += config.learningRate * fitness[i];
    }
  }

  // Update player openness history
  const signal = computeSupplySignal(
    packContents,
    cardDatabase,
    config.tideCount,
  );
  state.agents[0].opennessHistory.push(signal);
  if (state.agents[0].opennessHistory.length > config.opennessWindow) {
    state.agents[0].opennessHistory.shift();
  }

  logEvent("draft_pick_player", {
    pickNumber: state.currentPick,
    cardNumber,
    cardName: card?.name ?? "Unknown",
    cardTide: card?.tide ?? "Wild",
    cardsAvailableCount: packContents.length,
    packContents: packContents,
  });
  return true;
}

/** AI bot picks a card from its current pack. */
export function botPick(
  seatIndex: number,
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  const pack = state.packs[seatIndex];
  if (pack.length === 0) {
    return; // Empty pack; no-op
  }

  const agent = state.agents[seatIndex];

  // Update openness history before scoring
  const signal = computeSupplySignal(pack, cardDatabase, config.tideCount);
  agent.opennessHistory.push(signal);
  if (agent.opennessHistory.length > config.opennessWindow) {
    agent.opennessHistory.shift();
  }

  // Score all cards in the pack
  const scored: Array<{ cardNumber: number; score: number }> = [];
  for (const cardNum of pack) {
    const card = cardDatabase.get(cardNum);
    if (card) {
      scored.push({ cardNumber: cardNum, score: scoreCard(card, agent, config) });
    }
  }

  if (scored.length === 0) {
    return;
  }

  // Sort by score descending
  scored.sort((a, b) => b.score - a.score);

  // 80% chance of picking the best card, 20% random
  let chosen: { cardNumber: number; score: number };
  if (Math.random() < config.aiOptimality) {
    chosen = scored[0];
  } else {
    chosen = scored[Math.floor(Math.random() * scored.length)];
  }

  // Remove chosen card from pack
  const cardIndex = pack.indexOf(chosen.cardNumber);
  if (cardIndex !== -1) {
    pack.splice(cardIndex, 1);
  }

  // Add to agent's picks
  agent.picks.push(chosen.cardNumber);

  // Update preference vector
  const pickedCard = cardDatabase.get(chosen.cardNumber);
  if (pickedCard) {
    const fitness = computeFitness(pickedCard);
    for (let i = 0; i < config.tideCount; i++) {
      agent.preference[i] += config.learningRate * fitness[i];
    }
  }

  logEvent("draft_pick_bot", {
    seatNumber: seatIndex,
    pickNumber: state.currentPick,
    cardNumber: chosen.cardNumber,
    cardTide: pickedCard?.tide ?? "Wild",
  });
}

/** Rotate packs between seats. Packs always pass left (seat N's pack goes to seat N+1). */
export function rotatePacks(state: DraftState): void {
  const packs = state.packs;
  const count = packs.length;

  const last = packs[count - 1];
  for (let i = count - 1; i > 0; i--) {
    packs[i] = packs[i - 1];
  }
  packs[0] = last;

  logEvent("draft_packs_rotated", {
    roundNumber: state.currentRound,
    pickNumber: state.currentPick,
  });
}

/**
 * After all seats have picked, increment pick counter.
 * If round complete (10 picks), increment round and deal new packs if rounds remain.
 * If pool exhausted (30 picks), trigger refresh.
 */
export function advancePick(
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  state.currentPick += 1;
  state.totalPicks += 1;

  if (state.totalPicks >= config.roundsPerPool * config.picksPerRound) {
    // Pool exhausted -- refresh
    refreshPool(state, cardDatabase, config);
    return;
  }

  if (state.currentPick >= config.picksPerRound) {
    // Round complete -- increment round and deal new packs if rounds remain
    state.currentRound += 1;
    state.currentPick = 0;
    dealRound(state, cardDatabase, config);
  }
}

/** Prepare the state for a draft site visit (5 player picks). */
export function enterDraftSite(
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): void {
  state.sitePicksCompleted = 0;

  if (!state.isActive) {
    dealRound(state, cardDatabase, config);
  }

  logEvent("draft_site_entered", {
    picksRemainingInRound: config.picksPerRound - state.currentPick,
  });
}

/** Return the current pack available to seat 0 for display. */
export function getPlayerPack(state: DraftState): number[] {
  return state.packs[0];
}

/**
 * Process a player pick. Run all bot picks for this rotation,
 * rotate packs, and advance. Return whether the 5-pick site batch is complete.
 */
export function processPlayerPick(
  cardNumber: number,
  state: DraftState,
  cardDatabase: Map<number, CardData>,
  config: DraftConfig = DEFAULT_DRAFT_CONFIG,
): boolean {
  // Player picks -- abort if the card is not in the player's pack
  const picked = playerPick(cardNumber, state, cardDatabase, config);
  if (!picked) {
    throw new Error(
      `Card ${String(cardNumber)} is not in seat 0's current pack`,
    );
  }

  // All bots pick
  for (let seat = 1; seat < config.seatCount; seat++) {
    botPick(seat, state, cardDatabase, config);
  }

  // Rotate packs
  rotatePacks(state);

  // Advance pick counter
  advancePick(state, cardDatabase, config);

  // Track site picks
  state.sitePicksCompleted += 1;

  return state.sitePicksCompleted >= SITE_PICKS;
}

/** Finalize a draft site visit. Log the cards drafted during this visit. */
export function completeDraftSite(state: DraftState): void {
  const playerPicks = state.agents[0].picks;
  const sitePicks = playerPicks.slice(
    playerPicks.length - state.sitePicksCompleted,
  );

  logEvent("draft_site_completed", {
    cardsDrafted: sitePicks,
  });
}
