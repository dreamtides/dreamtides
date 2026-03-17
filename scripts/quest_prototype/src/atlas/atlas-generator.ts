import type {
  DreamAtlas,
  DreamscapeNode,
  SiteState,
  SiteType,
} from "../types/quest";
import { BIOMES, type Biome } from "../data/biomes";
import { logEvent } from "../logging";

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

/** Weighted random selection from an array of [item, weight] pairs. */
function weightedPick<T>(items: Array<[T, number]>): T {
  const total = items.reduce((sum, [, w]) => sum + w, 0);
  if (total <= 0) {
    return items[0][0];
  }
  let roll = Math.random() * total;
  for (const [item, weight] of items) {
    roll -= weight;
    if (roll <= 0) {
      return item;
    }
  }
  return items[items.length - 1][0];
}

/** Builds the weighted site pool based on completion level. */
function buildAdditionalSitePool(
  completionLevel: number,
): Array<[SiteType, number]> {
  const pool: Array<[SiteType, number]> = [];

  // Early game (all levels): Shop, Essence, DreamsignOffering
  pool.push(["Shop", 3]);
  pool.push(["Essence", 3]);
  pool.push(["DreamsignOffering", 3]);

  // Mid game (level 3+): add DreamJourney, TemptingOffer
  if (completionLevel >= 3) {
    pool.push(["DreamJourney", 2]);
    pool.push(["TemptingOffer", 2]);
  }

  // Late game (level 5+): add Transfiguration, Purge, Duplication
  if (completionLevel >= 5) {
    pool.push(["Transfiguration", 2]);
    pool.push(["Purge", 2]);
    pool.push(["Duplication", 2]);
  }

  // SpecialtyShop uncommon at any level
  pool.push(["SpecialtyShop", 1]);

  return pool;
}

/** Generates the site composition for a dreamscape. */
export function generateSiteComposition(
  completionLevel: number,
  isFirstDreamscape: boolean,
): SiteState[] {
  const sites: SiteState[] = [];

  // Draft sites based on completion level
  let draftCount: number;
  if (completionLevel <= 1) {
    draftCount = 2;
  } else if (completionLevel <= 3) {
    draftCount = 1;
  } else {
    draftCount = 0;
  }
  for (let i = 0; i < draftCount; i++) {
    sites.push({
      id: nextSiteId(),
      type: "Draft",
      isEnhanced: false,
      isVisited: false,
    });
  }

  // Dreamcaller draft only in the very first dreamscape
  if (isFirstDreamscape) {
    sites.push({
      id: nextSiteId(),
      type: "DreamcallerDraft",
      isEnhanced: false,
      isVisited: false,
    });
  }

  // 1-3 additional sites from the weighted pool
  const pool = buildAdditionalSitePool(completionLevel);
  const additionalCount = randomInt(1, 3);
  for (let i = 0; i < additionalCount; i++) {
    const siteType = weightedPick(pool);
    sites.push({
      id: nextSiteId(),
      type: siteType,
      isEnhanced: false,
      isVisited: false,
    });
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
  isFirstDreamscape: boolean,
  connections: string[],
): DreamscapeNode {
  const id = nextNodeId();
  const biome = assignBiome();
  const sites = generateSiteComposition(completionLevel, isFirstDreamscape);
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

/** Creates the initial atlas with the Nexus and 2-3 starting dreamscapes. */
export function generateInitialAtlas(completionLevel: number): DreamAtlas {
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

  // 2-3 initial dreamscape nodes
  const nodeCount = randomInt(2, 3);
  const baseAngle = randomFloat(0, Math.PI * 2);

  for (let i = 0; i < nodeCount; i++) {
    const angle =
      baseAngle +
      (i * Math.PI * 2) / nodeCount +
      randomFloat(-0.3, 0.3);
    const x = Math.cos(angle) * BASE_RADIUS;
    const y = Math.sin(angle) * BASE_RADIUS;

    const node = createNode(
      { x, y },
      completionLevel,
      i === 0,
      [nexusId],
    );
    nodes[node.id] = node;
    edges.push([nexusId, node.id]);
  }

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
      false,
      connections,
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

/** Site icon mapping: returns an emoji character for each site type. */
export function siteTypeIcon(siteType: SiteType): string {
  const icons: Record<SiteType, string> = {
    Battle: "\u2694\uFE0F",
    Draft: "\uD83C\uDCCF",
    DreamcallerDraft: "\uD83D\uDC51",
    Shop: "\uD83C\uDFEA",
    SpecialtyShop: "\u2B50",
    DreamsignOffering: "\u2728",
    DreamsignDraft: "\u2728",
    DreamJourney: "\uD83C\uDF19",
    TemptingOffer: "\u2696\uFE0F",
    Purge: "\uD83D\uDD25",
    Essence: "\uD83D\uDC8E",
    Transfiguration: "\u2697\uFE0F",
    Duplication: "\uD83D\uDCCB",
    Reward: "\uD83C\uDF81",
    Cleanse: "\u2744\uFE0F",
  };
  return icons[siteType];
}

/** Site type display name for tooltips. */
export function siteTypeName(siteType: SiteType): string {
  const names: Record<SiteType, string> = {
    Battle: "Battle",
    Draft: "Draft",
    DreamcallerDraft: "Dreamcaller Draft",
    Shop: "Shop",
    SpecialtyShop: "Specialty Shop",
    DreamsignOffering: "Dreamsign Offering",
    DreamsignDraft: "Dreamsign Draft",
    DreamJourney: "Dream Journey",
    TemptingOffer: "Tempting Offer",
    Purge: "Purge",
    Essence: "Essence",
    Transfiguration: "Transfiguration",
    Duplication: "Duplication",
    Reward: "Reward",
    Cleanse: "Cleanse",
  };
  return names[siteType];
}

/**
 * Returns the preview site types for a node tooltip.
 * Shows 2-3 non-draft, non-battle site icons.
 */
export function previewSiteTypes(node: DreamscapeNode): SiteType[] {
  const excluded: Set<SiteType> = new Set([
    "Battle",
    "Draft",
    "DreamcallerDraft",
  ]);
  return node.sites
    .filter((s) => !excluded.has(s.type))
    .map((s) => s.type)
    .slice(0, 3);
}
