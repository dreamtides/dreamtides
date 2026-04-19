import type { PackageTideId } from "./content";

/** The 8 tide affinities in Dreamtides. */
export type Tide =
  | "Bloom"
  | "Arc"
  | "Ignite"
  | "Pact"
  | "Umbra"
  | "Rime"
  | "Surge"
  | "Neutral";

/** The two card types in Dreamtides. */
export type CardType = "Character" | "Event";

/** A single card record loaded from card-data.json. */
export interface CardData {
  name: string;
  id: string;
  cardNumber: number;
  cardType: CardType;
  subtype: string;
  isStarter: boolean;
  energyCost: number | null;
  spark: number | null;
  isFast: boolean;
  tides: PackageTideId[];
  renderedText: string;
  imageNumber: number;
  artOwned: boolean;
}

/**
 * A `CardData` value whose top-level and `tides` array are frozen at runtime
 * (see `freezeCardData` in `create-battle-init.ts`). Used for surface
 * snapshots (`BattleInit.rewardOptions`) so consumers that receive a frozen
 * value get compile-time feedback if they try to mutate it (bug-030).
 */
export interface FrozenCardData extends Omit<CardData, "tides"> {
  readonly tides: readonly PackageTideId[];
}
