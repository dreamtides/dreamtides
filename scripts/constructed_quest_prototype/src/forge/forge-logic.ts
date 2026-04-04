import type { CardData, Tide } from "../types/cards";
import type { DeckEntry, ForgeRecipe } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { NAMED_TIDES } from "../data/card-database";
import {
  countDeckTides,
  excessCardsByTide,
  tideWeight,
  weightedSample,
} from "../data/tide-weights";

/**
 * Generates forge recipes based on the player's pool/deck composition.
 * Each recipe sacrifices `forgeCost` cards of one tide to gain a card
 * of a different tide.
 */
export function generateForgeRecipes(
  cardDatabase: Map<number, CardData>,
  playerPool: DeckEntry[],
  playerDeck: DeckEntry[],
  config: QuestConfig,
  isEnhanced: boolean,
): ForgeRecipe[] {
  const excess = excessCardsByTide(playerPool, playerDeck, cardDatabase);
  const deckTideCounts = countDeckTides(playerDeck, cardDatabase);

  // Count non-bane pool cards per tide (these are what can actually be sacrificed)
  const poolTideCounts = new Map<Tide, number>();
  for (const entry of playerPool) {
    if (entry.isBane) continue;
    const card = cardDatabase.get(entry.cardNumber);
    if (card) {
      poolTideCounts.set(card.tide, (poolTideCounts.get(card.tide) ?? 0) + 1);
    }
  }

  // Sort tides by excess descending
  const sortedTides = NAMED_TIDES.slice()
    .filter((t) => (poolTideCounts.get(t) ?? 0) >= config.forgeCost)
    .sort((a, b) => (excess.get(b) ?? 0) - (excess.get(a) ?? 0));

  // All non-Neutral cards for output selection
  const outputCandidates = Array.from(cardDatabase.values()).filter(
    (c) => c.tide !== "Neutral",
  );

  const recipes: ForgeRecipe[] = [];
  const usedSacrificeTides = new Set<Tide>();

  for (const sacrificeTide of sortedTides) {
    if (recipes.length >= config.forgeRecipes) break;
    if (usedSacrificeTides.has(sacrificeTide)) continue;
    usedSacrificeTides.add(sacrificeTide);

    if (isEnhanced) {
      recipes.push({
        sacrificeTide,
        sacrificeCount: config.forgeCost,
        outputCard: null,
      });
    } else {
      // Pick an output card from a different tide, weighted toward deck tides
      const eligible = outputCandidates.filter(
        (c) => c.tide !== sacrificeTide,
      );
      if (eligible.length === 0) continue;

      const picked = weightedSample(eligible, 1, (c) =>
        tideWeight(c.tide, deckTideCounts),
      );
      if (picked.length === 0) continue;

      recipes.push({
        sacrificeTide,
        sacrificeCount: config.forgeCost,
        outputCard: picked[0],
      });
    }
  }

  return recipes;
}

/**
 * Returns up to ~20 cards eligible for enhanced forge output selection.
 * Filters to cards whose tide is in the player's starting tides, then
 * samples via weighted random selection favoring deck tides.
 */
export function getForgeEligibleCards(
  cardDatabase: Map<number, CardData>,
  startingTides: Tide[],
  excludeTide?: Tide,
): CardData[] {
  const tideSet = new Set(startingTides);
  const eligible = Array.from(cardDatabase.values()).filter(
    (c) =>
      tideSet.has(c.tide) &&
      c.tide !== "Neutral" &&
      (excludeTide === undefined || c.tide !== excludeTide),
  );

  if (eligible.length <= 20) return eligible;

  // Simple random shuffle and take 20
  const shuffled = eligible.slice();
  for (let i = shuffled.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]];
  }
  return shuffled.slice(0, 20);
}
