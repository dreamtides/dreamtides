import type { CardData, Tide } from "../types/cards";

type NamedTide = Exclude<Tide, "Neutral">;

export interface PoolOptions {
  excludedTides?: readonly NamedTide[];
  consumedCardNumbers?: ReadonlySet<number>;
}

function sortCards(cards: CardData[]): CardData[] {
  return [...cards].sort((left, right) => left.cardNumber - right.cardNumber);
}

function isExcludedTide(card: CardData, excludedTides: readonly NamedTide[]): boolean {
  return excludedTides.includes(card.tide as NamedTide);
}

function offerablePool(
  cardDatabase: ReadonlyMap<number, CardData>,
  options: PoolOptions = {},
): CardData[] {
  const excludedTides = options.excludedTides ?? [];
  return sortCards(
    Array.from(cardDatabase.values()).filter(
      (card) => card.rarity !== "Starter" && !isExcludedTide(card, excludedTides),
    ),
  );
}

export function findStarterCards(
  cardDatabase: ReadonlyMap<number, CardData>,
): CardData[] {
  return sortCards(
    Array.from(cardDatabase.values()).filter((card) => card.rarity === "Starter"),
  );
}

export function offerableCards(
  cardDatabase: ReadonlyMap<number, CardData>,
  options: PoolOptions = {},
): CardData[] {
  return offerablePool(cardDatabase, options);
}

export function randomStartingTideCandidates(
  cardDatabase: ReadonlyMap<number, CardData>,
  tide: NamedTide,
): CardData[] {
  return offerablePool(cardDatabase).filter((card) => card.tide === tide);
}

export function neutralStartingCandidates(
  cardDatabase: ReadonlyMap<number, CardData>,
): CardData[] {
  return offerablePool(cardDatabase).filter(
    (card) => card.tide === "Neutral" && card.rarity !== "Legendary",
  );
}

export function draftPoolCards(
  cardDatabase: ReadonlyMap<number, CardData>,
  options: PoolOptions = {},
): CardData[] {
  const consumedCardNumbers = options.consumedCardNumbers ?? new Set<number>();
  return offerablePool(cardDatabase, options).filter(
    (card) => !consumedCardNumbers.has(card.cardNumber),
  );
}
