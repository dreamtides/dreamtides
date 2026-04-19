import { describe, expect, it } from "vitest";
import { parseRuntimeConfig } from "./runtime-config";

describe("parseRuntimeConfig", () => {
  it("enables playable mode only for a single playable battle param", () => {
    expect(parseRuntimeConfig("?battle=playable")).toEqual({
      battleMode: "playable",
      seedOverride: null,
      startInBattle: false,
    });
  });

  it.each([
    "",
    "?battle=auto",
    "?battle=PLAYABLE",
    "?battle=playable&battle=playable",
    "?battle=playable&battle=auto",
    "?mode=playable",
  ])("falls back to auto mode for %s", (search) => {
    expect(parseRuntimeConfig(search)).toEqual({
      battleMode: "auto",
      seedOverride: null,
      startInBattle: false,
    });
  });

  describe("seedOverride", () => {
    it("returns the parsed integer when playable and seed is a non-negative integer", () => {
      expect(parseRuntimeConfig("?battle=playable&seed=42").seedOverride).toBe(
        42,
      );
      expect(parseRuntimeConfig("?battle=playable&seed=0").seedOverride).toBe(0);
      expect(
        parseRuntimeConfig("?battle=playable&seed=12345").seedOverride,
      ).toBe(12345);
    });

    it("ignores the seed parameter when battleMode is auto", () => {
      expect(parseRuntimeConfig("?seed=42").seedOverride).toBeNull();
      expect(parseRuntimeConfig("?battle=auto&seed=42").seedOverride).toBeNull();
      expect(
        parseRuntimeConfig("?battle=garbage&seed=42").seedOverride,
      ).toBeNull();
    });

    it("rejects non-numeric, negative, or empty seed values in playable mode", () => {
      expect(
        parseRuntimeConfig("?battle=playable&seed=foo").seedOverride,
      ).toBeNull();
      expect(
        parseRuntimeConfig("?battle=playable&seed=-5").seedOverride,
      ).toBeNull();
      expect(
        parseRuntimeConfig("?battle=playable&seed=").seedOverride,
      ).toBeNull();
      expect(
        parseRuntimeConfig("?battle=playable&seed=1.5").seedOverride,
      ).toBeNull();
      expect(
        parseRuntimeConfig("?battle=playable&seed=1e3").seedOverride,
      ).toBeNull();
    });
  });

  describe("startInBattle", () => {
    it("returns true only when both battle=playable and startInBattle=1 are present", () => {
      expect(
        parseRuntimeConfig("?battle=playable&startInBattle=1").startInBattle,
      ).toBe(true);
    });

    it("returns false when startInBattle is set without playable mode", () => {
      expect(parseRuntimeConfig("?startInBattle=1").startInBattle).toBe(false);
      expect(
        parseRuntimeConfig("?battle=auto&startInBattle=1").startInBattle,
      ).toBe(false);
      expect(
        parseRuntimeConfig("?battle=garbage&startInBattle=1").startInBattle,
      ).toBe(false);
    });

    it("returns false for non-1 values of startInBattle", () => {
      expect(
        parseRuntimeConfig("?battle=playable&startInBattle=0").startInBattle,
      ).toBe(false);
      expect(
        parseRuntimeConfig("?battle=playable&startInBattle=true").startInBattle,
      ).toBe(false);
      expect(
        parseRuntimeConfig("?battle=playable&startInBattle=").startInBattle,
      ).toBe(false);
    });
  });
});
