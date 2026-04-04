import { useMemo } from "react";

/** Configuration derived from URL parameters. */
export interface QuestConfig {
  /** Whether to use revised tides. */
  revisedTides: boolean;
  /** Number of starting tides. */
  startingTides: number;
  /** Whether tides are assigned sequentially. */
  sequentialTides: boolean;
  /** Number of initial cards in the starter deck. */
  initialCards: number;
  /** Number of neutral cards in the starter deck. */
  starterNeutral: number;
  /** Number of low-cost cards in the starter deck. */
  starterLowCost: number;
  /** Number of mid-cost cards in the starter deck. */
  starterMidCost: number;
  /** Number of high-cost cards in the starter deck. */
  starterHighCost: number;
  /** Starting essence amount. */
  startingEssence: number;
  /** Number of cards in a loot pack. */
  lootPackSize: number;
  /** Duplicate penalty percentage for 2 copies. */
  dupePenalty2: number;
  /** Duplicate penalty percentage for 3+ copies. */
  dupePenalty3: number;
  /** Percentage of pack cards that match the current theme. */
  packOnTheme: number;
  /** Percentage of pack cards from adjacent tides. */
  packAdjacent: number;
  /** Percentage of pack cards that explore new tides. */
  packExplore: number;
  /** Minimum deck size. */
  minimumDeckSize: number;
  /** Maximum deck size. */
  maximumDeckSize: number;
  /** Maximum copies of any single card. */
  maxCopies: number;
  /** Number of cards offered in the card shop. */
  cardShopSize: number;
  /** Minimum card price in essence. */
  cardPriceMin: number;
  /** Maximum card price in essence. */
  cardPriceMax: number;
  /** Base cost to reroll the card shop. */
  rerollBase: number;
  /** Increment added to reroll cost each time. */
  rerollIncrement: number;
  /** Number of packs offered in the pack shop. */
  packShopSize: number;
  /** Percentage chance for a special pack to appear. */
  specialPackChance: number;
  /** Whether ante is enabled. */
  anteEnabled: boolean;
  /** Turn on which ante escalation begins. */
  escalationTurn: number;
  /** Maximum number of cards that can be anted. */
  maxAnteCards: number;
  /** Number of forge recipes available. */
  forgeRecipes: number;
  /** Cost to use the forge. */
  forgeCost: number;
  /** Total cards shown at a draft site. */
  draftSiteTotal: number;
  /** Number of cards kept from a draft site. */
  draftSiteKeep: number;
  /** Number of options the provisioner offers. */
  provisionerOptions: number;
  /** Number of dreamcaller choices offered. */
  dreamcallerChoices: number;
  /** Number of opponent preview cards shown. */
  opponentPreviewCards: number;
  /** Essence reward for winning a battle. */
  battleEssence: number;
  /** Additional essence reward per completion level. */
  essencePerLevel: number;
  /** Essence amount from an essence site. */
  essenceSiteAmount: number;
  /** Whether to show tide cost symbols on cards. */
  showTideSymbols: boolean;
}

/** Parses an integer URL parameter with optional min/max bounds. */
export function parseIntParam(
  params: URLSearchParams,
  key: string,
  defaultValue: number,
  min?: number,
  max?: number,
): number {
  const raw = params.get(key);
  if (raw === null) return defaultValue;
  const parsed = parseInt(raw, 10);
  if (isNaN(parsed)) return defaultValue;
  let value = parsed;
  if (min !== undefined && value < min) value = min;
  if (max !== undefined && value > max) value = max;
  return value;
}

/** Parses a boolean URL parameter. Treats "false" as false, anything else as true. */
export function parseBoolParam(
  params: URLSearchParams,
  key: string,
  defaultValue: boolean,
): boolean {
  const raw = params.get(key);
  if (raw === null) return defaultValue;
  return raw !== "false";
}

/** Parses quest configuration from the current URL search parameters. */
export function getQuestConfig(): QuestConfig {
  const params = new URLSearchParams(window.location.search);

  return {
    revisedTides: parseBoolParam(params, "revisedTides", true),
    startingTides: parseIntParam(params, "startingTides", 3, 1, 8),
    sequentialTides: parseBoolParam(params, "sequentialTides", true),
    initialCards: parseIntParam(params, "initialCards", 10, 1, 30),
    starterNeutral: parseIntParam(params, "starterNeutral", 5, 0, 20),
    starterLowCost: parseIntParam(params, "starterLowCost", 4, 0, 20),
    starterMidCost: parseIntParam(params, "starterMidCost", 3, 0, 20),
    starterHighCost: parseIntParam(params, "starterHighCost", 1, 0, 20),
    startingEssence: parseIntParam(params, "startingEssence", 250, 0, 9999),
    lootPackSize: parseIntParam(params, "lootPackSize", 4, 1, 20),
    dupePenalty2: parseIntParam(params, "dupePenalty2", 50, 0, 100),
    dupePenalty3: parseIntParam(params, "dupePenalty3", 90, 0, 100),
    packOnTheme: parseIntParam(params, "packOnTheme", 60, 0, 100),
    packAdjacent: parseIntParam(params, "packAdjacent", 25, 0, 100),
    packExplore: parseIntParam(params, "packExplore", 15, 0, 100),
    minimumDeckSize: parseIntParam(params, "minimumDeckSize", 25, 1, 100),
    maximumDeckSize: parseIntParam(params, "maximumDeckSize", 50, 1, 200),
    maxCopies: parseIntParam(params, "maxCopies", 2, 1, 10),
    cardShopSize: parseIntParam(params, "cardShopSize", 4, 1, 20),
    cardPriceMin: parseIntParam(params, "cardPriceMin", 50, 0, 9999),
    cardPriceMax: parseIntParam(params, "cardPriceMax", 100, 0, 9999),
    rerollBase: parseIntParam(params, "rerollBase", 40, 0, 9999),
    rerollIncrement: parseIntParam(params, "rerollIncrement", 20, 0, 9999),
    packShopSize: parseIntParam(params, "packShopSize", 3, 1, 20),
    specialPackChance: parseIntParam(params, "specialPackChance", 20, 0, 100),
    anteEnabled: parseBoolParam(params, "anteEnabled", true),
    escalationTurn: parseIntParam(params, "escalationTurn", 6, 1, 20),
    maxAnteCards: parseIntParam(params, "maxAnteCards", 2, 0, 10),
    forgeRecipes: parseIntParam(params, "forgeRecipes", 3, 1, 20),
    forgeCost: parseIntParam(params, "forgeCost", 4, 0, 20),
    draftSiteTotal: parseIntParam(params, "draftSiteTotal", 4, 1, 20),
    draftSiteKeep: parseIntParam(params, "draftSiteKeep", 1, 1, 20),
    provisionerOptions: parseIntParam(params, "provisionerOptions", 3, 1, 20),
    dreamcallerChoices: parseIntParam(params, "dreamcallerChoices", 3, 1, 20),
    opponentPreviewCards: parseIntParam(params, "opponentPreviewCards", 3, 0, 20),
    battleEssence: parseIntParam(params, "battleEssence", 150, 0, 9999),
    essencePerLevel: parseIntParam(params, "essencePerLevel", 50, 0, 9999),
    essenceSiteAmount: parseIntParam(params, "essenceSiteAmount", 200, 0, 9999),
    showTideSymbols: parseBoolParam(params, "showTideSymbols", true),
  };
}

/** React hook that returns quest configuration from URL parameters. */
export function useQuestConfig(): QuestConfig {
  return useMemo(() => getQuestConfig(), []);
}
