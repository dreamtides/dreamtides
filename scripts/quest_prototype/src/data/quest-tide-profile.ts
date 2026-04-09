import { leftNeighbor, rightNeighbor } from "./tide-circle";
import { NAMED_TIDES } from "./card-database";
import type { CardData, NamedTide, Tide } from "../types/cards";

type TideRecord = Record<Tide, number>;

type ProfileContributionMap = Record<
  "baseline" | "startingTide" | "neighbors" | "deck" | "dreamcaller" | "crystals" | "recentDraftPicks",
  TideRecord
>;

type QuestTideProfileCard = Pick<CardData, "rarity" | "tide">;

type QuestTideProfileCardDatabase = ReadonlyMap<number, QuestTideProfileCard>;

type QuestTideProfileDreamcaller = {
  tides: readonly [NamedTide, NamedTide];
} | null;

type QuestTideProfileCardNumberLike = number | { cardNumber: number };

const ALL_TIDES: readonly Tide[] = [
  ...NAMED_TIDES,
  "Neutral",
] as const;

const BASELINE_WEIGHT = 1;
const NEUTRAL_BASELINE_WEIGHT = 0.25;
const STARTING_TIDE_WEIGHT = 5;
const NEIGHBOR_WEIGHT = 3;
const NAMED_DECK_WEIGHT = 2;
const NEUTRAL_DECK_WEIGHT = 0.25;
const DREAMCALLER_WEIGHT = 2.5;
const CRYSTAL_WEIGHT = 1.5;
const RECENT_PICK_WEIGHT = 1.25;
const RECENT_NEUTRAL_PICK_WEIGHT = 0.2;
const RECENT_PICK_DECAY = 0.85;

function createTideRecord(value = 0): TideRecord {
  return {
    Bloom: value,
    Arc: value,
    Ignite: value,
    Pact: value,
    Umbra: value,
    Rime: value,
    Surge: value,
    Neutral: value,
  };
}

function createContributionMap(): ProfileContributionMap {
  return {
    baseline: createTideRecord(),
    startingTide: createTideRecord(),
    neighbors: createTideRecord(),
    deck: createTideRecord(),
    dreamcaller: createTideRecord(),
    crystals: createTideRecord(),
    recentDraftPicks: createTideRecord(),
  };
}

function createQuestTideProfileWeights(): TideRecord {
  return createTideRecord();
}

function addContribution(record: TideRecord, tide: Tide, amount: number): void {
  record[tide] += amount;
}

function extractCardNumber(item: QuestTideProfileCardNumberLike): number {
  return typeof item === "number" ? item : item.cardNumber;
}

function isStarterCard(card: QuestTideProfileCard): boolean {
  return card.rarity === "Starter";
}

function addAllTideBaseline(contributions: ProfileContributionMap): void {
  for (const tide of NAMED_TIDES) {
    addContribution(contributions.baseline, tide, BASELINE_WEIGHT);
  }
  addContribution(contributions.baseline, "Neutral", NEUTRAL_BASELINE_WEIGHT);
}

function addStartingTideWeight(
  contributions: ProfileContributionMap,
  startingTide: NamedTide | null,
): void {
  if (!startingTide) {
    return;
  }

  addContribution(contributions.startingTide, startingTide, STARTING_TIDE_WEIGHT);
  addContribution(contributions.neighbors, leftNeighbor(startingTide), NEIGHBOR_WEIGHT);
  addContribution(contributions.neighbors, rightNeighbor(startingTide), NEIGHBOR_WEIGHT);
}

function addDeckContributions(
  contributions: ProfileContributionMap,
  deck: ReadonlyArray<{ cardNumber: number }>,
  cardDatabase: QuestTideProfileCardDatabase,
): void {
  for (const entry of deck) {
    const card = cardDatabase.get(entry.cardNumber);
    if (!card || isStarterCard(card)) {
      continue;
    }

    if (card.tide === "Neutral") {
      addContribution(contributions.deck, "Neutral", NEUTRAL_DECK_WEIGHT);
      continue;
    }

    addContribution(contributions.deck, card.tide, NAMED_DECK_WEIGHT);
  }
}

function addDreamcallerContributions(
  contributions: ProfileContributionMap,
  dreamcaller: QuestTideProfileDreamcaller,
): void {
  if (!dreamcaller) {
    return;
  }

  addContribution(contributions.dreamcaller, dreamcaller.tides[0], DREAMCALLER_WEIGHT);
  addContribution(contributions.dreamcaller, dreamcaller.tides[1], DREAMCALLER_WEIGHT);
}

function addCrystalContributions(
  contributions: ProfileContributionMap,
  tideCrystals: Readonly<Record<Tide, number>>,
): void {
  for (const tide of ALL_TIDES) {
    const amount = tideCrystals[tide] ?? 0;
    if (amount > 0) {
      addContribution(contributions.crystals, tide, amount * CRYSTAL_WEIGHT);
    }
  }
}

function addRecentDraftPickContributions(
  contributions: ProfileContributionMap,
  recentDraftPicks: ReadonlyArray<QuestTideProfileCardNumberLike>,
  cardDatabase: QuestTideProfileCardDatabase,
): void {
  for (let index = 0; index < recentDraftPicks.length; index++) {
    const card = cardDatabase.get(extractCardNumber(recentDraftPicks[index]));
    if (!card || isStarterCard(card)) {
      continue;
    }

    const decay = Math.pow(RECENT_PICK_DECAY, index);
    if (card.tide === "Neutral") {
      addContribution(
        contributions.recentDraftPicks,
        "Neutral",
        RECENT_NEUTRAL_PICK_WEIGHT * decay,
      );
      continue;
    }

    addContribution(
      contributions.recentDraftPicks,
      card.tide,
      RECENT_PICK_WEIGHT * decay,
    );
  }
}

function sumContributions(contributions: ProfileContributionMap): TideRecord {
  const weights = createQuestTideProfileWeights();
  for (const tide of ALL_TIDES) {
    weights[tide] =
      contributions.baseline[tide] +
      contributions.startingTide[tide] +
      contributions.neighbors[tide] +
      contributions.deck[tide] +
      contributions.dreamcaller[tide] +
      contributions.crystals[tide] +
      contributions.recentDraftPicks[tide];
  }
  return weights;
}

/**
 * Quest-wide tide affinity profile used by tide-aware quest generators.
 */
export interface QuestTideProfile {
  weights: TideRecord;
  contributions: ProfileContributionMap;
}

/**
 * Computes the shared quest tide profile from the chosen starting tide,
 * current deck, crystal balance, dreamcaller, and recent draft picks.
 */
export function computeQuestTideProfile({
  startingTide,
  deck,
  cardDatabase,
  dreamcaller,
  tideCrystals,
  recentDraftPicks,
}: {
  startingTide: NamedTide | null;
  deck: ReadonlyArray<{ cardNumber: number }>;
  cardDatabase: QuestTideProfileCardDatabase;
  dreamcaller: QuestTideProfileDreamcaller;
  tideCrystals: Readonly<Record<Tide, number>>;
  recentDraftPicks: ReadonlyArray<QuestTideProfileCardNumberLike>;
}): QuestTideProfile {
  const contributions = createContributionMap();

  addAllTideBaseline(contributions);
  addStartingTideWeight(contributions, startingTide);
  addDeckContributions(contributions, deck, cardDatabase);
  addDreamcallerContributions(contributions, dreamcaller);
  addCrystalContributions(contributions, tideCrystals);
  addRecentDraftPickContributions(contributions, recentDraftPicks, cardDatabase);

  return {
    weights: sumContributions(contributions),
    contributions,
  };
}

/** Returns the computed weight for a tide in the profile. */
export function tideProfileWeight(profile: QuestTideProfile, tide: Tide): number {
  return profile.weights[tide];
}

/** Samples cards from a pool without replacement using profile tide weights. */
export function weightedSampleByProfile<T extends { tide: Tide }>(
  pool: ReadonlyArray<T>,
  profile: QuestTideProfile,
  count: number,
): T[] {
  const remaining = pool.map((item) => ({
    item,
    weight: tideProfileWeight(profile, item.tide),
  }));
  const selected: T[] = [];

  for (let pick = 0; pick < count && remaining.length > 0; pick++) {
    const total = remaining.reduce((sum, entry) => sum + entry.weight, 0);
    let roll = Math.random() * total;
    let chosenIndex = remaining.length - 1;

    for (let index = 0; index < remaining.length; index++) {
      roll -= remaining[index].weight;
      if (roll <= 0) {
        chosenIndex = index;
        break;
      }
    }

    selected.push(remaining[chosenIndex].item);
    remaining.splice(chosenIndex, 1);
  }

  return selected;
}
