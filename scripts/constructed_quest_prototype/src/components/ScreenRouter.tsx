import { useCallback } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { AtlasScreen } from "../screens/AtlasScreen";
import { QuestStartScreen } from "../screens/QuestStartScreen";
import { QuestCompleteScreen } from "../screens/QuestCompleteScreen";
import { DreamscapeScreen } from "../screens/DreamscapeScreen";
import { DraftSiteScreen } from "../screens/DraftSiteScreen";
import { DreamcallerDraftScreen } from "../screens/DreamcallerDraftScreen";
import { BattleScreen } from "../screens/BattleScreen";
import { ShopScreen } from "../screens/ShopScreen";
import { EssenceSiteScreen } from "../screens/EssenceSiteScreen";
import { DreamsignOfferingScreen } from "../screens/DreamsignOfferingScreen";
import { DreamsignDraftScreen } from "../screens/DreamsignDraftScreen";
import { DreamJourneyScreen } from "../screens/DreamJourneyScreen";
import { TemptingOfferScreen } from "../screens/TemptingOfferScreen";
import { TransfigurationSiteScreen } from "../screens/TransfigurationSiteScreen";
import { DuplicationSiteScreen } from "../screens/DuplicationSiteScreen";
import { RewardSiteScreen } from "../screens/RewardSiteScreen";
import { CleanseSiteScreen } from "../screens/CleanseSiteScreen";
import { LootPackScreen } from "../screens/LootPackScreen";
import { PackShopScreen } from "../screens/PackShopScreen";
import { ForgeScreen } from "../screens/ForgeScreen";
import { ProvisionerScreen } from "../screens/ProvisionerScreen";
import { siteTypeName } from "../atlas/atlas-generator";
import { logEvent } from "../logging";
import type { Screen, SiteState } from "../types/quest";

/** Computes a stable key for AnimatePresence from the current screen. */
function screenKey(screen: Screen): string {
  if (screen.type === "site") {
    return `screen-site-${screen.siteId}`;
  }
  return `screen-${screen.type}`;
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
        key={screenKey(screen)}
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
  const { state, cardDatabase } = useQuest();
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

  if (site.type === "DraftSite") {
    return <DraftSiteScreen siteId={siteId} />;
  }

  if (site.type === "DreamcallerDraft") {
    return <DreamcallerDraftScreen site={site} />;
  }

  if (site.type === "Battle") {
    return <BattleScreen site={site} cardDatabase={cardDatabase} />;
  }

  if (site.type === "CardShop") {
    return <ShopScreen site={site} />;
  }

  if (site.type === "Essence") {
    return <EssenceSiteScreen site={site} />;
  }

  if (site.type === "DreamsignOffering") {
    return <DreamsignOfferingScreen site={site} />;
  }

  if (site.type === "DreamsignDraft") {
    return <DreamsignDraftScreen site={site} />;
  }

  if (site.type === "DreamJourney") {
    return <DreamJourneyScreen site={site} />;
  }

  if (site.type === "TemptingOffer") {
    return <TemptingOfferScreen site={site} />;
  }

  if (site.type === "Transfiguration") {
    return <TransfigurationSiteScreen site={site} />;
  }

  if (site.type === "Duplication") {
    return <DuplicationSiteScreen site={site} />;
  }

  if (site.type === "Reward") {
    return <RewardSiteScreen site={site} />;
  }

  if (site.type === "Cleanse") {
    return <CleanseSiteScreen site={site} />;
  }

  if (site.type === "LootPack") {
    return <LootPackScreen site={site} />;
  }

  if (site.type === "PackShop") {
    return <PackShopScreen site={site} />;
  }

  if (site.type === "Forge") {
    return <ForgeScreen site={site} />;
  }

  if (site.type === "Provisioner") {
    return <ProvisionerScreen site={site} />;
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

