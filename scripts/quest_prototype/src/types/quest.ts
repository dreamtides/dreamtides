import type { Tide } from "./cards";
import type { ResolvedDreamcallerPackage } from "./content";
import type { DraftState } from "./draft";

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
  | "Draft"
  | "DreamcallerDraft"
  | "Shop"
  | "SpecialtyShop"
  | "DreamsignOffering"
  | "DreamsignDraft"
  | "DreamJourney"
  | "TemptingOffer"
  | "Purge"
  | "Essence"
  | "Transfiguration"
  | "Duplication"
  | "Reward"
  | "Cleanse";

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
  tide: Tide;
  abilityDescription: string;
  essenceBonus: number;
  tideCrystalGrant: Tide;
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
  | { type: "questStart" }
  | { type: "atlas" }
  | { type: "dreamscape" }
  | { type: "site"; siteId: string }
  | { type: "questComplete" };

/** The top-level quest state object. */
export interface QuestState {
  essence: number;
  deck: DeckEntry[];
  dreamcaller: Dreamcaller | null;
  resolvedPackage: ResolvedDreamcallerPackage | null;
  remainingDreamsignPool: string[];
  dreamsigns: Dreamsign[];
  completionLevel: number;
  atlas: DreamAtlas;
  currentDreamscape: string | null;
  visitedSites: string[];
  draftState: DraftState | null;
  screen: Screen;
  activeSiteId: string | null;
}
