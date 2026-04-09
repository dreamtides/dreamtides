import { describe, it, expect } from "vitest";
import { DREAMCALLERS } from "./dreamcallers";
import { DREAMSIGNS } from "./dreamsigns";
import { DREAM_JOURNEYS } from "./dream-journeys";
import type { JourneyEffect } from "./dream-journeys";
import { TEMPTING_OFFERS } from "./tempting-offers";
import type { OfferEffect } from "./tempting-offers";
import { BIOMES } from "./biomes";
import type { Tide, Rarity, NamedTide } from "../types/cards";

const ALL_TIDES: Tide[] = [
  "Bloom",
  "Arc",
  "Ignite",
  "Pact",
  "Umbra",
  "Rime",
  "Surge",
  "Neutral",
];
const NAMED_TIDES: Tide[] = ALL_TIDES.filter((t) => t !== "Neutral");
const ALL_RARITIES: Rarity[] = ["Common", "Uncommon", "Rare", "Legendary", "Starter"];
const tideSet = new Set<string>(ALL_TIDES);
const raritySet = new Set<string>(ALL_RARITIES);

/** Returns the tide values referenced by a JourneyEffect, if any. */
function journeyEffectTides(e: JourneyEffect): string[] {
  switch (e.type) {
    case "addTideCrystal":
      return [e.tide];
    case "removeCardsAndAddTideCrystal":
      return [e.tide];
    default:
      return [];
  }
}

/** Returns the rarity values referenced by a JourneyEffect, if any. */
function journeyEffectRarities(e: JourneyEffect): string[] {
  switch (e.type) {
    case "addRandomCards":
      return [e.rarity];
    case "removeCardsAndAddRandomCards":
      return [e.rarity];
    default:
      return [];
  }
}

/** Returns the tide values referenced by an OfferEffect, if any. */
function offerEffectTides(e: OfferEffect): string[] {
  switch (e.type) {
    case "addTideCrystal":
      return [e.tide];
    case "addMultipleTideCrystals":
      return e.crystals.map((c) => c.tide);
    default:
      return [];
  }
}

/** Returns the rarity values referenced by an OfferEffect, if any. */
function offerEffectRarities(e: OfferEffect): string[] {
  switch (e.type) {
    case "addRandomCards":
      return [e.rarity];
    default:
      return [];
  }
}

describe("dreamcallers", () => {
  it("has exactly 10 entries", () => {
    expect(DREAMCALLERS).toHaveLength(10);
  });

  it("every entry has two named tides and a matching named crystal", () => {
    for (const dc of DREAMCALLERS) {
      expect(dc.name.length).toBeGreaterThan(0);
      expect(dc.tides).toHaveLength(2);
      expect(dc.tides[0]).not.toBe(dc.tides[1]);
      expect(tideSet.has(dc.tides[0])).toBe(true);
      expect(tideSet.has(dc.tides[1])).toBe(true);
      expect(dc.tides).not.toContain("Neutral");
      expect(dc.abilityDescription.length).toBeGreaterThan(0);
      expect(dc.essenceBonus).toBeGreaterThanOrEqual(50);
      expect(dc.essenceBonus).toBeLessThanOrEqual(150);
      expect(dc.tides).toContain(dc.tideCrystalGrant);
    }
  });

  it("covers every named tide at least once", () => {
    const tides = new Set(DREAMCALLERS.flatMap((dc) => dc.tides));
    for (const t of NAMED_TIDES) {
      expect(tides.has(t)).toBe(true);
    }
  });

  it("covers all 7 neighbor pairs", () => {
    const TIDE_CIRCLE: NamedTide[] = ["Bloom", "Arc", "Ignite", "Pact", "Umbra", "Rime", "Surge"];
    const pairs = new Set<string>();
    for (const dc of DREAMCALLERS) {
      const sorted = [...dc.tides].sort((a, b) => TIDE_CIRCLE.indexOf(a) - TIDE_CIRCLE.indexOf(b));
      pairs.add(`${sorted[0]}/${sorted[1]}`);
    }
    for (let i = 0; i < 7; i++) {
      const a = TIDE_CIRCLE[i];
      const b = TIDE_CIRCLE[(i + 1) % 7];
      const sorted = [a, b].sort((x, y) => TIDE_CIRCLE.indexOf(x) - TIDE_CIRCLE.indexOf(y));
      expect(pairs.has(`${sorted[0]}/${sorted[1]}`)).toBe(true);
    }
  });

  it("has unique names", () => {
    const names = DREAMCALLERS.map((dc) => dc.name);
    expect(new Set(names).size).toBe(names.length);
  });
});

describe("dreamsigns", () => {
  it("has exactly 10 entries", () => {
    expect(DREAMSIGNS).toHaveLength(10);
  });

  it("every entry has all required fields with valid tides", () => {
    for (const ds of DREAMSIGNS) {
      expect(ds.name.length).toBeGreaterThan(0);
      expect(tideSet.has(ds.tide)).toBe(true);
      expect(ds.effectDescription.length).toBeGreaterThan(0);
    }
  });

  it("has unique names", () => {
    const names = DREAMSIGNS.map((ds) => ds.name);
    expect(new Set(names).size).toBe(names.length);
  });
});

describe("dream journeys", () => {
  it("has exactly 10 entries", () => {
    expect(DREAM_JOURNEYS).toHaveLength(10);
  });

  it("every entry has a name, description, and typed effect", () => {
    for (const dj of DREAM_JOURNEYS) {
      expect(dj.name.length).toBeGreaterThan(0);
      expect(dj.description.length).toBeGreaterThan(0);
      expect(typeof dj.effect.type).toBe("string");
    }
  });

  it("effect tides are valid Tide values", () => {
    for (const dj of DREAM_JOURNEYS) {
      for (const t of journeyEffectTides(dj.effect)) {
        expect(tideSet.has(t)).toBe(true);
      }
    }
  });

  it("effect rarities are valid Rarity values", () => {
    for (const dj of DREAM_JOURNEYS) {
      for (const r of journeyEffectRarities(dj.effect)) {
        expect(raritySet.has(r)).toBe(true);
      }
    }
  });

  it("no addEssence effect has a negative amount", () => {
    for (const dj of DREAM_JOURNEYS) {
      if (dj.effect.type === "addEssence") {
        expect(dj.effect.amount).toBeGreaterThan(0);
      }
    }
  });

  it("compound effects encode both parts of their description", () => {
    for (const dj of DREAM_JOURNEYS) {
      const e = dj.effect;
      if (e.type === "addEssenceAndRemoveCards") {
        expect(e.essenceAmount).toBeGreaterThan(0);
        expect(e.removeCount).toBeGreaterThan(0);
      }
      if (e.type === "removeCardsAndAddRandomCards") {
        expect(e.removeCount).toBeGreaterThan(0);
        expect(e.addCount).toBeGreaterThan(0);
      }
      if (e.type === "removeCardsAndAddTideCrystal") {
        expect(e.removeCount).toBeGreaterThan(0);
        expect(e.crystalCount).toBeGreaterThan(0);
      }
    }
  });

  it("has unique names", () => {
    const names = DREAM_JOURNEYS.map((dj) => dj.name);
    expect(new Set(names).size).toBe(names.length);
  });
});

describe("tempting offers", () => {
  it("has exactly 10 entries", () => {
    expect(TEMPTING_OFFERS).toHaveLength(10);
  });

  it("every entry has descriptions and typed benefit/cost effects", () => {
    for (const to of TEMPTING_OFFERS) {
      expect(to.benefitDescription.length).toBeGreaterThan(0);
      expect(to.costDescription.length).toBeGreaterThan(0);
      expect(typeof to.benefit.type).toBe("string");
      expect(typeof to.cost.type).toBe("string");
    }
  });

  it("effect tides are valid Tide values", () => {
    for (const to of TEMPTING_OFFERS) {
      for (const t of [
        ...offerEffectTides(to.benefit),
        ...offerEffectTides(to.cost),
      ]) {
        expect(tideSet.has(t)).toBe(true);
      }
    }
  });

  it("effect rarities are valid Rarity values", () => {
    for (const to of TEMPTING_OFFERS) {
      for (const r of [
        ...offerEffectRarities(to.benefit),
        ...offerEffectRarities(to.cost),
      ]) {
        expect(raritySet.has(r)).toBe(true);
      }
    }
  });

  it("at least 3 costs use addBaneCards", () => {
    const baneCount = TEMPTING_OFFERS.filter(
      (to) => to.cost.type === "addBaneCards",
    ).length;
    expect(baneCount).toBeGreaterThanOrEqual(3);
  });

  it("addMultipleTideCrystals entries grant crystals matching their description", () => {
    for (const to of TEMPTING_OFFERS) {
      if (to.benefit.type === "addMultipleTideCrystals") {
        expect(to.benefit.crystals.length).toBeGreaterThan(0);
        for (const c of to.benefit.crystals) {
          expect(tideSet.has(c.tide)).toBe(true);
          expect(c.count).toBeGreaterThan(0);
        }
      }
    }
  });
});

describe("biomes", () => {
  it("has exactly 9 entries", () => {
    expect(BIOMES).toHaveLength(9);
  });

  it("every entry has a name, hex color, and enhancedSiteType", () => {
    for (const b of BIOMES) {
      expect(b.name.length).toBeGreaterThan(0);
      expect(b.color).toMatch(/^#[0-9a-f]{6}$/i);
      expect(typeof b.enhancedSiteType).toBe("string");
    }
  });

  it("has exactly one biome per enhanced site type", () => {
    const expected = new Set([
      "CardShop",
      "DreamsignOffering",
      "DreamJourney",
      "TemptingOffer",
      "LootPack",
      "Essence",
      "Forge",
      "Duplication",
      "PackShop",
    ]);
    const actual = new Set(BIOMES.map((b) => b.enhancedSiteType));
    expect(actual).toEqual(expected);
  });

  it("has unique colors", () => {
    const colors = BIOMES.map((b) => b.color);
    expect(new Set(colors).size).toBe(colors.length);
  });

  it("has unique names", () => {
    const names = BIOMES.map((b) => b.name);
    expect(new Set(names).size).toBe(names.length);
  });
});
