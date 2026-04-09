import { NAMED_TIDES } from "./card-database";
import type { NamedTide } from "../types/cards";

const NAMED_TIDE_CIRCLE = NAMED_TIDES as readonly NamedTide[];

function tideIndex(tide: NamedTide): number {
  const index = NAMED_TIDE_CIRCLE.indexOf(tide);
  if (index === -1) {
    throw new Error(`Unknown named tide: ${tide}`);
  }
  return index;
}

/** Returns the named tide counter-clockwise from the given tide. */
export function leftNeighbor(tide: NamedTide): NamedTide {
  const index = tideIndex(tide);
  return NAMED_TIDE_CIRCLE[
    (index + NAMED_TIDE_CIRCLE.length - 1) % NAMED_TIDE_CIRCLE.length
  ];
}

/** Returns the named tide clockwise from the given tide. */
export function rightNeighbor(tide: NamedTide): NamedTide {
  const index = tideIndex(tide);
  return NAMED_TIDE_CIRCLE[(index + 1) % NAMED_TIDE_CIRCLE.length];
}

/** Returns a stable unordered key for a two-tide pair. */
export function normalizedPairKey(pair: readonly [NamedTide, NamedTide]): string {
  const left = tideIndex(pair[0]);
  const right = tideIndex(pair[1]);
  if (left <= right) {
    return `${pair[0]}/${pair[1]}`;
  }
  return `${pair[1]}/${pair[0]}`;
}

/** Returns whether a pair contains the given tide. */
export function pairContainsTide(
  pair: readonly [NamedTide, NamedTide],
  tide: NamedTide,
): boolean {
  return pair[0] === tide || pair[1] === tide;
}
