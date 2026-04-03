import type { CardData } from "../types/cards";
import type { TransfigurationType } from "../types/quest";

/** Color hex value for each transfiguration type. */
export const TRANSFIGURATION_COLORS: Readonly<
  Record<TransfigurationType, string>
> = {
  Viridian: "#10b981",
  Golden: "#f59e0b",
  Scarlet: "#ef4444",
  Azure: "#3b82f6",
  Bronze: "#d97706",
};

/** A prepared transfiguration offer with pre-computed preview and description. */
export interface TransfigurationOffer {
  type: TransfigurationType;
  description: string;
  previewCard: CardData;
}

/** Returns true if the card is eligible for Viridian transfiguration. */
export function isViridianEligible(card: CardData): boolean {
  return card.energyCost !== null && card.energyCost > 0;
}

/** Returns true if the card's rules text contains at least one digit. */
export function isGoldenEligible(card: CardData): boolean {
  return /\d/.test(card.renderedText);
}

/** Returns true if the card is a Character (eligible for Scarlet). */
export function isScarletEligible(card: CardData): boolean {
  return card.cardType === "Character";
}

/** Returns true if the card is an Event (eligible for Azure). */
export function isAzureEligible(card: CardData): boolean {
  return card.cardType === "Event";
}

/** Returns true if the card is an Event (eligible for Bronze). */
export function isBronzeEligible(card: CardData): boolean {
  return card.cardType === "Event";
}

/** Map of transfiguration type to its eligibility check function. */
const ELIGIBILITY_CHECKS: Readonly<
  Record<TransfigurationType, (card: CardData) => boolean>
> = {
  Viridian: isViridianEligible,
  Golden: isGoldenEligible,
  Scarlet: isScarletEligible,
  Azure: isAzureEligible,
  Bronze: isBronzeEligible,
};

/** Returns the list of transfiguration types the card is eligible for. */
export function eligibleTransfigurations(
  card: CardData,
): TransfigurationType[] {
  const types: TransfigurationType[] = [];
  for (const [type, check] of Object.entries(ELIGIBILITY_CHECKS)) {
    if (check(card)) {
      types.push(type as TransfigurationType);
    }
  }
  return types;
}

/** Builds the preview card and description for a given transfiguration type. */
function buildOffer(
  card: CardData,
  type: TransfigurationType,
): TransfigurationOffer {
  switch (type) {
    case "Viridian": {
      const oldCost = card.energyCost ?? 0;
      const newCost = Math.round(oldCost / 2);
      return {
        type,
        description: `Energy cost: ${String(oldCost)} \u2192 ${String(newCost)}`,
        previewCard: { ...card, energyCost: newCost },
      };
    }
    case "Scarlet": {
      const oldSpark = card.spark ?? 0;
      const newSpark = oldSpark === 0 ? 1 : oldSpark * 2;
      return {
        type,
        description: `Spark: ${String(oldSpark)} \u2192 ${String(newSpark)}`,
        previewCard: { ...card, spark: newSpark },
      };
    }
    case "Azure":
      return {
        type,
        description: "Adds: Draw a card.",
        previewCard: {
          ...card,
          renderedText:
            card.renderedText +
            (card.renderedText.length > 0 ? " " : "") +
            "Draw a card.",
        },
      };
    case "Bronze":
      return {
        type,
        description: "Adds: Reclaim.",
        previewCard: {
          ...card,
          renderedText:
            card.renderedText +
            (card.renderedText.length > 0 ? " " : "") +
            "Reclaim.",
        },
      };
    case "Golden": {
      const match = card.renderedText.match(/\d+/);
      if (!match) {
        return {
          type,
          description: "Modifies a number in rules text.",
          previewCard: card,
        };
      }
      const num = parseInt(match[0], 10);
      const delta = Math.random() < 0.5 ? -1 : 1;
      const newNum = num + delta;
      const newText = card.renderedText.replace(match[0], String(newNum));
      return {
        type,
        description: `Number in text: ${String(num)} \u2192 ${String(newNum)}`,
        previewCard: { ...card, renderedText: newText },
      };
    }
  }
}

/**
 * Assigns a random eligible transfiguration to a card and returns
 * the complete offer with pre-computed preview and description.
 * Returns null if the card already has a transfiguration or no types
 * are eligible.
 */
export function assignTransfiguration(
  card: CardData,
  existingTransfiguration: TransfigurationType | null,
): TransfigurationOffer | null {
  if (existingTransfiguration !== null) {
    return null;
  }
  const eligible = eligibleTransfigurations(card);
  if (eligible.length === 0) {
    return null;
  }
  const chosen = eligible[Math.floor(Math.random() * eligible.length)];
  return buildOffer(card, chosen);
}

/** Returns a human-readable description of what a transfiguration does to a card. */
export function describeTransfiguration(
  card: CardData,
  type: TransfigurationType,
): string {
  return buildOffer(card, type).description;
}

/** Returns a record of the specific fields modified by a transfiguration offer. */
export function transfigurationEffectDetails(
  offer: TransfigurationOffer,
  originalCard: CardData,
): Record<string, unknown> {
  const details: Record<string, unknown> = {};
  if (offer.previewCard.energyCost !== originalCard.energyCost) {
    details.energyCost = {
      from: originalCard.energyCost,
      to: offer.previewCard.energyCost,
    };
  }
  if (offer.previewCard.spark !== originalCard.spark) {
    details.spark = {
      from: originalCard.spark,
      to: offer.previewCard.spark,
    };
  }
  if (offer.previewCard.renderedText !== originalCard.renderedText) {
    details.renderedText = {
      from: originalCard.renderedText,
      to: offer.previewCard.renderedText,
    };
  }
  return details;
}
