import { describe, expect, it } from "vitest";
import type { DreamsignTemplate } from "../types/content";
import {
  drawDreamsignOptions,
  readDreamsignPool,
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

describe("readDreamsignPool", () => {
  it("treats the pool as a unique set of stable template ids", () => {
    const pool = readDreamsignPool(
      [
        "glacial-insight",
        "missing-id",
        "glacial-insight",
        "embers-whisper",
      ],
      DREAMSIGN_TEMPLATES,
    );

    expect(pool.availableIds).toEqual([
      "glacial-insight",
      "embers-whisper",
    ]);
  });
});

describe("drawDreamsignOptions", () => {
  it("spends shown ids immediately from the shared pool", () => {
    const draw = drawDreamsignOptions(
      ["embers-whisper", "glacial-insight", "verdant-accord"],
      DREAMSIGN_TEMPLATES,
      [],
      2,
    );

    expect(draw.offeredDreamsigns).toHaveLength(2);
    expect(draw.remainingDreamsignPool).toHaveLength(1);
    expect(draw.offeredIds.every((id) => !draw.remainingDreamsignPool.includes(id))).toBe(true);
  });

  it("consumes the shared pool across sequential reveals without repeats", () => {
    const first = drawDreamsignOptions(
      ["embers-whisper", "glacial-insight", "verdant-accord"],
      DREAMSIGN_TEMPLATES,
      [],
      2,
    );
    const second = drawDreamsignOptions(
      first.remainingDreamsignPool,
      DREAMSIGN_TEMPLATES,
      [],
      2,
    );

    expect([...first.offeredIds, ...second.offeredIds].sort()).toEqual([
      "embers-whisper",
      "glacial-insight",
      "verdant-accord",
    ]);
    expect(second.remainingDreamsignPool).toEqual([]);
  });

  it("cleans up stale ids instead of preserving a fake non-empty pool", () => {
    const draw = drawDreamsignOptions(
      ["missing-id", "glacial-insight"],
      DREAMSIGN_TEMPLATES,
      [],
      2,
    );

    expect(draw.offeredIds).toEqual(["glacial-insight"]);
    expect(draw.remainingDreamsignPool).toEqual([]);
  });

  it("degrades to a clean no-offer path when the pool is exhausted", () => {
    expect(
      drawDreamsignOptions(["missing-id"], DREAMSIGN_TEMPLATES, [], 3),
    ).toEqual({
      offeredIds: [],
      offeredDreamsigns: [],
      remainingDreamsignPool: [],
    });
  });

  it("only reveals dreamsigns matching the selected package tides", () => {
    const draw = drawDreamsignOptions(
      ["embers-whisper", "glacial-insight", "verdant-accord"],
      DREAMSIGN_TEMPLATES,
      ["beta"],
      3,
    );

    expect(draw.offeredIds).toEqual(["glacial-insight"]);
    expect(draw.remainingDreamsignPool).toEqual([
      "embers-whisper",
      "verdant-accord",
    ]);
  });

  it("keeps neutral dreamsigns eligible for any package", () => {
    const draw = drawDreamsignOptions(
      ["embers-whisper", "glacial-insight", "neutral-sign"],
      [
        ...DREAMSIGN_TEMPLATES,
        {
          id: "neutral-sign",
          name: "Neutral Sign",
          effectDescription: "Always eligible.",
        },
      ],
      ["beta"],
      2,
    );

    expect(draw.offeredIds.sort()).toEqual([
      "glacial-insight",
      "neutral-sign",
    ]);
  });
});

describe("resolveDreamsignTemplates", () => {
  it("returns templates in canonical pool order without duplicates", () => {
    expect(
      resolveDreamsignTemplates(
        [
          "glacial-insight",
          "missing-id",
          "embers-whisper",
          "glacial-insight",
        ],
        DREAMSIGN_TEMPLATES,
      ).map((template) => template.id),
    ).toEqual(["glacial-insight", "embers-whisper"]);
  });

  it("filters resolved templates to matching and neutral dreamsigns", () => {
    expect(
      resolveDreamsignTemplates(
        [
          "glacial-insight",
          "embers-whisper",
          "neutral-sign",
        ],
        [
          ...DREAMSIGN_TEMPLATES,
          {
            id: "neutral-sign",
            name: "Neutral Sign",
            effectDescription: "Always eligible.",
          },
        ],
        ["beta"],
      ).map((template) => template.id),
    ).toEqual(["glacial-insight", "neutral-sign"]);
  });
});
