import type { Tide } from "./cards";

export type PackageTideId = string;

export interface DreamcallerContent {
  id: string;
  name: string;
  title: string;
  awakening: number;
  renderedText: string;
  imageNumber: string;
  mandatoryTides: PackageTideId[];
  optionalTides: PackageTideId[];
}

export interface DreamsignTemplate {
  id: string;
  name: string;
  effectDescription: string;
  displayTide: Tide;
  packageTides: PackageTideId[];
}

export interface ResolvedDreamcallerPackage {
  dreamcaller: DreamcallerContent;
  mandatoryTides: PackageTideId[];
  optionalSubset: PackageTideId[];
  selectedTides: PackageTideId[];
  draftPoolCopiesByCard: Record<string, number>;
  dreamsignPoolIds: string[];
  mandatoryOnlyPoolSize: number;
  draftPoolSize: number;
  doubledCardCount: number;
  legalSubsetCount: number;
  preferredSubsetCount: number;
}
