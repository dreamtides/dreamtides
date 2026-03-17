import { useCallback, useMemo } from "react";
import { motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { SiteCard } from "../components/SiteCard";
import { logEvent } from "../logging";
import { generateNewNodes } from "../atlas/atlas-generator";

/** The dreamscape view: shows all sites in the current dreamscape. */
export function DreamscapeScreen() {
  const { state, mutations } = useQuest();
  const { atlas, currentDreamscape, completionLevel } = state;

  const node = currentDreamscape !== null ? atlas.nodes[currentDreamscape] : undefined;

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

  // Handle dreamscape completion: generates new atlas nodes, marks the
  // node as completed, and transitions back to the atlas.
  // The dreamscape_completed event is logged by setCurrentDreamscape(null).
  const handleDreamscapeCompletion = useCallback(() => {
    if (!currentDreamscape) return;

    const updatedAtlas = generateNewNodes(
      atlas,
      currentDreamscape,
      completionLevel,
    );
    mutations.updateAtlas(updatedAtlas);
    mutations.setCurrentDreamscape(null);
    mutations.setScreen({ type: "atlas" });
  }, [atlas, currentDreamscape, completionLevel, mutations]);

  const allSitesVisited = useMemo(() => {
    if (!node) return false;
    return node.sites.every((s) => s.isVisited);
  }, [node]);

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

      {/* Action buttons */}
      <div className="mt-6 flex gap-3 md:mt-8">
        {allSitesVisited && (
          <button
            className="rounded-lg px-5 py-2.5 text-sm font-bold text-white transition-colors md:text-base"
            style={{
              background: "linear-gradient(135deg, #7c3aed 0%, #a855f7 100%)",
            }}
            onClick={handleDreamscapeCompletion}
          >
            Complete Dreamscape
          </button>
        )}
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
