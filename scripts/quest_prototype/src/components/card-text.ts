import type { CardData } from "../types/cards";

/** Symbol types recognized in card rules text. */
export type SymbolType = "energy" | "spark" | "trigger" | "fast";

/** A parsed segment of rules text: either plain text or a special symbol. */
export type TextSegment =
  | { kind: "text"; value: string }
  | { kind: "symbol"; symbol: SymbolType; char: string };

/** Maps special Unicode characters to their symbol type. */
const SYMBOL_MAP: Readonly<Record<string, SymbolType>> = {
  "\u25CF": "energy",
  "\u234F": "spark",
  "\u25B8": "trigger",
  "\u21AF": "fast",
};

/** Parses rules text into segments of plain text and special symbols. */
export function tokenizeRulesText(text: string): TextSegment[] {
  const segments: TextSegment[] = [];
  let buffer = "";

  function flushBuffer() {
    if (buffer.length > 0) {
      segments.push({ kind: "text", value: buffer });
      buffer = "";
    }
  }

  for (const char of text) {
    const symbolType = SYMBOL_MAP[char];
    if (symbolType !== undefined) {
      flushBuffer();
      segments.push({ kind: "symbol", symbol: symbolType, char });
    } else {
      buffer += char;
    }
  }
  flushBuffer();
  return segments;
}

/** Format the card type and subtype line. */
export function formatTypeLine(card: CardData): string {
  if (card.cardType === "Character") {
    if (card.subtype && card.subtype !== "" && card.subtype !== "*") {
      return card.subtype;
    }
    return "";
  }

  if (card.subtype && card.subtype !== "" && card.subtype !== "*") {
    return `${card.cardType} \u2014 ${card.subtype}`;
  }
  return card.cardType;
}
