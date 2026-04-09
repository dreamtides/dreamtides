import type { CardData, Tide, Rarity } from "../types/cards";

/** Tide circle: Bloom -> Arc -> Ignite -> Pact -> Umbra -> Rime -> Surge -> Bloom. */
export const TIDE_CIRCLE_ORDER: readonly Tide[] = [
  "Bloom",
  "Arc",
  "Ignite",
  "Pact",
  "Umbra",
  "Rime",
  "Surge",
] as const;

/** Compute shortest distance on the 7-tide circle. Returns -1 for Neutral. */
export function tideCircleDistance(a: Tide, b: Tide): number {
  const idxA = TIDE_CIRCLE_ORDER.indexOf(a);
  const idxB = TIDE_CIRCLE_ORDER.indexOf(b);
  if (idxA === -1 || idxB === -1) return -1;
  const diff = Math.abs(idxA - idxB);
  return Math.min(diff, 7 - diff);
}

/** Returns the two neighboring tides on the circle. Returns empty for Neutral. */
export function adjacentTides(tide: Tide): Tide[] {
  const idx = TIDE_CIRCLE_ORDER.indexOf(tide);
  if (idx === -1) return [];
  const prev = TIDE_CIRCLE_ORDER[(idx + 6) % 7];
  const next = TIDE_CIRCLE_ORDER[(idx + 1) % 7];
  return [prev, next];
}

/** Gray circle SVG used as fallback icon for the Neutral tide. */
const NEUTRAL_TIDE_FALLBACK =
  "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='64' height='64'%3E%3Ccircle cx='32' cy='32' r='28' fill='%239ca3af'/%3E%3C/svg%3E";

/** The 7 named tides (excludes Neutral), for iteration. */
export const NAMED_TIDES: readonly Tide[] = [
  "Bloom",
  "Arc",
  "Ignite",
  "Pact",
  "Umbra",
  "Rime",
  "Surge",
] as const;

/** Theme color hex value for each tide. */
export const TIDE_COLORS: Readonly<Record<Tide, string>> = {
  Bloom: "#10b981",
  Arc: "#f59e0b",
  Ignite: "#ef4444",
  Pact: "#ec4899",
  Umbra: "#8b5cf6",
  Rime: "#3b82f6",
  Surge: "#06b6d4",
  Neutral: "#9ca3af",
};

/** Display color hex value for each rarity. */
export const RARITY_COLORS: Readonly<Record<Rarity, string>> = {
  Common: "#ffffff",
  Uncommon: "#10b981",
  Rare: "#3b82f6",
  Legendary: "#a855f7",
  Starter: "#d4a017",
};

/** Returns the URL path for a card's image. */
export function cardImageUrl(cardNumber: number): string {
  return `/cards/${String(cardNumber)}.webp`;
}

/** Returns the URL path for a tide's icon. Neutral returns an inline SVG fallback. */
export function tideIconUrl(tide: Tide): string {
  if (tide === "Neutral") {
    return NEUTRAL_TIDE_FALLBACK;
  }
  return `/tides/${tide}.png`;
}

/**
 * Fetches card-data.json and returns a Map keyed by card number.
 * The JSON file is served from the public directory at /card-data.json.
 */
export async function loadCardDatabase(): Promise<Map<number, CardData>> {
  const response = await fetch("/card-data.json");
  if (!response.ok) {
    throw new Error(
      `Failed to load card data: ${String(response.status)} ${response.statusText}`,
    );
  }
  const cards = (await response.json()) as CardData[];
  const database = new Map<number, CardData>();
  for (const card of cards) {
    database.set(card.cardNumber, card);
  }
  return database;
}
