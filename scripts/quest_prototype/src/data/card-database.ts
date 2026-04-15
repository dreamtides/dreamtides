import type { PackageTideId } from "../types/content";
import type { CardData, Tide, Rarity } from "../types/cards";

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
  Starter: "#f59e0b",
  Common: "#ffffff",
  Uncommon: "#10b981",
  Rare: "#3b82f6",
  Legendary: "#a855f7",
};

const PACKAGE_TIDE_ACCENT_PALETTE: readonly Tide[] = NAMED_TIDES;

/** Returns the URL path for a card's image. */
export function cardImageUrl(cardNumber: number): string {
  return `/cards/${String(cardNumber)}.webp`;
}

/** Returns a stable accent tide for a hidden package tide id. */
export function packageTideAccent(packageTideId: PackageTideId): Tide {
  if (packageTideId in TIDE_COLORS) {
    return packageTideId as Tide;
  }

  if (packageTideId.startsWith("accent:")) {
    const accentTide = packageTideId.slice("accent:".length);
    if (accentTide in TIDE_COLORS) {
      return accentTide as Tide;
    }
  }

  let hash = 0;
  for (const char of packageTideId) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return PACKAGE_TIDE_ACCENT_PALETTE[hash % PACKAGE_TIDE_ACCENT_PALETTE.length];
}

export function isStarterCard(card: Pick<CardData, "rarity">): boolean {
  return card.rarity === "Starter";
}

/** Returns the accent tide used for legacy display surfaces. */
export function cardAccentTide(card: Pick<CardData, "tides">): Tide {
  if (card.tides.length === 0) {
    return "Neutral";
  }
  return packageTideAccent(card.tides[0]);
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
