import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  cardImageUrl,
  tideIconUrl,
  loadCardDatabase,
  NAMED_TIDES,
  TIDE_COLORS,
  RARITY_COLORS,
} from "./card-database";
import type { Tide, Rarity, CardData } from "../types/cards";

const SAMPLE_CARD: CardData = {
  name: "Test Card",
  id: "abc-123",
  cardNumber: 42,
  cardType: "Character",
  subtype: "Beast",
  rarity: "Common",
  energyCost: 3,
  spark: 2,
  isFast: false,
  tide: "Bloom",
  tideCost: 1,
  renderedText: "Sample text",
  imageNumber: 12345,
  artOwned: true,
};

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("loadCardDatabase", () => {
  it("fetches card-data.json and returns a Map keyed by cardNumber", async () => {
    const cards: CardData[] = [
      SAMPLE_CARD,
      { ...SAMPLE_CARD, cardNumber: 99, name: "Another Card" },
    ];
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(cards),
      }),
    );
    const db = await loadCardDatabase();
    expect(db.size).toBe(2);
    expect(db.get(42)?.name).toBe("Test Card");
    expect(db.get(99)?.name).toBe("Another Card");
  });

  it("throws on non-ok response", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 404,
        statusText: "Not Found",
      }),
    );
    await expect(loadCardDatabase()).rejects.toThrow("Failed to load card data");
  });
});

describe("cardImageUrl", () => {
  it("returns the webp path for a given card number", () => {
    expect(cardImageUrl(42)).toBe("/cards/42.webp");
  });

  it("works for card number 1", () => {
    expect(cardImageUrl(1)).toBe("/cards/1.webp");
  });

  it("works for large card numbers", () => {
    expect(cardImageUrl(503)).toBe("/cards/503.webp");
  });
});

describe("tideIconUrl", () => {
  it("returns the png path for a named tide", () => {
    expect(tideIconUrl("Bloom")).toBe("/tides/Bloom.png");
  });

  it("returns png paths for all named tides", () => {
    const named: Tide[] = [
      "Bloom",
      "Arc",
      "Ignite",
      "Pact",
      "Umbra",
      "Rime",
      "Surge",
    ];
    for (const tide of named) {
      expect(tideIconUrl(tide)).toBe(`/tides/${tide}.png`);
    }
  });

  it("returns a fallback for Wild tide", () => {
    const url = tideIconUrl("Wild");
    expect(url).not.toBe("/tides/Wild.png");
    expect(url.length).toBeGreaterThan(0);
    expect(url).toContain("data:");
  });
});

describe("NAMED_TIDES", () => {
  it("contains exactly 7 tides excluding Wild", () => {
    expect(NAMED_TIDES).toHaveLength(7);
    expect(NAMED_TIDES).not.toContain("Wild");
  });

  it("contains all named tide values", () => {
    expect(NAMED_TIDES).toContain("Bloom");
    expect(NAMED_TIDES).toContain("Arc");
    expect(NAMED_TIDES).toContain("Ignite");
    expect(NAMED_TIDES).toContain("Pact");
    expect(NAMED_TIDES).toContain("Umbra");
    expect(NAMED_TIDES).toContain("Rime");
    expect(NAMED_TIDES).toContain("Surge");
  });
});

describe("TIDE_COLORS", () => {
  it("maps all 8 tides to hex colors", () => {
    const expected: Record<Tide, string> = {
      Bloom: "#10b981",
      Arc: "#f59e0b",
      Ignite: "#ef4444",
      Pact: "#ec4899",
      Umbra: "#8b5cf6",
      Rime: "#3b82f6",
      Surge: "#06b6d4",
      Wild: "#9ca3af",
    };
    for (const [tide, color] of Object.entries(expected)) {
      expect(TIDE_COLORS[tide as Tide]).toBe(color);
    }
  });
});

describe("RARITY_COLORS", () => {
  it("maps all 4 rarities to hex colors", () => {
    const expected: Record<Rarity, string> = {
      Common: "#ffffff",
      Uncommon: "#10b981",
      Rare: "#3b82f6",
      Legendary: "#a855f7",
    };
    for (const [rarity, color] of Object.entries(expected)) {
      expect(RARITY_COLORS[rarity as Rarity]).toBe(color);
    }
  });
});
