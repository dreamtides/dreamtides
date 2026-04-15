import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CardData } from "../types/cards";
import { selectRareRewards } from "./tide-weights";

function makeCard(overrides: Partial<CardData> = {}): CardData {
  return {
    name: "Test Card",
    id: "test-id",
    cardNumber: 1,
    cardType: "Character",
    subtype: "",
    rarity: "Rare",
    energyCost: 2,
    spark: 1,
    isFast: false,
    tides: ["alpha"],
    renderedText: "Test text",
    imageNumber: 1,
    artOwned: false,
    ...overrides,
  };
}

function makeDatabase(cards: CardData[]): Map<number, CardData> {
  return new Map(cards.map((card) => [card.cardNumber, card]));
}

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("selectRareRewards", () => {
  it("prefers package-adjacent rare rewards", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);

    const rewards = selectRareRewards(
      makeDatabase([
        makeCard({ cardNumber: 1, tides: ["alpha"] }),
        makeCard({ cardNumber: 2, tides: ["alpha", "beta"] }),
        makeCard({ cardNumber: 3, tides: ["alpha", "gamma"] }),
        makeCard({ cardNumber: 4, tides: ["alpha", "delta"] }),
        makeCard({ cardNumber: 5, tides: ["beta"] }),
      ]),
      ["alpha"],
    );

    expect(rewards).toHaveLength(4);
    for (const reward of rewards) {
      expect(reward.tides).toContain("alpha");
    }
  });

  it("falls back to the broader rare pool when no adjacent rare rewards exist", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);

    const rewards = selectRareRewards(
      makeDatabase([
        makeCard({ cardNumber: 1, tides: ["beta"] }),
        makeCard({ cardNumber: 2, tides: ["gamma"] }),
        makeCard({ cardNumber: 3, tides: ["delta"] }),
        makeCard({ cardNumber: 4, tides: ["epsilon"] }),
      ]),
      ["alpha"],
    );

    expect(rewards).toHaveLength(4);
    expect(rewards.map((reward) => reward.cardNumber)).toEqual([1, 2, 3, 4]);
  });
});
