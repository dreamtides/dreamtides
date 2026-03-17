/** A structured effect for the benefit or cost of a tempting offer. */
export type OfferEffect =
  | { type: "addEssence"; amount: number }
  | { type: "addRandomCards"; count: number; rarity: string }
  | { type: "addTideCrystal"; tide: string; count: number }
  | { type: "addBaneCards"; count: number }
  | { type: "removeEssence"; amount: number }
  | { type: "removeDreamsign" }
  | { type: "reduceMaxDreamsigns"; amount: number }
  | { type: "removeRandomCards"; count: number }
  | { type: "addDreamsign" };

/** A risk/reward choice with a powerful benefit and a meaningful cost. */
export interface TemptingOffer {
  benefitDescription: string;
  costDescription: string;
  benefit: OfferEffect;
  cost: OfferEffect;
}

/** The 10 available tempting offers. */
export const TEMPTING_OFFERS: readonly TemptingOffer[] = [
  {
    benefitDescription:
      "A river of liquid gold pours from a crack in the dream. Gain 300 essence.",
    costDescription:
      "The gold carries a curse. Three bane cards seep into your deck.",
    benefit: { type: "addEssence", amount: 300 },
    cost: { type: "addBaneCards", count: 3 },
  },
  {
    benefitDescription:
      "A spectral merchant offers two rare cards from a locked vault, free of charge.",
    costDescription:
      "The vault's guardian marks your soul. Two bane cards join your deck.",
    benefit: { type: "addRandomCards", count: 2, rarity: "Rare" },
    cost: { type: "addBaneCards", count: 2 },
  },
  {
    benefitDescription:
      "A tide crystal hums with concentrated power. Gain 2 Bloom tide crystals.",
    costDescription:
      "The crystal's resonance destabilizes your essence reserves. Lose 150 essence.",
    benefit: { type: "addTideCrystal", tide: "Bloom", count: 2 },
    cost: { type: "removeEssence", amount: 150 },
  },
  {
    benefitDescription:
      "An ancient dreamsign materializes before you, offering its passive power freely.",
    costDescription:
      "Accepting the sign tears away another. Lose one existing dreamsign at random.",
    benefit: { type: "addDreamsign" },
    cost: { type: "removeDreamsign" },
  },
  {
    benefitDescription:
      "The dreamscape peels back to reveal a hoard of 400 essence.",
    costDescription:
      "The hoard is trapped. Four bane cards latch onto your deck as you flee.",
    benefit: { type: "addEssence", amount: 400 },
    cost: { type: "addBaneCards", count: 4 },
  },
  {
    benefitDescription:
      "A shadowed forge offers to craft a legendary card from raw nightmare.",
    costDescription:
      "The forge demands fuel. Lose 250 essence to feed the flames.",
    benefit: { type: "addRandomCards", count: 1, rarity: "Legendary" },
    cost: { type: "removeEssence", amount: 250 },
  },
  {
    benefitDescription:
      "A constellation of tide crystals descends. Gain one crystal each of Arc and Rime.",
    costDescription:
      "The celestial gift narrows your dreamsign capacity. Max dreamsigns reduced by 2.",
    benefit: { type: "addTideCrystal", tide: "Arc", count: 2 },
    cost: { type: "reduceMaxDreamsigns", amount: 2 },
  },
  {
    benefitDescription:
      "A phantasmal librarian offers three uncommon cards of surprising synergy.",
    costDescription:
      "The librarian reclaims a toll. Two random cards are stripped from your deck.",
    benefit: { type: "addRandomCards", count: 3, rarity: "Uncommon" },
    cost: { type: "removeRandomCards", count: 2 },
  },
  {
    benefitDescription:
      "Essence condenses from the fog in shimmering droplets. Gain 200 essence.",
    costDescription:
      "The fog leaves toxic residue. Two bane cards drift into your deck.",
    benefit: { type: "addEssence", amount: 200 },
    cost: { type: "addBaneCards", count: 2 },
  },
  {
    benefitDescription:
      "A weeping idol grants a surge of raw power. Gain 500 essence.",
    costDescription:
      "The idol's tears are corrosive. Your max dreamsigns capacity is reduced by 3.",
    benefit: { type: "addEssence", amount: 500 },
    cost: { type: "reduceMaxDreamsigns", amount: 3 },
  },
] as const;
