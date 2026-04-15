import { beforeEach, describe, expect, it, vi } from "vitest";
import type { CardData } from "../types/cards";
import type { DreamsignTemplate } from "../types/content";
import { generateRewardSiteData } from "./reward-generator";

function makeCard(overrides: Partial<CardData> = {}): CardData {
  return {
    name: "Test Card",
    id: "test-id",
    cardNumber: 1,
    cardType: "Character",
    subtype: "",
    rarity: "Common",
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

const DREAMSIGN_TEMPLATES: DreamsignTemplate[] = [
  {
    id: "dreamsign-1",
    name: "Dreamsign One",
    displayTide: "Bloom",
    packageTides: ["alpha"],
    effectDescription: "First effect.",
  },
  {
    id: "dreamsign-2",
    name: "Dreamsign Two",
    displayTide: "Arc",
    packageTides: ["beta"],
    effectDescription: "Second effect.",
  },
];

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("generateRewardSiteData", () => {
  it("prefers package-adjacent cards when revealing a card reward", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);

    const result = generateRewardSiteData({
      cardDatabase: makeDatabase([
        makeCard({ cardNumber: 1, tides: ["alpha"] }),
        makeCard({ cardNumber: 2, tides: ["beta"] }),
      ]),
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
      remainingDreamsignPoolIds: [],
      selectedPackageTides: ["alpha"],
    });

    expect(result.reward).toEqual({
      rewardType: "card",
      cardNumber: 1,
      cardName: "Test Card",
    });
    expect(result.spentDreamsignPoolIds).toEqual([]);
  });

  it("spends a shown dreamsign from the shared pool", () => {
    vi.spyOn(Math, "random")
      .mockReturnValueOnce(0)
      .mockReturnValueOnce(0)
      .mockReturnValueOnce(0.95);

    const result = generateRewardSiteData({
      cardDatabase: makeDatabase([makeCard({ tides: ["alpha"] })]),
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
      remainingDreamsignPoolIds: ["dreamsign-1", "dreamsign-2"],
      selectedPackageTides: ["alpha"],
    });

    expect(result.reward).toEqual({
      rewardType: "dreamsign",
      dreamsignId: "dreamsign-1",
      dreamsignName: "Dreamsign One",
      dreamsignTide: "Bloom",
      dreamsignEffect: "First effect.",
    });
    expect(result.spentDreamsignPoolIds).toEqual(["dreamsign-1"]);
    expect(result.remainingDreamsignPoolIds).toEqual(["dreamsign-2"]);
  });

  it("falls back to broader Dreamsign pool entries when no adjacent ones remain", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);

    const result = generateRewardSiteData({
      cardDatabase: new Map(),
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
      remainingDreamsignPoolIds: ["dreamsign-2"],
      selectedPackageTides: ["alpha"],
    });

    expect(result.reward).toEqual({
      rewardType: "dreamsign",
      dreamsignId: "dreamsign-2",
      dreamsignName: "Dreamsign Two",
      dreamsignTide: "Arc",
      dreamsignEffect: "Second effect.",
    });
    expect(result.remainingDreamsignPoolIds).toEqual([]);
  });

  it("falls back to essence when no card or Dreamsign reward is available", () => {
    vi.spyOn(Math, "random").mockReturnValue(0.5);

    const result = generateRewardSiteData({
      cardDatabase: new Map(),
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
      remainingDreamsignPoolIds: [],
      selectedPackageTides: ["alpha"],
    });

    expect(result.reward.rewardType).toBe("essence");
    expect(result.spentDreamsignPoolIds).toEqual([]);
    expect(result.remainingDreamsignPoolIds).toEqual([]);
  });
});
