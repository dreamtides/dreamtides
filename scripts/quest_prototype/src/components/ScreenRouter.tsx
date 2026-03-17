import { useCallback } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { AtlasScreen } from "../screens/AtlasScreen";
import { QuestStartScreen } from "../screens/QuestStartScreen";
import { QuestCompleteScreen } from "../screens/QuestCompleteScreen";
import { DreamscapeScreen } from "../screens/DreamscapeScreen";
import { DraftSiteScreen } from "../screens/DraftSiteScreen";
import { ShopScreen } from "../screens/ShopScreen";
import { SpecialtyShopScreen } from "../screens/SpecialtyShopScreen";
import { generateNewNodes, siteTypeName } from "../atlas/atlas-generator";
import { logEvent } from "../logging";
import type { SiteState } from "../types/quest";

/** Computes a stable key for AnimatePresence from the current screen type. */
function screenKey(screenType: string): string {
  return `screen-${screenType}`;
}

/** Routes to the correct screen component based on quest state. */
export function ScreenRouter() {
  const { state } = useQuest();
  const { screen } = state;

  function renderScreen() {
    switch (screen.type) {
      case "questStart":
        return <QuestStartScreen />;
      case "atlas":
        return <AtlasScreen />;
      case "dreamscape":
        return <DreamscapeScreen />;
      case "site":
        return <SiteScreen siteId={screen.siteId} />;
      case "questComplete":
        return <QuestCompleteScreen />;
    }
  }

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={screenKey(screen.type)}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.35 }}
      >
        {renderScreen()}
      </motion.div>
    </AnimatePresence>
  );
}

/** Resolves the active site from state and renders the appropriate screen. */
function SiteScreen({ siteId }: { siteId: string }) {
  const { state } = useQuest();
  const { atlas, currentDreamscape } = state;

  const node = currentDreamscape !== null ? atlas.nodes[currentDreamscape] : undefined;
  const site = node?.sites.find((s) => s.id === siteId);

  if (!site) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <p className="text-lg opacity-50">Site not found.</p>
      </div>
    );
  }

  if (site.type === "Draft") {
    return <DraftSiteScreen siteId={siteId} />;
  }

  if (site.type === "Battle") {
    return <BattleSitePlaceholder site={site} />;
  }

  if (site.type === "Shop") {
    return <ShopScreen site={site} />;
  }

  if (site.type === "SpecialtyShop") {
    return <SpecialtyShopScreen site={site} />;
  }

  return <GenericSitePlaceholder site={site} />;
}

/** Auto-complete placeholder for non-battle site types. */
function GenericSitePlaceholder({ site }: { site: SiteState }) {
  const { mutations } = useQuest();

  const handleAutoComplete = useCallback(() => {
    logEvent("site_completed", {
      siteType: site.type,
      outcome: "auto-completed",
    });
    mutations.markSiteVisited(site.id);
    mutations.setScreen({ type: "dreamscape" });
  }, [site, mutations]);

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <h2 className="text-2xl font-bold" style={{ color: "#a855f7" }}>
        {siteTypeName(site.type)}
      </h2>
      {site.isEnhanced && (
        <span
          className="rounded-full px-3 py-1 text-sm font-bold"
          style={{
            background: "rgba(168, 85, 247, 0.15)",
            color: "#c084fc",
            border: "1px solid rgba(168, 85, 247, 0.3)",
          }}
        >
          {"\u2B50"} Enhanced
        </span>
      )}
      <p className="opacity-50">
        This site will be implemented in a later task.
      </p>
      <button
        className="rounded-lg px-5 py-2.5 font-medium text-white"
        style={{ backgroundColor: "#7c3aed" }}
        onClick={handleAutoComplete}
      >
        Auto-complete
      </button>
    </div>
  );
}

/**
 * Auto-complete placeholder for battle sites. Calls the same state
 * mutations that the real BattleScreen will use, so swapping this
 * out only requires changing the ScreenRouter wire-up.
 */
function BattleSitePlaceholder({ site }: { site: SiteState }) {
  const { state, mutations } = useQuest();
  const { atlas, currentDreamscape, completionLevel } = state;

  const essenceReward = 100 + completionLevel * 50;
  const isFinalBoss = completionLevel >= 6;
  const isMiniboss = completionLevel >= 3 && completionLevel < 6;

  let titleLabel = "Battle";
  let titleColor = "#ef4444";
  if (isFinalBoss) {
    titleLabel = "Final Boss";
    titleColor = "#fbbf24";
  } else if (isMiniboss) {
    titleLabel = "Miniboss";
    titleColor = "#ef4444";
  }

  const handleAutoComplete = useCallback(() => {
    logEvent("battle_started", {
      completionLevel,
      enemyName: "Auto-resolved",
      isMiniboss,
      isFinalBoss,
    });

    mutations.changeEssence(essenceReward, "battle_reward");

    logEvent("site_completed", {
      siteType: "Battle",
      outcome: `Victory - earned ${String(essenceReward)} essence`,
    });

    mutations.markSiteVisited(site.id);

    // Complete the dreamscape: generate new atlas nodes and mark
    // this node as completed before incrementing completion level,
    // since the final boss transition to questComplete needs the
    // atlas to already reflect the completed dreamscape.
    if (currentDreamscape) {
      const node = atlas.nodes[currentDreamscape];
      const updatedAtlas = generateNewNodes(
        atlas,
        currentDreamscape,
        completionLevel,
      );
      mutations.updateAtlas(updatedAtlas);

      logEvent("dreamscape_completed", {
        dreamscapeId: currentDreamscape,
        sitesVisitedCount: (node?.sites.length ?? 0),
      });

      mutations.setCurrentDreamscape(null);
    }

    // incrementCompletionLevel handles the quest-complete screen
    // transition for the final boss (level 7).
    mutations.incrementCompletionLevel(essenceReward, null);

    if (!isFinalBoss) {
      mutations.setScreen({ type: "atlas" });
    }
  }, [atlas, currentDreamscape, completionLevel, isMiniboss, isFinalBoss, essenceReward, site, mutations]);

  return (
    <div className="flex h-full flex-col items-center justify-center gap-5 p-8">
      <div className="text-4xl">{"\u2694\uFE0F"}</div>
      <h2 className="text-2xl font-bold" style={{ color: titleColor }}>
        {titleLabel}
      </h2>
      <p className="text-sm opacity-60">
        Completion Level: {String(completionLevel)}/7
      </p>
      <p className="text-sm" style={{ color: "#fbbf24" }}>
        Reward: {String(essenceReward)} essence
      </p>
      <p className="opacity-50">
        The battle screen will be implemented in a later task.
      </p>
      <button
        className="rounded-lg px-5 py-2.5 font-bold text-white"
        style={{
          background: `linear-gradient(135deg, ${titleColor} 0%, ${titleColor}cc 100%)`,
        }}
        onClick={handleAutoComplete}
      >
        Auto-complete
      </button>
    </div>
  );
}
