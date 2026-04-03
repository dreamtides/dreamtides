import type { SiteType } from "../types/quest";

/** A cosmetic label for dreamscape nodes with an enhanced site type. */
export interface Biome {
  name: string;
  color: string;
  enhancedSiteType: SiteType;
}

/** The 9 biomes, one per enhanced site type. */
export const BIOMES: readonly Biome[] = [
  {
    name: "Verdant Hollow",
    color: "#2dd4bf",
    enhancedSiteType: "Shop",
  },
  {
    name: "Starfall Glade",
    color: "#c084fc",
    enhancedSiteType: "DreamsignOffering",
  },
  {
    name: "Wanderer's Threshold",
    color: "#818cf8",
    enhancedSiteType: "DreamJourney",
  },
  {
    name: "The Gilded Maw",
    color: "#fbbf24",
    enhancedSiteType: "TemptingOffer",
  },
  {
    name: "Ashfall Basin",
    color: "#f87171",
    enhancedSiteType: "Purge",
  },
  {
    name: "Crystal Spire",
    color: "#38bdf8",
    enhancedSiteType: "Essence",
  },
  {
    name: "Shadowforge",
    color: "#a78bfa",
    enhancedSiteType: "Transfiguration",
  },
  {
    name: "Hall of Echoes",
    color: "#34d399",
    enhancedSiteType: "Duplication",
  },
  {
    name: "The Obsidian Bazaar",
    color: "#fb923c",
    enhancedSiteType: "SpecialtyShop",
  },
] as const;
