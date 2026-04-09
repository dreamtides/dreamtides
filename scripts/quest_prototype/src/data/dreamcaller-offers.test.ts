import { describe, expect, it } from "vitest";
import type { NamedTide } from "../types/cards";
import type { Dreamcaller } from "../types/quest";
import { selectOfferedDreamcallers } from "./dreamcaller-offers";
import type { QuestTideProfile } from "./quest-tide-profile";

const ZERO_TIDES = {
  Bloom: 0,
  Arc: 0,
  Ignite: 0,
  Pact: 0,
  Umbra: 0,
  Rime: 0,
  Surge: 0,
  Neutral: 0,
} as const;

function caller(name: string, tides: [NamedTide, NamedTide]): Dreamcaller {
  return {
    name,
    tides,
    abilityDescription: "Ability.",
    essenceBonus: 100,
    tideCrystalGrant: tides[0],
  };
}

function profile(): QuestTideProfile {
  return {
    weights: {
      Bloom: 8,
      Arc: 4,
      Ignite: 1,
      Pact: 1,
      Umbra: 1,
      Rime: 1,
      Surge: 4,
      Neutral: 0.5,
    },
    contributions: {
      baseline: {
        Bloom: 1,
        Arc: 1,
        Ignite: 1,
        Pact: 1,
        Umbra: 1,
        Rime: 1,
        Surge: 1,
        Neutral: 0.5,
      },
      startingTide: {
        ...ZERO_TIDES,
        Bloom: 5,
      },
      neighbors: {
        ...ZERO_TIDES,
        Arc: 2,
        Surge: 2,
      },
      deck: ZERO_TIDES,
      dreamcaller: ZERO_TIDES,
      crystals: ZERO_TIDES,
      recentDraftPicks: ZERO_TIDES,
    },
  };
}

describe("selectOfferedDreamcallers", () => {
  const pool = [
    caller("left", ["Surge", "Bloom"]),
    caller("right", ["Bloom", "Arc"]),
    caller("adaptive", ["Umbra", "Rime"]),
    caller("other", ["Ignite", "Pact"]),
  ];

  it("offers the two starting-tide neighbor forks when available", () => {
    const offered = selectOfferedDreamcallers({
      pool,
      startingTide: "Bloom",
      profile: profile(),
    });
    expect(offered).toHaveLength(3);
    expect(offered.map((dc) => dc.name)).toContain("left");
    expect(offered.map((dc) => dc.name)).toContain("right");
  });

  it("does not return duplicate dreamcallers", () => {
    const offered = selectOfferedDreamcallers({
      pool,
      startingTide: "Bloom",
      profile: profile(),
    });
    expect(new Set(offered.map((dc) => dc.name)).size).toBe(offered.length);
  });

  it("returns the whole pool when fewer than three dreamcallers are available", () => {
    const offered = selectOfferedDreamcallers({
      pool: pool.slice(0, 2),
      startingTide: "Bloom",
      profile: profile(),
    });
    expect(offered.map((dc) => dc.name).sort()).toEqual(["left", "right"]);
  });
});
