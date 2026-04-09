import type { CardData, NamedTide, Tide } from "./cards";

/** Badge applied to a card via a Transfiguration site. */
export type TransfigurationType =
  | "Viridian"
  | "Golden"
  | "Scarlet"
  | "Azure"
  | "Bronze";

/** All site types available in dreamscapes. */
export type SiteType =
  | "Battle"
  | "LootPack"
  | "CardShop"
  | "PackShop"
  | "DreamcallerDraft"
  | "DraftSite"
  | "Forge"
  | "Provisioner"
  | "DreamsignOffering"
  | "DreamsignDraft"
  | "DreamJourney"
  | "TemptingOffer"
  | "Essence"
  | "Transfiguration"
  | "Duplication"
  | "Reward"
  | "Cleanse";

/** State for the ante system during battles. */
export interface AnteState {
  anteAccepted: boolean;
  playerAnteCards: number[];
  opponentAnteCards: number[];
  escalationTriggered: boolean;
  playerConceded: boolean;
  escalationPhase: "none" | "pending" | "resolved";
}

/** Types of card packs available in the pack shop. */
export type PackType = "tide" | "alliance" | "removal" | "aggro" | "events";

/** A slot in the pack shop. */
export interface PackShopSlot {
  packType: PackType;
  tide?: Tide;
  alliance?: string;
  price: number;
  cards: CardData[];
  purchased: boolean;
}

/** A recipe for the forge system. */
export interface ForgeRecipe {
  sacrificeTide: Tide;
  sacrificeCount: number;
  outputCard: CardData | null;
}

/** An option available from the provisioner. */
export interface ProvisionerOption {
  siteType: SiteType;
  cost: number;
  purchased: boolean;
}

/** An entry in the player's deck. Duplicates are possible. */
export interface DeckEntry {
  entryId: string;
  cardNumber: number;
  transfiguration: TransfigurationType | null;
  isBane: boolean;
}

/** A selected character that grants bonuses. */
export interface Dreamcaller {
  name: string;
  tides: [NamedTide, NamedTide];
  abilityDescription: string;
  essenceBonus: number;
  tideCrystalGrant: NamedTide;
}

/** A passive effect collected during the quest. */
export interface Dreamsign {
  name: string;
  tide: Tide;
  effectDescription: string;
  isBane: boolean;
}

/** A site within a dreamscape. */
export interface SiteState {
  id: string;
  type: SiteType;
  isEnhanced: boolean;
  isVisited: boolean;
  data?: Record<string, unknown>;
}

/** A node on the Dream Atlas representing a dreamscape. */
export interface DreamscapeNode {
  id: string;
  biomeName: string;
  biomeColor: string;
  sites: SiteState[];
  position: { x: number; y: number };
  status: "completed" | "available" | "unavailable";
  enhancedSiteType: SiteType | null;
}

/** The Dream Atlas graph containing all dreamscape nodes. */
export interface DreamAtlas {
  nodes: Record<string, DreamscapeNode>;
  edges: Array<[string, string]>;
  nexusId: string;
}

/** Discriminated union for the current screen. */
export type Screen =
  | { type: "dreamscape" }
  | { type: "atlas" }
  | { type: "site"; siteId: string }
  | { type: "questComplete" };

/** The top-level quest state object. */
export interface QuestState {
  essence: number;
  deck: DeckEntry[];
  pool: DeckEntry[];
  dreamcaller: Dreamcaller | null;
  dreamsigns: Dreamsign[];
  tideCrystals: Record<Tide, number>;
  completionLevel: number;
  atlas: DreamAtlas;
  currentDreamscape: string | null;
  visitedSites: string[];
  startingTide: NamedTide | null;
  anteState: AnteState | null;
  screen: Screen;
  activeSiteId: string | null;
  essenceWarningShown: boolean;
}
