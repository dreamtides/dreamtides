import { describe, expect, it } from "vitest";
import { createBattleRng, deriveBattleSeed } from "./random";

describe("deriveBattleSeed", () => {
  it("hashes the exact battle entry key contract deterministically", () => {
    expect(deriveBattleSeed("site-7::3::dreamscape-2")).toBe(
      deriveBattleSeed("site-7::3::dreamscape-2"),
    );
    expect(deriveBattleSeed("site-7::3::dreamscape-2")).not.toBe(
      deriveBattleSeed("site-8::3::dreamscape-2"),
    );
  });
});

describe("createBattleRng", () => {
  it("keeps named streams independent while remaining reproducible", () => {
    const rewardA = createBattleRng(12345, "reward");
    const rewardB = createBattleRng(12345, "reward");
    const enemy = createBattleRng(12345, "enemyDescriptor");

    const rewardSequenceA = Array.from({ length: 5 }, () => rewardA.nextInt(1000));
    const rewardSequenceB = Array.from({ length: 5 }, () => rewardB.nextInt(1000));
    const enemySequence = Array.from({ length: 5 }, () => enemy.nextInt(1000));

    expect(rewardSequenceA).toEqual(rewardSequenceB);
    expect(enemySequence).not.toEqual(rewardSequenceA);
  });
});
