import { describe, expect, it } from "vitest";
import type { QuestContent } from "../data/quest-content";
import type { DreamcallerContent, ResolvedDreamcallerPackage } from "../types/content";
import { createStartInBattleState } from "./start-in-battle-state";

function makeDreamcaller(): DreamcallerContent {
  return {
    id: "dreamcaller-1",
    name: "Test Dreamcaller",
    title: "Caller of Tests",
    awakening: 4,
    renderedText: "Test ability.",
    imageNumber: "0001",
    mandatoryTides: ["Bloom"],
    optionalTides: ["Arc", "Ignite", "Pact", "Rime"],
  };
}

function makeResolvedPackage(): ResolvedDreamcallerPackage {
  const dreamcaller = makeDreamcaller();
  return {
    dreamcaller,
    mandatoryTides: ["Bloom"],
    optionalSubset: ["Arc", "Ignite", "Pact"],
    selectedTides: ["Bloom", "Arc", "Ignite", "Pact"],
    draftPoolCopiesByCard: { "101": 2 },
    dreamsignPoolIds: ["dreamsign-1", "dreamsign-2"],
    mandatoryOnlyPoolSize: 120,
    draftPoolSize: 200,
    doubledCardCount: 4,
    legalSubsetCount: 12,
    preferredSubsetCount: 6,
  };
}

function makeQuestContent(): QuestContent {
  const resolvedPackage = makeResolvedPackage();
  return {
    cardDatabase: new Map(),
    cardsByPackageTide: new Map(),
    dreamcallers: [resolvedPackage.dreamcaller],
    dreamsignTemplates: [],
    resolvedPackagesByDreamcallerId: new Map([
      [resolvedPackage.dreamcaller.id, resolvedPackage],
    ]),
  };
}

describe("createStartInBattleState", () => {
  it("builds a starter run already routed to the first battle site", () => {
    const state = createStartInBattleState(makeQuestContent());

    expect(state).not.toBeNull();
    expect(state?.deck).toHaveLength(10);
    expect(state?.dreamcaller?.id).toBe("dreamcaller-1");
    expect(state?.screen.type).toBe("site");
    expect(state?.currentDreamscape).not.toBeNull();
    expect(state?.activeSiteId).toBe(
      state?.screen.type === "site" ? state.screen.siteId : null,
    );

    const currentNode =
      state?.currentDreamscape === null || state === null
        ? undefined
        : state.atlas.nodes[state.currentDreamscape];
    const activeBattleSite = currentNode?.sites.find(
      (site) =>
        state?.screen.type === "site" &&
        site.id === state.screen.siteId &&
        site.type === "Battle",
    );

    expect(activeBattleSite?.type).toBe("Battle");
  });

  it("returns null when no resolved dreamcaller package is available", () => {
    const dreamcaller = makeDreamcaller();
    const questContent: QuestContent = {
      cardDatabase: new Map(),
      cardsByPackageTide: new Map(),
      dreamcallers: [dreamcaller],
      dreamsignTemplates: [],
      resolvedPackagesByDreamcallerId: new Map(),
    };

    expect(createStartInBattleState(questContent)).toBeNull();
  });
});
