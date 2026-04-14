import { loadCardDatabase, packageTideAccent } from "./card-database";
import { DREAMSIGN_TEMPLATES } from "./dreamsigns";
import type {
  DreamcallerContent,
  DreamsignTemplate,
  PackageTideId,
  ResolvedDreamcallerPackage,
} from "../types/content";
import type { CardData, Tide } from "../types/cards";

const DREAMCALLER_JSON_PATH = "/dreamcaller-data.json";
const LEGAL_MIN_POOL_SIZE = 175;
const LEGAL_MAX_POOL_SIZE = 225;
const PREFERRED_MIN_POOL_SIZE = 190;
const PREFERRED_MAX_POOL_SIZE = 210;
const MANDATORY_ONLY_MIN_POOL_SIZE = 110;
const MANDATORY_ONLY_MAX_POOL_SIZE = 150;

export interface QuestContent {
  cardDatabase: Map<number, CardData>;
  cardsByPackageTide: Map<PackageTideId, CardData[]>;
  dreamcallers: DreamcallerContent[];
  dreamsignTemplates: readonly DreamsignTemplate[];
  resolvedPackagesByDreamcallerId: Map<string, ResolvedDreamcallerPackage>;
}

/** Returns the overlap count between a candidate and the selected package tides. */
export function countPackageOverlap(
  candidatePackageTides: readonly PackageTideId[],
  selectedPackageTides: Iterable<PackageTideId>,
): number {
  const selected = new Set(selectedPackageTides);
  let overlap = 0;

  for (const packageTideId of candidatePackageTides) {
    if (selected.has(packageTideId)) {
      overlap += 1;
    }
  }

  return overlap;
}

/** Returns whether a candidate shares any package tide with the selected package. */
export function isPackageAdjacent(
  candidatePackageTides: readonly PackageTideId[],
  selectedPackageTides: Iterable<PackageTideId>,
): boolean {
  return countPackageOverlap(candidatePackageTides, selectedPackageTides) > 0;
}

/** Returns a stable accent tide for Dreamcaller display surfaces. */
export function dreamcallerAccentTide(
  dreamcaller: Pick<DreamcallerContent, "mandatoryTides">,
): Tide {
  if (dreamcaller.mandatoryTides.length === 0) {
    return "Neutral";
  }
  return packageTideAccent(dreamcaller.mandatoryTides[0]);
}

/** Fetches normalized Dreamcaller content from the asset pipeline output. */
export async function loadDreamcallerContent(): Promise<DreamcallerContent[]> {
  const response = await fetch(DREAMCALLER_JSON_PATH);
  if (!response.ok) {
    throw new Error(
      `Failed to load Dreamcaller data: ${String(response.status)} ${response.statusText}`,
    );
  }
  return (await response.json()) as DreamcallerContent[];
}

/** Loads normalized quest content and validates Dreamcaller package data up front. */
export async function loadQuestContent(): Promise<QuestContent> {
  const [cardDatabase, dreamcallers] = await Promise.all([
    loadCardDatabase(),
    loadDreamcallerContent(),
  ]);
  const cards = Array.from(cardDatabase.values());
  const cardsByPackageTide = buildCardsByPackageTideIndex(cards);
  const resolvedPackagesByDreamcallerId = new Map<
    string,
    ResolvedDreamcallerPackage
  >();
  const errors: string[] = [];

  for (const dreamcaller of dreamcallers) {
    try {
      resolvedPackagesByDreamcallerId.set(
        dreamcaller.id,
        resolveDreamcallerPackage(
          dreamcaller,
          cards,
          DREAMSIGN_TEMPLATES,
        ),
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      errors.push(`${dreamcaller.name}: ${message}`);
    }
  }

  if (errors.length > 0) {
    throw new Error(
      `Dreamcaller package validation failed:\n${errors.join("\n")}`,
    );
  }

  return {
    cardDatabase,
    cardsByPackageTide,
    dreamcallers,
    dreamsignTemplates: DREAMSIGN_TEMPLATES,
    resolvedPackagesByDreamcallerId,
  };
}

/** Builds a package-tide index so later generators can query adjacent cards cheaply. */
export function buildCardsByPackageTideIndex(
  cards: readonly CardData[],
): Map<PackageTideId, CardData[]> {
  const index = new Map<PackageTideId, CardData[]>();

  for (const card of cards) {
    for (const packageTideId of card.tides) {
      const bucket = index.get(packageTideId);
      if (bucket === undefined) {
        index.set(packageTideId, [card]);
      } else {
        bucket.push(card);
      }
    }
  }

  return index;
}

/** Resolves and validates the fixed package for one Dreamcaller. */
export function resolveDreamcallerPackage(
  dreamcaller: DreamcallerContent,
  cards: readonly CardData[],
  dreamsignTemplates: readonly DreamsignTemplate[],
): ResolvedDreamcallerPackage {
  const mandatoryOnlyPool = buildDraftPoolCopies(cards, dreamcaller.mandatoryTides);
  const mandatoryOnlyPoolSize = countDraftPoolSize(mandatoryOnlyPool);

  if (
    mandatoryOnlyPoolSize < MANDATORY_ONLY_MIN_POOL_SIZE ||
    mandatoryOnlyPoolSize > MANDATORY_ONLY_MAX_POOL_SIZE
  ) {
    throw new Error(
      `mandatory-only pool size ${String(mandatoryOnlyPoolSize)} is outside ${String(MANDATORY_ONLY_MIN_POOL_SIZE)}-${String(MANDATORY_ONLY_MAX_POOL_SIZE)}`,
    );
  }

  const candidates = enumeratePackageCandidates(dreamcaller.optionalTides).map(
    (optionalSubset) => {
      const selectedTides = [...dreamcaller.mandatoryTides, ...optionalSubset];
      const draftPoolCopiesByCard = buildDraftPoolCopies(cards, selectedTides);
      const draftPoolSize = countDraftPoolSize(draftPoolCopiesByCard);

      return {
        optionalSubset,
        selectedTides,
        draftPoolCopiesByCard,
        draftPoolSize,
      };
    },
  );

  const legalCandidates = candidates.filter(
    (candidate) =>
      candidate.draftPoolSize >= LEGAL_MIN_POOL_SIZE &&
      candidate.draftPoolSize <= LEGAL_MAX_POOL_SIZE,
  );

  if (legalCandidates.length === 0) {
    throw new Error(
      `no legal optional subset produced a ${String(LEGAL_MIN_POOL_SIZE)}-${String(LEGAL_MAX_POOL_SIZE)} card pool`,
    );
  }

  const preferredCandidates = legalCandidates.filter(
    (candidate) =>
      candidate.draftPoolSize >= PREFERRED_MIN_POOL_SIZE &&
      candidate.draftPoolSize <= PREFERRED_MAX_POOL_SIZE,
  );
  const selectedCandidate = chooseBestCandidate(
    preferredCandidates.length > 0 ? preferredCandidates : legalCandidates,
  );
  const doubledCardCount = countDoubledCards(
    selectedCandidate.draftPoolCopiesByCard,
  );
  const dreamsignPoolIds = dreamsignTemplates
    .filter((template) =>
      isPackageAdjacent(template.packageTides, selectedCandidate.selectedTides),
    )
    .map((template) => template.id);

  return {
    dreamcaller,
    mandatoryTides: [...dreamcaller.mandatoryTides],
    optionalSubset: [...selectedCandidate.optionalSubset],
    selectedTides: [...selectedCandidate.selectedTides],
    draftPoolCopiesByCard: selectedCandidate.draftPoolCopiesByCard,
    dreamsignPoolIds,
    mandatoryOnlyPoolSize,
    draftPoolSize: selectedCandidate.draftPoolSize,
    doubledCardCount,
    legalSubsetCount: legalCandidates.length,
    preferredSubsetCount: preferredCandidates.length,
  };
}

function buildDraftPoolCopies(
  cards: readonly CardData[],
  selectedPackageTides: readonly PackageTideId[],
): Record<string, number> {
  const draftPoolCopiesByCard: Record<string, number> = {};

  for (const card of cards) {
    const overlapCount = countPackageOverlap(card.tides, selectedPackageTides);
    const copies = Math.min(2, overlapCount);

    if (copies > 0) {
      draftPoolCopiesByCard[String(card.cardNumber)] = copies;
    }
  }

  return draftPoolCopiesByCard;
}

function countDraftPoolSize(draftPoolCopiesByCard: Record<string, number>): number {
  return Object.values(draftPoolCopiesByCard).reduce(
    (total, copies) => total + copies,
    0,
  );
}

function countDoubledCards(draftPoolCopiesByCard: Record<string, number>): number {
  return Object.values(draftPoolCopiesByCard).filter((copies) => copies === 2)
    .length;
}

function enumeratePackageCandidates(
  optionalTides: readonly PackageTideId[],
): PackageTideId[][] {
  return [
    ...buildCombinations(optionalTides, 3),
    ...buildCombinations(optionalTides, 4),
  ];
}

function buildCombinations<T>(values: readonly T[], size: number): T[][] {
  const combinations: T[][] = [];
  const current: T[] = [];

  function visit(startIndex: number) {
    if (current.length === size) {
      combinations.push([...current]);
      return;
    }

    for (
      let index = startIndex;
      index <= values.length - (size - current.length);
      index += 1
    ) {
      current.push(values[index]);
      visit(index + 1);
      current.pop();
    }
  }

  visit(0);
  return combinations;
}

function chooseBestCandidate<
  T extends { optionalSubset: readonly PackageTideId[]; draftPoolSize: number },
>(candidates: readonly T[]): T {
  return [...candidates].sort((left, right) => {
    if (right.draftPoolSize !== left.draftPoolSize) {
      return right.draftPoolSize - left.draftPoolSize;
    }

    return compareSubsetKeys(left.optionalSubset, right.optionalSubset);
  })[0];
}

function compareSubsetKeys(
  left: readonly PackageTideId[],
  right: readonly PackageTideId[],
): number {
  const leftKey = [...left].sort().join("|");
  const rightKey = [...right].sort().join("|");
  return leftKey.localeCompare(rightKey);
}
