import { describe, expect, it } from "vitest";
import {
  leftNeighbor,
  normalizedPairKey,
  pairContainsTide,
  rightNeighbor,
} from "./tide-circle";

describe("tide-circle", () => {
  it("returns adjacent neighbors on the revised circle", () => {
    expect(leftNeighbor("Bloom")).toBe("Surge");
    expect(rightNeighbor("Bloom")).toBe("Arc");
    expect(leftNeighbor("Surge")).toBe("Rime");
    expect(rightNeighbor("Surge")).toBe("Bloom");
  });

  it("normalizes unordered tide pairs into a stable key", () => {
    expect(normalizedPairKey(["Bloom", "Arc"])).toBe("Bloom/Arc");
    expect(normalizedPairKey(["Arc", "Bloom"])).toBe("Bloom/Arc");
    expect(normalizedPairKey(["Surge", "Bloom"])).toBe("Bloom/Surge");
  });

  it("checks whether a named tide is in a pair", () => {
    expect(pairContainsTide(["Bloom", "Arc"], "Bloom")).toBe(true);
    expect(pairContainsTide(["Bloom", "Arc"], "Pact")).toBe(false);
  });
});
