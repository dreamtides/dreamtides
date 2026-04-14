import { describe, expect, it } from "vitest";
import type { DreamsignTemplate } from "../types/content";
import {
  drawDreamsignOptions,
  resolveDreamsignTemplates,
} from "./dreamsign-pool";

const DREAMSIGN_TEMPLATES: DreamsignTemplate[] = [
  {
    id: "embers-whisper",
    name: "Ember's Whisper",
    displayTide: "Ignite",
    packageTides: ["alpha"],
    effectDescription: "Fire.",
  },
  {
    id: "glacial-insight",
    name: "Glacial Insight",
    displayTide: "Rime",
    packageTides: ["beta"],
    effectDescription: "Ice.",
  },
  {
    id: "verdant-accord",
    name: "Verdant Accord",
    displayTide: "Bloom",
    packageTides: ["gamma"],
    effectDescription: "Growth.",
  },
];

describe("drawDreamsignOptions", () => {
  it("spends shown ids immediately from the shared pool", () => {
    const draw = drawDreamsignOptions(
      ["embers-whisper", "glacial-insight", "verdant-accord"],
      DREAMSIGN_TEMPLATES,
      2,
    );

    expect(draw.offeredDreamsigns).toHaveLength(2);
    expect(draw.remainingDreamsignPool).toHaveLength(1);
    expect(draw.offeredIds.every((id) => !draw.remainingDreamsignPool.includes(id))).toBe(true);
  });

  it("ignores stale ids that do not map to a template", () => {
    const draw = drawDreamsignOptions(
      ["missing-id", "glacial-insight"],
      DREAMSIGN_TEMPLATES,
      2,
    );

    expect(draw.offeredIds).toEqual(["glacial-insight"]);
    expect(draw.remainingDreamsignPool).toEqual(["missing-id"]);
  });
});

describe("resolveDreamsignTemplates", () => {
  it("returns templates in pool order and skips missing ids", () => {
    expect(
      resolveDreamsignTemplates(
        ["glacial-insight", "missing-id", "embers-whisper"],
        DREAMSIGN_TEMPLATES,
      ).map((template) => template.id),
    ).toEqual(["glacial-insight", "embers-whisper"]);
  });
});
