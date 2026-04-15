import { generateInitialAtlas } from "../atlas/atlas-generator";
import { createDreamsign } from "../data/dreamsigns";
import { initializeDraftState } from "../draft/draft-engine";
import { resolveDreamsignTemplates } from "../dreamsign/dreamsign-pool";
import { logEvent } from "../logging";
import type { QuestContent } from "../data/quest-content";
import type { CardData } from "../types/cards";
import type { DreamcallerContent } from "../types/content";
import type { QuestMutations } from "../state/quest-context";
import type { QuestState } from "../types/quest";

export interface QuestStartBootstrapArgs {
  dreamcaller: DreamcallerContent;
  state: Pick<
    QuestState,
    "completionLevel" | "deck" | "dreamsigns" | "essence"
  >;
  mutations: Pick<
    QuestMutations,
    | "setCurrentDreamscape"
    | "setDraftState"
    | "setDreamcallerSelection"
    | "setScreen"
    | "updateAtlas"
  >;
  cardDatabase: Map<number, CardData>;
  questContent: Pick<
    QuestContent,
    "dreamsignTemplates" | "resolvedPackagesByDreamcallerId"
  >;
}

export function bootstrapQuestStart({
  dreamcaller,
  state,
  mutations,
  cardDatabase,
  questContent,
}: QuestStartBootstrapArgs): void {
  const resolvedPackage = questContent.resolvedPackagesByDreamcallerId.get(
    dreamcaller.id,
  );

  if (resolvedPackage === undefined) {
    throw new Error(`Missing resolved package for ${dreamcaller.id}`);
  }

  mutations.setDreamcallerSelection(resolvedPackage);

  const playerHasBanes =
    state.deck.some((entry) => entry.isBane) ||
    state.dreamsigns.some((dreamsign) => dreamsign.isBane);
  const atlas = generateInitialAtlas(state.completionLevel, {
    cardDatabase,
    dreamsignPool: resolveDreamsignTemplates(
      resolvedPackage.dreamsignPoolIds,
      questContent.dreamsignTemplates,
    ).map((template) => createDreamsign(template)),
    playerHasBanes,
    selectedPackageTides: resolvedPackage.selectedTides,
  });
  const draftState = initializeDraftState(cardDatabase, resolvedPackage);

  mutations.setDraftState(draftState, "quest_start");
  mutations.updateAtlas(atlas);

  const firstNode = Object.values(atlas.nodes).find(
    (node) => node.status === "available",
  );

  logEvent("quest_started", {
    initialEssence: state.essence,
    dreamcallerId: dreamcaller.id,
    dreamcallerName: dreamcaller.name,
    dreamcallerAwakening: dreamcaller.awakening,
    packageSummary: {
      mandatoryTides: resolvedPackage.mandatoryTides,
      optionalSubset: resolvedPackage.optionalSubset,
      selectedTides: resolvedPackage.selectedTides,
    },
    selectedPackageTides: resolvedPackage.selectedTides,
    draftPoolSize: resolvedPackage.draftPoolSize,
    dreamsignPoolSize: resolvedPackage.dreamsignPoolIds.length,
    dreamscapesGenerated: Object.keys(atlas.nodes).length - 1,
  });

  if (firstNode !== undefined) {
    mutations.setCurrentDreamscape(firstNode.id);
    mutations.setScreen({ type: "dreamscape" });
    return;
  }

  mutations.setScreen({ type: "atlas" });
}
