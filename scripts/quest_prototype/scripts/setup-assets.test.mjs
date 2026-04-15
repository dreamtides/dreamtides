// @vitest-environment node

import {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, describe, expect, it, vi } from "vitest";
import { imageHash, setupAssets } from "./setup-assets.mjs";

afterEach(() => {
  vi.restoreAllMocks();
});

describe("setupAssets", () => {
  it("normalizes TOML cards and dreamcallers into runtime JSON artifacts", () => {
    const tempRoot = mkdtempSync(join(tmpdir(), "quest-setup-assets-"));
    const publicDir = join(tempRoot, "public");
    const imageCacheDir = join(tempRoot, "image-cache");
    const dreamcallerArtDir = join(tempRoot, "dreamcaller-art");
    const tideIconsDir = join(tempRoot, "tides-src");
    const cardTomlPath = join(tempRoot, "rendered-cards.toml");
    const dreamcallerTomlPath = join(tempRoot, "dreamcallers.toml");
    const cachedImagePath = join(imageCacheDir, imageHash(101));

    mkdirSync(imageCacheDir, { recursive: true });
    mkdirSync(dreamcallerArtDir, { recursive: true });
    mkdirSync(tideIconsDir, { recursive: true });
    mkdirSync(dirname(cachedImagePath), { recursive: true });
    writeFileSync(cachedImagePath, "fake-webp");
    writeFileSync(join(dreamcallerArtDir, "0007.png"), "fake-png");
    writeFileSync(join(tideIconsDir, "Bloom.png"), "fake-png");
    writeFileSync(
      cardTomlPath,
      `[[cards]]
name = "Null Spark"
id = "null-spark"
card-number = 1
card-type = "Character"
rarity = "Common"
energy-cost = "*"
is-fast = false
tides = ["core", "accent:Bloom"]
rendered-text = "Rules text."
image-number = 101
art-owned = true

[[cards]]
name = "Missing Subtype"
id = "missing-subtype"
card-number = 2
card-type = "Event"
rarity = "Rare"
energy-cost = 2
spark = ""
is-fast = true
tides = ["support"]
rendered-text = ""
image-number = 102
art-owned = false

[[cards]]
name = "Starter Card"
id = "starter-card"
card-number = 3
card-type = "Character"
subtype = "Beast"
rarity = "Starter"
energy-cost = 1
spark = 1
is-fast = false
tides = ["ignored"]
rendered-text = ""
image-number = 103
art-owned = true
`,
    );
    writeFileSync(
      dreamcallerTomlPath,
      `[[dreamcaller]]
id = "dc-1"
name = "Dreamcaller One"
title = "Keeper of Test Cases"
awakening = 4
rendered-text = "Choose tides."
image-number = "0007"
mandatory-tides = ["core", "bridge"]
optional-tides = ["support", "tempo", "finish", "value"]
`,
    );
    vi.spyOn(console, "warn").mockImplementation(() => {});

    setupAssets({
      cardTomlPath,
      dreamcallerTomlPath,
      tideIconsDir,
      publicDir,
      imageCacheDir,
      dreamcallerArtDir,
    });

    const cards = JSON.parse(
      readFileSync(join(publicDir, "card-data.json"), "utf8"),
    );
    const dreamcallers = JSON.parse(
      readFileSync(join(publicDir, "dreamcaller-data.json"), "utf8"),
    );

    expect(cards).toEqual([
      {
        name: "Null Spark",
        id: "null-spark",
        cardNumber: 1,
        cardType: "Character",
        subtype: "",
        rarity: "Common",
        energyCost: null,
        spark: null,
        isFast: false,
        tides: ["core", "accent:Bloom"],
        renderedText: "Rules text.",
        imageNumber: 101,
        artOwned: true,
      },
      {
        name: "Missing Subtype",
        id: "missing-subtype",
        cardNumber: 2,
        cardType: "Event",
        subtype: "",
        rarity: "Rare",
        energyCost: 2,
        spark: null,
        isFast: true,
        tides: ["support"],
        renderedText: "",
        imageNumber: 102,
        artOwned: false,
      },
      {
        name: "Starter Card",
        id: "starter-card",
        cardNumber: 3,
        cardType: "Character",
        subtype: "Beast",
        rarity: "Starter",
        energyCost: 1,
        spark: 1,
        isFast: false,
        tides: ["ignored"],
        renderedText: "",
        imageNumber: 103,
        artOwned: true,
      },
    ]);
    expect(dreamcallers).toEqual([
      {
        id: "dc-1",
        name: "Dreamcaller One",
        title: "Keeper of Test Cases",
        awakening: 4,
        renderedText: "Choose tides.",
        imageNumber: "0007",
        mandatoryTides: ["core", "bridge"],
        optionalTides: ["support", "tempo", "finish", "value"],
      },
    ]);
    expect(existsSync(join(publicDir, "cards", "1.webp"))).toBe(true);
    expect(existsSync(join(publicDir, "dreamcallers", "0007.png"))).toBe(true);
    expect(existsSync(join(publicDir, "tides", "Bloom.png"))).toBe(true);
  });
});
