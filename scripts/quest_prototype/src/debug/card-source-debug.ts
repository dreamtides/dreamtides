import type { CardData, FrozenCardData } from "../types/cards";
import type { ResolvedDreamcallerPackage } from "../types/content";
import type {
  CardSourceDebugEntry,
  CardSourceDebugState,
  CardSourceDebugSurface,
} from "../types/quest";

function sortTides(tides: readonly string[]): string[] {
  return [...tides].sort((left, right) => left.localeCompare(right));
}

function buildCardSourceDebugEntry(
  card: CardData | FrozenCardData,
  resolvedPackage: ResolvedDreamcallerPackage | null,
): CardSourceDebugEntry {
  const mandatoryTides = new Set(resolvedPackage?.mandatoryTides ?? []);
  const optionalTides = new Set(resolvedPackage?.optionalSubset ?? []);
  const matchedMandatoryTides = card.tides.filter((tide) => mandatoryTides.has(tide));
  const matchedOptionalTides = card.tides.filter((tide) => optionalTides.has(tide));

  return {
    cardNumber: card.cardNumber,
    cardName: card.name,
    cardTides: sortTides(card.tides),
    matchedMandatoryTides: sortTides(matchedMandatoryTides),
    matchedOptionalTides: sortTides(matchedOptionalTides),
    isFallback:
      matchedMandatoryTides.length === 0 && matchedOptionalTides.length === 0,
  };
}

export function buildCardSourceDebugState(
  screenLabel: string,
  surface: CardSourceDebugSurface,
  cards: readonly (CardData | FrozenCardData)[],
  resolvedPackage: ResolvedDreamcallerPackage | null,
): CardSourceDebugState | null {
  if (cards.length === 0) {
    return null;
  }

  return {
    screenLabel,
    surface,
    entries: cards.map((card) => buildCardSourceDebugEntry(card, resolvedPackage)),
  };
}
