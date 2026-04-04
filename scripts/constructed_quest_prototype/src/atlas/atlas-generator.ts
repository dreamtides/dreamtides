import type {
  DeckEntry,
  DreamAtlas,
  DreamscapeNode,
  SiteState,
  SiteType,
} from "../types/quest";
import type { CardData, Tide } from "../types/cards";
import type { Dreamsign } from "../types/quest";
import type { QuestConfig } from "../state/quest-config";
import { BIOMES, type Biome } from "../data/biomes";
import { selectPackTide } from "../data/tide-weights";
import { logEvent } from "../logging";

/** Parameters for site generation that require external data. */
export interface SiteGenerationContext {
  cardDatabase: Map<number, CardData>;
  dreamsignPool: ReadonlyArray<Omit<Dreamsign, "isBane">>;
  playerHasBanes: boolean;
  startingTides: Tide[];
  playerPool: DeckEntry[];
  config: QuestConfig;
}

const BASE_RADIUS = 200;
const RADIUS_INCREMENT = 160;

let nodeIdCounter = 0;
let siteIdCounter = 0;

function nextNodeId(): string {
  nodeIdCounter += 1;
  return `dreamscape-${String(nodeIdCounter)}`;
}

function nextSiteId(): string {
  siteIdCounter += 1;
  return `site-${String(siteIdCounter)}`;
}

/** Resets internal counters. Call when starting a new quest. */
export function resetAtlasGenerator(): void {
  nodeIdCounter = 0;
  siteIdCounter = 0;
}

function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}

function randomFloat(min: number, max: number): number {
  return Math.random() * (max - min) + min;
}

function pickRandom<T>(items: readonly T[]): T {
  return items[Math.floor(Math.random() * items.length)];
}

function distance(
  a: { x: number; y: number },
  b: { x: number; y: number },
): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  return Math.sqrt(dx * dx + dy * dy);
}


/** Loot pack counts per completion level (levels 0-6). */
const LOOT_PACK_COUNTS: readonly number[] = [3, 3, 2, 2, 1, 1, 1];

/** Returns the site pool for a given completion level. */
function getPoolForLevel(level: number, hasBanes: boolean): SiteType[] {
  const pool: SiteType[] = [];
  switch (level) {
    case 0:
      // Level 0 is fixed composition, no pool needed
      break;
    case 1:
      pool.push("CardShop", "PackShop", "Essence", "DreamsignOffering");
      break;
    case 2:
      pool.push("CardShop", "PackShop", "DraftSite", "Essence", "DreamsignDraft", "DreamJourney", "Reward");
      break;
    case 3:
      pool.push("CardShop", "PackShop", "Forge", "DraftSite", "DreamJourney", "TemptingOffer", "Essence");
      break;
    case 4:
      pool.push("CardShop", "PackShop", "Forge", "Provisioner", "Transfiguration", "Duplication", "DraftSite", "DreamJourney", "TemptingOffer", "DreamsignDraft");
      break;
    case 5:
      pool.push("CardShop", "PackShop", "Forge", "Provisioner", "Transfiguration", "Duplication", "DreamJourney", "TemptingOffer");
      break;
    default:
      // Level 6+
      pool.push("CardShop", "PackShop", "Forge", "Transfiguration", "Essence");
      break;
  }
  if (hasBanes && level > 0) {
    pool.push("Cleanse");
  }
  return pool;
}

/** Returns the [min, max] count range for pool sites at a given level. */
function getPoolCountRange(level: number): [number, number] {
  switch (level) {
    case 0: return [0, 0];
    case 1: return [1, 2];
    case 2: return [2, 3];
    case 3: return [2, 3];
    case 4: return [3, 4];
    case 5: return [3, 4];
    default: return [2, 3]; // Level 6+
  }
}

/** Generates reward data for a Reward site at dreamscape creation time. */
function generateRewardData(
  context: SiteGenerationContext,
): Record<string, unknown> {
  const { cardDatabase, dreamsignPool, startingTides } = context;
  const excludedSet = new Set(startingTides);
  const cards = Array.from(cardDatabase.values()).filter(
    (c) => !excludedSet.has(c.tide),
  );

  // 70% chance card reward, 30% chance dreamsign reward
  if (cards.length > 0 && Math.random() < 0.7) {
    const card = cards[Math.floor(Math.random() * cards.length)];
    return {
      rewardType: "card",
      cardNumber: card.cardNumber,
      cardName: card.name,
    };
  }

  if (dreamsignPool.length > 0) {
    const template =
      dreamsignPool[Math.floor(Math.random() * dreamsignPool.length)];
    return {
      rewardType: "dreamsign",
      dreamsignName: template.name,
      dreamsignTide: template.tide,
      dreamsignEffect: template.effectDescription,
    };
  }

  // Fallback to essence reward
  return {
    rewardType: "essence",
    essenceAmount: randomInt(150, 350),
  };
}

/** Generates the site composition for a dreamscape based on completion level. */
export function generateSiteComposition(
  completionLevel: number,
  context: SiteGenerationContext,
): SiteState[] {
  const sites: SiteState[] = [];
  const clampedLevel = Math.min(Math.max(completionLevel, 0), 6);

  if (clampedLevel === 0) {
    // Level 0: fixed composition (DreamcallerDraft, 3 LootPacks, CardShop, Battle)
    sites.push({
      id: nextSiteId(),
      type: "DreamcallerDraft",
      isEnhanced: false,
      isVisited: false,
    });
    const existingPackTides: Tide[] = [];
    for (let i = 0; i < 3; i++) {
      const packTide = selectPackTide(
        context.playerPool,
        context.cardDatabase,
        context.config,
        existingPackTides,
      );
      existingPackTides.push(packTide);
      sites.push({
        id: nextSiteId(),
        type: "LootPack",
        isEnhanced: false,
        isVisited: false,
        data: { packTide },
      });
    }
    sites.push({
      id: nextSiteId(),
      type: "CardShop",
      isEnhanced: false,
      isVisited: false,
    });
  } else {
    // Levels 1-6: loot packs + random pool sites
    const lootPackCount = LOOT_PACK_COUNTS[clampedLevel] ?? 1;
    const existingPackTides: Tide[] = [];
    for (let i = 0; i < lootPackCount; i++) {
      const packTide = selectPackTide(
        context.playerPool,
        context.cardDatabase,
        context.config,
        existingPackTides,
      );
      existingPackTides.push(packTide);
      sites.push({
        id: nextSiteId(),
        type: "LootPack",
        isEnhanced: false,
        isVisited: false,
        data: { packTide },
      });
    }

    // Random pool sites
    const pool = getPoolForLevel(clampedLevel, context.playerHasBanes);
    const [minCount, maxCount] = getPoolCountRange(clampedLevel);
    const poolCount = randomInt(minCount, maxCount);
    for (let i = 0; i < poolCount && pool.length > 0; i++) {
      const siteType = pickRandom(pool);
      const data =
        siteType === "Reward" ? generateRewardData(context) : undefined;
      sites.push({
        id: nextSiteId(),
        type: siteType,
        isEnhanced: false,
        isVisited: false,
        data,
      });
    }
  }

  // Battle site always last
  sites.push({
    id: nextSiteId(),
    type: "Battle",
    isEnhanced: false,
    isVisited: false,
  });

  return sites;
}

/** Randomly assigns a biome from the 9 biomes. */
export function assignBiome(): Biome {
  return pickRandom(BIOMES);
}

/**
 * Marks the biome's enhanced site type on matching sites.
 * Returns the enhanced site type if found, null otherwise.
 */
function applyBiomeEnhancement(
  sites: SiteState[],
  biome: Biome,
): SiteType | null {
  let enhancedType: SiteType | null = null;
  for (let i = 0; i < sites.length; i++) {
    if (sites[i].type === biome.enhancedSiteType) {
      sites[i] = { ...sites[i], isEnhanced: true };
      enhancedType = biome.enhancedSiteType;
    }
  }
  return enhancedType;
}

/** Creates a single dreamscape node at the given position. */
function createNode(
  position: { x: number; y: number },
  completionLevel: number,
  connections: string[],
  context: SiteGenerationContext,
): DreamscapeNode {
  const id = nextNodeId();
  const biome = assignBiome();
  const sites = generateSiteComposition(completionLevel, context);
  const enhancedSiteType = applyBiomeEnhancement(sites, biome);

  logEvent("atlas_node_generated", {
    nodeId: id,
    connections,
    position: { x: position.x, y: position.y },
  });

  logEvent("dreamscape_generated", {
    dreamscapeId: id,
    biomeName: biome.name,
    siteTypes: sites.map((s) => s.type),
    enhancedSiteType,
  });

  return {
    id,
    biomeName: biome.name,
    biomeColor: biome.color,
    sites,
    position,
    status: "available",
    enhancedSiteType,
  };
}

/** Creates the initial atlas with the Nexus and a single starting dreamscape. */
export function generateInitialAtlas(
  completionLevel: number,
  context: SiteGenerationContext,
): DreamAtlas {
  resetAtlasGenerator();

  const nexusId = "nexus";
  const nodes: Record<string, DreamscapeNode> = {};
  const edges: Array<[string, string]> = [];

  // Nexus at center
  nodes[nexusId] = {
    id: nexusId,
    biomeName: "Nexus",
    biomeColor: "#7c3aed",
    sites: [],
    position: { x: 0, y: 0 },
    status: "completed",
    enhancedSiteType: null,
  };

  // Single initial dreamscape node
  const angle = randomFloat(0, Math.PI * 2);
  const x = Math.cos(angle) * BASE_RADIUS;
  const y = Math.sin(angle) * BASE_RADIUS;

  const node = createNode(
    { x, y },
    completionLevel,
    [nexusId],
    context,
  );
  nodes[node.id] = node;
  edges.push([nexusId, node.id]);

  return { nodes, edges, nexusId };
}

/**
 * Generates 2-4 new nodes after completing a dreamscape.
 * New nodes connect to the completed node and any geometrically
 * nearby existing nodes (within 1.5x the radius increment).
 */
export function generateNewNodes(
  atlas: DreamAtlas,
  completedNodeId: string,
  completionLevel: number,
  context: SiteGenerationContext,
): DreamAtlas {
  const completedNode = atlas.nodes[completedNodeId];
  if (!completedNode) {
    return atlas;
  }

  const updatedNodes = { ...atlas.nodes };
  const updatedEdges = [...atlas.edges];

  // Mark the completed node
  updatedNodes[completedNodeId] = {
    ...completedNode,
    status: "completed",
  };

  // Determine the generation ring (how many levels deep)
  const completedDistance = distance(
    completedNode.position,
    { x: 0, y: 0 },
  );
  const newRadius = completedDistance + RADIUS_INCREMENT;

  // Angle from center to the completed node
  const parentAngle = Math.atan2(
    completedNode.position.y,
    completedNode.position.x,
  );

  const newNodeCount = randomInt(2, 4);
  const spread = Math.PI / 3; // 60-degree spread for children

  for (let i = 0; i < newNodeCount; i++) {
    const angleOffset =
      ((i - (newNodeCount - 1) / 2) * spread) / Math.max(newNodeCount - 1, 1);
    const angle = parentAngle + angleOffset + randomFloat(-0.15, 0.15);
    const x = Math.cos(angle) * newRadius;
    const y = Math.sin(angle) * newRadius;

    const connections = [completedNodeId];

    // Connect to nearby existing nodes
    const proximityThreshold = RADIUS_INCREMENT * 1.5;
    for (const [existingId, existingNode] of Object.entries(updatedNodes)) {
      if (existingId === completedNodeId) continue;
      if (existingId === atlas.nexusId) continue;
      const dist = distance({ x, y }, existingNode.position);
      if (dist < proximityThreshold && !connections.includes(existingId)) {
        connections.push(existingId);
      }
    }

    const node = createNode(
      { x, y },
      completionLevel,
      connections,
      context,
    );
    updatedNodes[node.id] = node;

    for (const connId of connections) {
      updatedEdges.push([connId, node.id]);
    }
  }

  // Update availability: a node is available if connected to nexus or any completed node
  const completedIds = new Set(
    Object.values(updatedNodes)
      .filter((n) => n.status === "completed")
      .map((n) => n.id),
  );

  for (const [nodeId, node] of Object.entries(updatedNodes)) {
    if (node.status === "completed") continue;
    const isConnectedToCompleted = updatedEdges.some(
      ([a, b]) =>
        (a === nodeId && completedIds.has(b)) ||
        (b === nodeId && completedIds.has(a)),
    );
    updatedNodes[nodeId] = {
      ...node,
      status: isConnectedToCompleted ? "available" : "unavailable",
    };
  }

  return {
    nodes: updatedNodes,
    edges: updatedEdges,
    nexusId: atlas.nexusId,
  };
}

/** Metadata for each site type: icon and display name in one table. */
const SITE_TYPE_META: Record<SiteType, { icon: string; name: string }> = {
  Battle: { icon: "\u2694\uFE0F", name: "Battle" },
  LootPack: { icon: "\uD83C\uDCCF", name: "Loot Pack" },
  CardShop: { icon: "\uD83D\uDED2", name: "Card Shop" },
  PackShop: { icon: "\uD83C\uDF81", name: "Pack Shop" },
  DreamcallerDraft: { icon: "\uD83D\uDC51", name: "Dreamcaller Draft" },
  DraftSite: { icon: "\uD83C\uDFB2", name: "Draft" },
  Forge: { icon: "\uD83D\uDD28", name: "Forge" },
  Provisioner: { icon: "\uD83E\uDDED", name: "Provisioner" },
  DreamsignOffering: { icon: "\u2728", name: "Dreamsign Offering" },
  DreamsignDraft: { icon: "\u2728", name: "Dreamsign Draft" },
  DreamJourney: { icon: "\uD83C\uDF19", name: "Dream Journey" },
  TemptingOffer: { icon: "\u2696\uFE0F", name: "Tempting Offer" },
  Essence: { icon: "\uD83D\uDC8E", name: "Essence" },
  Transfiguration: { icon: "\u2697\uFE0F", name: "Transfiguration" },
  Duplication: { icon: "\uD83D\uDCCB", name: "Duplication" },
  Reward: { icon: "\uD83C\uDFC6", name: "Reward" },
  Cleanse: { icon: "\u2744\uFE0F", name: "Cleanse" },
};

/** Returns an emoji icon for the given site type. */
export function siteTypeIcon(siteType: SiteType): string {
  return SITE_TYPE_META[siteType].icon;
}

/** Returns the display name for the given site type. */
export function siteTypeName(siteType: SiteType): string {
  return SITE_TYPE_META[siteType].name;
}

/**
 * Returns the preview site types for a node tooltip.
 * Shows 2-3 non-draft, non-battle site icons.
 */
export function previewSiteTypes(node: DreamscapeNode): SiteType[] {
  const excluded: Set<SiteType> = new Set([
    "Battle",
    "DraftSite",
  ]);
  return node.sites
    .filter((s) => !excluded.has(s.type))
    .map((s) => s.type)
    .slice(0, 3);
}

/** Returns a reward preview label for atlas tooltip display, or null if not a reward site. */
export function rewardPreviewLabel(site: SiteState): string | null {
  if (site.type !== "Reward" || !site.data) return null;
  const rewardType = site.data["rewardType"] as string | undefined;
  if (rewardType === "card") {
    const name = (site.data["cardName"] as string | undefined) ?? "Card";
    return `Reward: ${name}`;
  }
  if (rewardType === "dreamsign") {
    const name = (site.data["dreamsignName"] as string | undefined) ?? "Dreamsign";
    return `Reward: ${name}`;
  }
  if (rewardType === "essence") {
    const amount = (site.data["essenceAmount"] as number | undefined) ?? 0;
    return `Reward: ${String(amount)} Essence`;
  }
  return null;
}
