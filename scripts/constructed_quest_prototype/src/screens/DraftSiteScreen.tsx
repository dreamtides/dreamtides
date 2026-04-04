import { useCallback } from "react";
import { useQuest } from "../state/quest-context";
import { logEvent } from "../logging";

/** Placeholder draft site screen. Will be rewritten in a later task. */
export function DraftSiteScreen({ siteId }: { siteId: string }) {
  const { mutations } = useQuest();

  const handleContinue = useCallback(() => {
    logEvent("site_completed", {
      siteType: "DraftSite",
      outcome: "auto-completed",
    });
    mutations.markSiteVisited(siteId);
    mutations.setScreen({ type: "dreamscape" });
  }, [siteId, mutations]);

  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <h2 className="text-2xl font-bold" style={{ color: "#a855f7" }}>
        Draft
      </h2>
      <p className="opacity-50">
        This site will be implemented in a later task.
      </p>
      <button
        className="rounded-lg px-5 py-2.5 font-medium text-white"
        style={{ backgroundColor: "#7c3aed" }}
        onClick={handleContinue}
      >
        Auto-complete
      </button>
    </div>
  );
}
