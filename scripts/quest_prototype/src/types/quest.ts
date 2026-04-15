import type { Tide } from "./cards";
import type { PackageTideId, ResolvedDreamcallerPackage } from "./content";
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

/** The selected Dreamcaller package shown in player-facing UI. */
export interface Dreamcaller {
  id: string;
  name: string;
  title: string;
  awakening: number;
  renderedText: string;
  imageNumber: string;
  accentTide: Tide;
}

/** A passive effect collected during the quest. */
export interface Dreamsign {
  name: string;
  tide: Tide;
  effectDescription: string;
  isBane: boolean;
}

/** One card currently being explained by the provenance debug overlay. */
export interface CardSourceDebugEntry {
  cardNumber: number;
  cardName: string;
  cardTides: PackageTideId[];
  matchedMandatoryTides: PackageTideId[];
  matchedOptionalTides: PackageTideId[];
  isFallback: boolean;
}

/** Which surface produced the currently explained cards. */
export type CardSourceDebugSurface =
  | "Draft"
  | "Shop"
  | "SpecialtyShop"
  | "BattleReward"
  | "Reward";

/** Global debug data for cards currently revealed on a quest screen. */
export interface CardSourceDebugState {
  screenLabel: string;
  surface: CardSourceDebugSurface;
  entries: CardSourceDebugEntry[];
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
  cardSourceDebug: CardSourceDebugState | null;
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
