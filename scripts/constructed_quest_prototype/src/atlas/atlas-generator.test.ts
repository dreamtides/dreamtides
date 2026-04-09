import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  generateSiteComposition,
  generateInitialAtlas,
  generateNewNodes,
  assignBiome,
  previewSiteTypes,
  rewardPreviewLabel,
  resetAtlasGenerator,
  type SiteGenerationContext,
} from "./atlas-generator";
import type { DreamscapeNode, SiteState } from "../types/quest";
import type { CardData } from "../types/cards";
import type { QuestConfig } from "../state/quest-config";

const TEST_CONFIG: QuestConfig = {
  revisedTides: true,
  initialCards: 10,
  starterNeutral: 5,
  starterLowCost: 4,
  starterMidCost: 3,
  starterHighCost: 1,
  startingEssence: 400,
  lootPackSize: 4,
  dupePenalty2: 50,
  dupePenalty3: 90,
  packOnTheme: 60,
  packAdjacent: 25,
  packExplore: 15,
  minimumDeckSize: 25,
  maximumDeckSize: 50,
  maxCopies: 2,
  cardShopSize: 4,
  cardPriceMin: 30,
  cardPriceMax: 70,
  rerollBase: 10,
  rerollIncrement: 5,
  packShopSize: 3,
  specialPackChance: 20,
  anteEnabled: true,
  escalationTurn: 6,
  maxAnteCards: 2,
  forgeRecipes: 3,
  forgeCost: 4,
  draftSiteTotal: 4,
  draftSiteKeep: 1,
  provisionerOptions: 3,
  dreamcallerChoices: 3,
  opponentPreviewCards: 3,
  battleEssence: 150,
  essencePerLevel: 50,
  essenceSiteAmount: 200,
  showTideSymbols: true,
};

function defaultContext(
  overrides?: Partial<SiteGenerationContext>,
): SiteGenerationContext {
  const db = new Map<number, CardData>();
  db.set(1, {
    name: "Test Card",
    id: "test-card",
    cardNumber: 1,
    cardType: "Character",
    subtype: "",
    rarity: "Common",
    energyCost: 2,
    spark: 1,
    isFast: false,
    tide: "Bloom",
    tideCost: 1,
    renderedText: "Test rules text.",
    imageNumber: 1,
    artOwned: false,
  });
  return {
    cardDatabase: db,
    dreamsignPool: [
      {
        name: "Test Dreamsign",
        tide: "Bloom",
        effectDescription: "Test effect.",
      },
    ],
    playerHasBanes: false,
    startingTide: "Bloom",
    playerPool: [],
    config: TEST_CONFIG,
    ...overrides,
  };
}

beforeEach(() => {
  resetAtlasGenerator();
  vi.restoreAllMocks();
  vi.spyOn(console, "log").mockImplementation(() => {});
});

describe("generateSiteComposition", () => {
  it("level 0 produces DreamcallerDraft, 2 LootPacks, CardShop, PackShop, Battle", () => {
    const sites = generateSiteComposition(0, defaultContext({ dreamscapeTide: "Bloom" }));
    const types = sites.map((s) => s.type);
    expect(types[0]).toBe("DreamcallerDraft");
    expect(types.filter((t) => t === "LootPack")).toHaveLength(2);
    expect(types).toContain("CardShop");
    expect(types).toContain("PackShop");
    expect(types[types.length - 1]).toBe("Battle");
    expect(types).toHaveLength(6);
  });

  it("level 1+ always includes CardShop and PackShop", () => {
    for (let i = 0; i < 20; i++) {
      const sites = generateSiteComposition(1, defaultContext());
      const types = sites.map((s) => s.type);
      expect(types).toContain("CardShop");
      expect(types).toContain("PackShop");
    }
  });

  it("produces correct site counts for levels 1-6", () => {
    const expectedPackCounts = [3, 3, 2, 2, 1, 1, 1];
    for (let level = 1; level <= 6; level++) {
      for (let i = 0; i < 20; i++) {
        resetAtlasGenerator();
        const sites = generateSiteComposition(level, defaultContext());
        const packs = sites.filter((s) => s.type === "LootPack");
        expect(packs.length).toBe(expectedPackCounts[level]);
        expect(sites[sites.length - 1].type).toBe("Battle");
        // Total sites = packs + pool sites + battle
        expect(sites.length).toBeGreaterThanOrEqual(packs.length + 2);
      }
    }
  });

  it("includes DreamcallerDraft only at level 0", () => {
    const level0 = generateSiteComposition(0, defaultContext());
    expect(level0.some((s) => s.type === "DreamcallerDraft")).toBe(true);
    const level1 = generateSiteComposition(1, defaultContext());
    expect(level1.some((s) => s.type === "DreamcallerDraft")).toBe(false);
  });

  it("always ends with a Battle site", () => {
    for (let level = 0; level <= 7; level++) {
      resetAtlasGenerator();
      const sites = generateSiteComposition(level, defaultContext());
      expect(sites[sites.length - 1].type).toBe("Battle");
    }
  });

  it("has at least 2 non-draft non-battle sites for hover preview", () => {
    for (let i = 0; i < 50; i++) {
      resetAtlasGenerator();
      const sites = generateSiteComposition(0, defaultContext());
      const previewable = sites.filter(
        (s) =>
          s.type !== "Battle" &&
          s.type !== "DraftSite",
      );
      expect(previewable.length).toBeGreaterThanOrEqual(2);
    }
  });

  it("assigns unique IDs to all sites", () => {
    const sites = generateSiteComposition(0, defaultContext());
    const ids = sites.map((s) => s.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("populates reward data on Reward sites", () => {
    let foundReward = false;
    for (let i = 0; i < 200; i++) {
      resetAtlasGenerator();
      const sites = generateSiteComposition(2, defaultContext());
      const reward = sites.find((s) => s.type === "Reward");
      if (reward) {
        foundReward = true;
        expect(reward.data).toBeDefined();
        expect(reward.data!["rewardType"]).toBeDefined();
        break;
      }
    }
    expect(foundReward).toBe(true);
  });

  it("excludes Cleanse sites when player has no banes", () => {
    for (let i = 0; i < 100; i++) {
      resetAtlasGenerator();
      const sites = generateSiteComposition(
        1,
        defaultContext({ playerHasBanes: false }),
      );
      const cleanse = sites.filter((s) => s.type === "Cleanse");
      expect(cleanse.length).toBe(0);
    }
  });

  it("can include Cleanse sites when player has banes", () => {
    let foundCleanse = false;
    for (let i = 0; i < 200; i++) {
      resetAtlasGenerator();
      const sites = generateSiteComposition(
        1,
        defaultContext({ playerHasBanes: true }),
      );
      if (sites.some((s) => s.type === "Cleanse")) {
        foundCleanse = true;
        break;
      }
    }
    expect(foundCleanse).toBe(true);
  });

  it("LootPack sites have packTide in their data", () => {
    const sites = generateSiteComposition(0, defaultContext());
    const packs = sites.filter((s) => s.type === "LootPack");
    expect(packs.length).toBe(2);
    for (const pack of packs) {
      expect(pack.data).toBeDefined();
      expect(pack.data!["packTide"]).toBeDefined();
    }
  });
});

describe("generateInitialAtlas", () => {
  it("creates exactly 1 dreamscape node plus the nexus", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    const nodeCount = Object.keys(atlas.nodes).length;
    expect(nodeCount).toBe(2);
  });

  it("places the nexus at (0,0) with status completed", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    const nexus = atlas.nodes[atlas.nexusId];
    expect(nexus).toBeDefined();
    expect(nexus.position.x).toBe(0);
    expect(nexus.position.y).toBe(0);
    expect(nexus.status).toBe("completed");
  });

  it("marks all non-nexus nodes as available", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    for (const [id, node] of Object.entries(atlas.nodes)) {
      if (id !== atlas.nexusId) {
        expect(node.status).toBe("available");
      }
    }
  });

  it("creates edges from nexus to each dreamscape node", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    const nonNexusIds = Object.keys(atlas.nodes).filter(
      (id) => id !== atlas.nexusId,
    );
    for (const nodeId of nonNexusIds) {
      const hasEdge = atlas.edges.some(
        ([a, b]) =>
          (a === atlas.nexusId && b === nodeId) ||
          (b === atlas.nexusId && a === nodeId),
      );
      expect(hasEdge).toBe(true);
    }
  });

  it("positions dreamscape nodes at the base radius distance from nexus", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    for (const [id, node] of Object.entries(atlas.nodes)) {
      if (id === atlas.nexusId) continue;
      const dist = Math.sqrt(
        node.position.x * node.position.x +
          node.position.y * node.position.y,
      );
      expect(dist).toBeCloseTo(200, 0);
    }
  });

  it("includes DreamcallerDraft in every initial dreamscape node", () => {
    for (let i = 0; i < 30; i++) {
      const atlas = generateInitialAtlas(0, defaultContext());
      const initialNodes = Object.values(atlas.nodes).filter(
        (n) => n.id !== atlas.nexusId,
      );
      for (const node of initialNodes) {
        expect(node.sites.some((s) => s.type === "DreamcallerDraft")).toBe(true);
      }
    }
  });
});

describe("generateNewNodes", () => {
  it("generates 2-4 new nodes connected to the completed node", () => {
    for (let i = 0; i < 20; i++) {
      const atlas = generateInitialAtlas(0, defaultContext());
      const completedId = Object.keys(atlas.nodes).find(
        (id) => id !== atlas.nexusId,
      )!;
      const updated = generateNewNodes(atlas, completedId, 0, defaultContext());
      const newNodeCount =
        Object.keys(updated.nodes).length - Object.keys(atlas.nodes).length;
      expect(newNodeCount).toBeGreaterThanOrEqual(2);
      expect(newNodeCount).toBeLessThanOrEqual(4);
    }
  });

  it("marks the completed node as completed", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    const completedId = Object.keys(atlas.nodes).find(
      (id) => id !== atlas.nexusId,
    )!;
    const updated = generateNewNodes(atlas, completedId, 0, defaultContext());
    expect(updated.nodes[completedId].status).toBe("completed");
  });

  it("sets correct availability on new nodes", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    const completedId = Object.keys(atlas.nodes).find(
      (id) => id !== atlas.nexusId,
    )!;
    const updated = generateNewNodes(atlas, completedId, 0, defaultContext());

    const completedIds = new Set(
      Object.values(updated.nodes)
        .filter((n) => n.status === "completed")
        .map((n) => n.id),
    );

    for (const [nodeId, node] of Object.entries(updated.nodes)) {
      if (node.status === "completed") continue;
      const connectedToCompleted = updated.edges.some(
        ([a, b]) =>
          (a === nodeId && completedIds.has(b)) ||
          (b === nodeId && completedIds.has(a)),
      );
      if (connectedToCompleted) {
        expect(node.status).toBe("available");
      } else {
        expect(node.status).toBe("unavailable");
      }
    }
  });

  it("returns atlas unchanged for an invalid node ID", () => {
    const atlas = generateInitialAtlas(0, defaultContext());
    const result = generateNewNodes(atlas, "nonexistent", 0, defaultContext());
    expect(result).toBe(atlas);
  });
});

describe("assignBiome", () => {
  it("returns a biome with name, color, and enhancedSiteType", () => {
    const biome = assignBiome();
    expect(biome.name).toBeDefined();
    expect(typeof biome.name).toBe("string");
    expect(biome.color).toBeDefined();
    expect(biome.enhancedSiteType).toBeDefined();
  });
});

describe("previewSiteTypes", () => {
  it("excludes Battle and Draft but includes DreamcallerDraft", () => {
    const node: DreamscapeNode = {
      id: "test",
      biomeName: "Test",
      biomeColor: "#000",
      sites: [
        { id: "s1", type: "DraftSite", isEnhanced: false, isVisited: false },
        { id: "s2", type: "Battle", isEnhanced: false, isVisited: false },
        { id: "s3", type: "DreamcallerDraft", isEnhanced: false, isVisited: false },
        { id: "s4", type: "CardShop", isEnhanced: false, isVisited: false },
        { id: "s5", type: "Essence", isEnhanced: false, isVisited: false },
      ],
      position: { x: 0, y: 0 },
      status: "available",
      enhancedSiteType: null,
    };
    const preview = previewSiteTypes(node);
    expect(preview).toEqual(["DreamcallerDraft", "CardShop", "Essence"]);
  });

  it("returns at most 3 site types", () => {
    const node: DreamscapeNode = {
      id: "test",
      biomeName: "Test",
      biomeColor: "#000",
      sites: [
        { id: "s1", type: "CardShop", isEnhanced: false, isVisited: false },
        { id: "s2", type: "Essence", isEnhanced: false, isVisited: false },
        { id: "s3", type: "LootPack", isEnhanced: false, isVisited: false },
        { id: "s4", type: "DreamJourney", isEnhanced: false, isVisited: false },
      ],
      position: { x: 0, y: 0 },
      status: "available",
      enhancedSiteType: null,
    };
    const preview = previewSiteTypes(node);
    expect(preview.length).toBeLessThanOrEqual(3);
  });
});

describe("rewardPreviewLabel", () => {
  it("returns card reward label for card reward sites", () => {
    const site: SiteState = {
      id: "s1",
      type: "Reward",
      isEnhanced: false,
      isVisited: false,
      data: { rewardType: "card", cardNumber: 1, cardName: "Fire Bolt" },
    };
    expect(rewardPreviewLabel(site)).toBe("Reward: Fire Bolt");
  });

  it("returns dreamsign reward label for dreamsign reward sites", () => {
    const site: SiteState = {
      id: "s2",
      type: "Reward",
      isEnhanced: false,
      isVisited: false,
      data: {
        rewardType: "dreamsign",
        dreamsignName: "Ember's Whisper",
        dreamsignTide: "Ignite",
        dreamsignEffect: "Fire effect.",
      },
    };
    expect(rewardPreviewLabel(site)).toBe("Reward: Ember's Whisper");
  });

  it("returns essence reward label for essence reward sites", () => {
    const site: SiteState = {
      id: "s3",
      type: "Reward",
      isEnhanced: false,
      isVisited: false,
      data: { rewardType: "essence", essenceAmount: 250 },
    };
    expect(rewardPreviewLabel(site)).toBe("Reward: 250 Essence");
  });

  it("returns null for non-reward sites", () => {
    const site: SiteState = {
      id: "s4",
      type: "CardShop",
      isEnhanced: false,
      isVisited: false,
    };
    expect(rewardPreviewLabel(site)).toBeNull();
  });
});
