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
    isStarter: false,
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
    effectDescription: "First effect.",
    imageName: "dreamsign-1.png",
  },
  {
    id: "dreamsign-2",
    name: "Dreamsign Two",
    effectDescription: "Second effect.",
    imageName: "dreamsign-2.png",
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

  it("skips starter cards when picking a card reward", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);

    const result = generateRewardSiteData({
      cardDatabase: makeDatabase([
        makeCard({ cardNumber: 1, isStarter: true, tides: ["alpha"] }),
        makeCard({ cardNumber: 2, name: "Real Reward", tides: ["alpha"] }),
      ]),
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
      remainingDreamsignPoolIds: [],
      selectedPackageTides: ["alpha"],
    });

    expect(result.reward).toEqual({
      rewardType: "card",
      cardNumber: 2,
      cardName: "Real Reward",
    });
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
      dreamsign: {
        id: "dreamsign-1",
        name: "Dreamsign One",
        effectDescription: "First effect.",
        imageName: "dreamsign-1.png",
        imageAlt: "Dreamsign One Dreamsign artwork",
        tide: null,
        isBane: false,
      },
    });
    expect(result.spentDreamsignPoolIds).toEqual(["dreamsign-1"]);
    expect(result.remainingDreamsignPoolIds).toEqual(["dreamsign-2"]);
  });

  it("draws randomly from the remaining Dreamsign pool", () => {
    vi.spyOn(Math, "random").mockReturnValue(0);

    const result = generateRewardSiteData({
      cardDatabase: new Map(),
      dreamsignTemplates: DREAMSIGN_TEMPLATES,
      remainingDreamsignPoolIds: ["dreamsign-2"],
      selectedPackageTides: ["alpha"],
    });

    expect(result.reward).toEqual({
      rewardType: "dreamsign",
      dreamsign: {
        id: "dreamsign-2",
        name: "Dreamsign Two",
        effectDescription: "Second effect.",
        imageName: "dreamsign-2.png",
        imageAlt: "Dreamsign Two Dreamsign artwork",
        tide: null,
        isBane: false,
      },
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
