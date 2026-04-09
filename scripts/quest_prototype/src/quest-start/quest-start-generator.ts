import { NAMED_TIDES } from "../data/card-database";
import {
  findStarterCards,
  neutralStartingCandidates,
  randomStartingTideCandidates,
} from "../data/card-pools";
import type { CardData, NamedTide } from "../types/cards";

/** Card-number groups used to initialize a quest run. */
export interface StartingDeckPlan {
  starterCardNumbers: number[];
  tideCardNumbers: number[];
  neutralCardNumbers: number[];
  deckCardNumbers: number[];
  consumedRandomCardNumbers: number[];
}

function shuffle<T>(items: readonly T[]): T[] {
  const result = [...items];
  for (let index = result.length - 1; index > 0; index--) {
    const swapIndex = Math.floor(Math.random() * (index + 1));
    [result[index], result[swapIndex]] = [result[swapIndex], result[index]];
  }
  return result;
}

function sampleCardNumbers(cards: CardData[], count: number, label: string): number[] {
  if (cards.length < count) {
    throw new Error(
      `Expected at least ${String(count)} cards for ${label}, found ${String(cards.length)}`,
    );
  }
  return shuffle(cards).slice(0, count).map((card) => card.cardNumber);
}

/** Returns 3 random distinct named tides, excluding any caller-provided tides. */
export function selectStartingTideOptions(
  excludedTides: readonly NamedTide[],
): NamedTide[] {
  const excludedSet = new Set(excludedTides);
  const pool = NAMED_TIDES.filter(
    (tide): tide is NamedTide => tide !== "Neutral" && !excludedSet.has(tide),
  );
  return shuffle(pool).slice(0, 3);
}

/** Builds the fixed 30-card starting loadout for a chosen tide. */
export function buildStartingDeckPlan(
  cardDatabase: ReadonlyMap<number, CardData>,
  startingTide: NamedTide,
): StartingDeckPlan {
  const starterCardNumbers = findStarterCards(cardDatabase).map(
    (card) => card.cardNumber,
  );
  if (starterCardNumbers.length !== 10) {
    throw new Error(
      `Expected 10 Starter cards, found ${String(starterCardNumbers.length)}`,
    );
  }

  const tideCardNumbers = sampleCardNumbers(
    randomStartingTideCandidates(cardDatabase, startingTide),
    10,
    `${startingTide} starting tide package`,
  );
  const neutralCardNumbers = sampleCardNumbers(
    neutralStartingCandidates(cardDatabase),
    10,
    "Neutral starting package",
  );

  return {
    starterCardNumbers,
    tideCardNumbers,
    neutralCardNumbers,
    deckCardNumbers: [
      ...starterCardNumbers,
      ...tideCardNumbers,
      ...neutralCardNumbers,
    ],
    consumedRandomCardNumbers: [...tideCardNumbers, ...neutralCardNumbers],
  };
}
