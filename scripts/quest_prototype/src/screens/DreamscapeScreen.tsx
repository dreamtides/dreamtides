import { useCallback, useMemo } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { SiteCard } from "../components/SiteCard";
import { logEvent } from "../logging";

/** The dreamscape view: shows all sites in the current dreamscape. */
export function DreamscapeScreen() {
  const { state, mutations } = useQuest();
  const { currentDreamscape, completionLevel } = state;

  const node = currentDreamscape !== null ? state.atlas.nodes[currentDreamscape] : undefined;

  const allNonBattleVisited = useMemo(() => {
    if (!node) return false;
    return node.sites
      .filter((s) => s.type !== "Battle")
      .every((s) => s.isVisited);
  }, [node]);

  const handleSiteClick = useCallback(
    (siteId: string) => {
      if (!node) return;
      const site = node.sites.find((s) => s.id === siteId);
      if (!site) return;

      logEvent("site_entered", {
        siteType: site.type,
        dreamscapeId: node.id,
        siteId: site.id,
        isEnhanced: site.isEnhanced,
      });

      mutations.setScreen({ type: "site", siteId });
    },
    [node, mutations],
  );

  const handleReturnToAtlas = useCallback(() => {
    mutations.setCurrentDreamscape(null);
    mutations.setScreen({ type: "atlas" });
  }, [mutations]);

  if (!node) {
    return (
      <div className="flex h-full items-center justify-center p-8">
        <p className="text-lg opacity-50">No dreamscape selected.</p>
      </div>
    );
  }

  return (
    <motion.div
      className="flex h-full flex-col items-center px-4 py-6 md:px-8 md:py-8"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -20 }}
      transition={{ duration: 0.4 }}
    >
      {/* Biome header banner */}
      <div className="mb-6 text-center md:mb-8">
        <h2
          className="text-2xl font-bold tracking-wide md:text-3xl"
          style={{ color: node.biomeColor }}
        >
          {node.biomeName}
        </h2>
        <p className="mt-1 text-sm opacity-50">
          Visit all sites to unlock the battle
        </p>
      </div>

      {/* Site list */}
      <div className="flex w-full max-w-lg flex-col gap-3 md:gap-4">
        {node.sites.map((site) => {
          const isBattle = site.type === "Battle";
          const isLocked = isBattle && !allNonBattleVisited;

          return (
            <SiteCard
              key={site.id}
              site={site}
              isLocked={isLocked}
              completionLevel={completionLevel}
              biomeColor={node.biomeColor}
              onSiteClick={handleSiteClick}
            />
          );
        })}
      </div>

      {/* Escape hatch: return to atlas without completing */}
      <div className="mt-6 flex gap-3 md:mt-8">
        <button
          className="rounded-lg px-4 py-2 text-sm font-medium transition-colors md:text-base"
          style={{
            background: "rgba(107, 114, 128, 0.2)",
            border: "1px solid rgba(107, 114, 128, 0.4)",
            color: "#9ca3af",
          }}
          onClick={handleReturnToAtlas}
        >
          Return to Atlas
        </button>
      </div>
    </motion.div>
  );
}
