import {
  leftNeighbor,
  normalizedPairKey,
  pairContainsTide,
  rightNeighbor,
} from "./tide-circle";
import {
  tideProfileWeight,
  type QuestTideProfile,
} from "./quest-tide-profile";
import type { NamedTide } from "../types/cards";
import type { Dreamcaller } from "../types/quest";

interface OfferInput {
  pool: readonly Dreamcaller[];
  startingTide: NamedTide | null;
  profile: QuestTideProfile;
}

function dreamcallerWeight(
  dreamcaller: Dreamcaller,
  profile: QuestTideProfile,
): number {
  return (
    tideProfileWeight(profile, dreamcaller.tides[0]) +
    tideProfileWeight(profile, dreamcaller.tides[1])
  ) / 2;
}

function takeWeighted(
  pool: readonly Dreamcaller[],
  profile: QuestTideProfile,
): Dreamcaller | null {
  if (pool.length === 0) {
    return null;
  }

  const total = pool.reduce(
    (sum, dreamcaller) => sum + dreamcallerWeight(dreamcaller, profile),
    0,
  );
  let roll = Math.random() * total;
  for (const dreamcaller of pool) {
    roll -= dreamcallerWeight(dreamcaller, profile);
    if (roll <= 0) {
      return dreamcaller;
    }
  }

  return pool[pool.length - 1];
}

function takeMatchingPair(
  pool: readonly Dreamcaller[],
  pair: readonly [NamedTide, NamedTide],
): Dreamcaller | null {
  const key = normalizedPairKey(pair);
  return pool.find((dreamcaller) => normalizedPairKey(dreamcaller.tides) === key) ?? null;
}

function removeByName(
  pool: readonly Dreamcaller[],
  selected: Dreamcaller,
): Dreamcaller[] {
  return pool.filter((dreamcaller) => dreamcaller.name !== selected.name);
}

/** Selects up to 3 dreamcaller offers: left fork, right fork, adaptive slot. */
export function selectOfferedDreamcallers(input: OfferInput): Dreamcaller[] {
  let remaining = [...input.pool];
  const offered: Dreamcaller[] = [];

  function add(dreamcaller: Dreamcaller | null): void {
    if (
      dreamcaller === null ||
      offered.some((existing) => existing.name === dreamcaller.name)
    ) {
      return;
    }

    offered.push(dreamcaller);
    remaining = removeByName(remaining, dreamcaller);
  }

  if (input.startingTide !== null) {
    add(takeMatchingPair(remaining, [
      leftNeighbor(input.startingTide),
      input.startingTide,
    ]));
    add(takeMatchingPair(remaining, [
      input.startingTide,
      rightNeighbor(input.startingTide),
    ]));
  }

  while (offered.length < 3 && remaining.length > 0) {
    const st = input.startingTide;
    const fallback =
      st === null
        ? null
        : remaining.find((dreamcaller) =>
            pairContainsTide(dreamcaller.tides, st),
          ) ??
          remaining.find(
            (dreamcaller) =>
              pairContainsTide(
                dreamcaller.tides,
                leftNeighbor(st),
              ) ||
              pairContainsTide(
                dreamcaller.tides,
                rightNeighbor(st),
              ),
          );

    add(fallback ?? takeWeighted(remaining, input.profile));
  }

  return offered;
}
