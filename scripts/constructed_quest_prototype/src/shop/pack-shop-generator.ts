import type { CardData, Tide } from "../types/cards";
import type { DeckEntry, PackShopSlot } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { adjacentTides, NAMED_TIDES } from "../data/card-database";
import { countDeckTides, tideWeight, weightedSample } from "../data/tide-weights";

/** Special (non-tide) pack types. */
const SPECIAL_PACK_TYPES = ["alliance", "removal", "aggro", "events"] as const;

/** Keywords that identify removal cards. */
const REMOVAL_KEYWORDS = ["dissolve", "prevent", "destroy", "banish", "remove"];

/** Picks a tide weighted toward the player's pool composition. */
function pickWeightedTide(
  poolTideCounts: Map<Tide, number>,
  startingTides: Tide[],
): Tide {
  // Seed starting tides if not yet present
  for (const tide of startingTides) {
    if (!poolTideCounts.has(tide)) {
      poolTideCounts.set(tide, 1);
    }
  }

  const tides = [...NAMED_TIDES];
  const totalWeight = tides.reduce(
    (sum, t) => sum + tideWeight(t, poolTideCounts),
    0,
  );
  let roll = Math.random() * totalWeight;

  for (const t of tides) {
    roll -= tideWeight(t, poolTideCounts);
    if (roll <= 0) return t;
  }

  return tides[tides.length - 1];
}

/** Generates cards for a tide pack: 4 cards from the chosen tide. */
function generateTidePackCards(
  cardDatabase: Map<number, CardData>,
  tide: Tide,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide === tide,
  );
  if (candidates.length === 0) return [];
  return weightedSample(candidates, 4, () => 1);
}

/** Generates cards for an alliance pack: 4 cards from a tide and its neighbors. */
function generateAlliancePackCards(
  cardDatabase: Map<number, CardData>,
  tide: Tide,
): CardData[] {
  const neighbors = adjacentTides(tide);
  const allowedTides = new Set<Tide>([tide, ...neighbors]);
  const candidates = Array.from(cardDatabase.values()).filter((c) =>
    allowedTides.has(c.tide),
  );
  if (candidates.length === 0) return [];
  return weightedSample(candidates, 4, () => 1);
}

/** Generates cards for a removal pack: 4 cards whose text contains removal keywords. */
function generateRemovalPackCards(
  cardDatabase: Map<number, CardData>,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter((c) => {
    const text = c.renderedText.toLowerCase();
    return REMOVAL_KEYWORDS.some((kw) => text.includes(kw));
  });
  if (candidates.length === 0) return [];
  return weightedSample(candidates, 4, () => 1);
}

/** Generates cards for an aggro pack: 4 Character cards costing 0-3. */
function generateAggroPackCards(
  cardDatabase: Map<number, CardData>,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) =>
      c.cardType === "Character" &&
      c.energyCost !== null &&
      c.energyCost <= 3,
  );
  if (candidates.length === 0) return [];
  return weightedSample(candidates, 4, () => 1);
}

/** Generates cards for an events pack: 4 Event cards. */
function generateEventsPackCards(
  cardDatabase: Map<number, CardData>,
): CardData[] {
  const candidates = Array.from(cardDatabase.values()).filter(
    (c) => c.cardType === "Event",
  );
  if (candidates.length === 0) return [];
  return weightedSample(candidates, 4, () => 1);
}

/**
 * Generates pack shop inventory with `config.packShopSize` packs.
 * Each slot is either a tide pack (80%) or a special pack (20%).
 */
export function generatePackShopInventory(
  cardDatabase: Map<number, CardData>,
  playerPool: ReadonlyArray<DeckEntry>,
  startingTides: Tide[],
  config: QuestConfig,
): PackShopSlot[] {
  const poolTideCounts = countDeckTides(playerPool, cardDatabase);
  const packs: PackShopSlot[] = [];

  for (let i = 0; i < config.packShopSize; i++) {
    const isSpecial = Math.random() * 100 < config.specialPackChance;

    if (isSpecial) {
      const specialType =
        SPECIAL_PACK_TYPES[
          Math.floor(Math.random() * SPECIAL_PACK_TYPES.length)
        ];

      switch (specialType) {
        case "alliance": {
          const tide = pickWeightedTide(
            new Map(poolTideCounts),
            startingTides,
          );
          const neighbors = adjacentTides(tide);
          const allianceLabel = `${tide} + ${neighbors.join(" & ")}`;
          packs.push({
            packType: "alliance",
            tide,
            alliance: allianceLabel,
            price: 125,
            cards: generateAlliancePackCards(cardDatabase, tide),
            purchased: false,
          });
          break;
        }
        case "removal":
          packs.push({
            packType: "removal",
            price: 125,
            cards: generateRemovalPackCards(cardDatabase),
            purchased: false,
          });
          break;
        case "aggro":
          packs.push({
            packType: "aggro",
            price: 100,
            cards: generateAggroPackCards(cardDatabase),
            purchased: false,
          });
          break;
        case "events":
          packs.push({
            packType: "events",
            price: 100,
            cards: generateEventsPackCards(cardDatabase),
            purchased: false,
          });
          break;
      }
    } else {
      const tide = pickWeightedTide(new Map(poolTideCounts), startingTides);
      packs.push({
        packType: "tide",
        tide,
        price: 100,
        cards: generateTidePackCards(cardDatabase, tide),
        purchased: false,
      });
    }
  }

  return packs;
}
