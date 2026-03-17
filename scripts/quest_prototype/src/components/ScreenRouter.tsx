import { AnimatePresence } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { AtlasScreen } from "../screens/AtlasScreen";
import { QuestStartScreen } from "../screens/QuestStartScreen";

/** Routes to the correct screen component based on quest state. */
export function ScreenRouter() {
  const { state } = useQuest();

  return (
    <AnimatePresence mode="wait">
      {state.screen.type === "questStart" && (
        <QuestStartScreen key="questStart" />
      )}
      {state.screen.type === "atlas" && (
        <AtlasScreen key="atlas" />
      )}
      {state.screen.type === "dreamscape" && (
        <DreamscapePlaceholder key="dreamscape" />
      )}
      {state.screen.type === "site" && (
        <SitePlaceholder key="site" />
      )}
      {state.screen.type === "questComplete" && (
        <QuestCompletePlaceholder key="questComplete" />
      )}
    </AnimatePresence>
  );
}

/** Placeholder for the dreamscape screen (built in a later task). */
function DreamscapePlaceholder() {
  const { mutations } = useQuest();
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <h2 className="text-2xl font-bold" style={{ color: "#a855f7" }}>
        Dreamscape View
      </h2>
      <p className="opacity-50">This screen will be implemented in a later task.</p>
      <button
        className="rounded-lg px-4 py-2 font-medium text-white"
        style={{ backgroundColor: "#7c3aed" }}
        onClick={() => { mutations.setScreen({ type: "atlas" }); }}
      >
        Back to Atlas
      </button>
    </div>
  );
}

/** Placeholder for the site screen (built in a later task). */
function SitePlaceholder() {
  const { mutations } = useQuest();
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <h2 className="text-2xl font-bold" style={{ color: "#a855f7" }}>
        Site View
      </h2>
      <p className="opacity-50">This screen will be implemented in a later task.</p>
      <button
        className="rounded-lg px-4 py-2 font-medium text-white"
        style={{ backgroundColor: "#7c3aed" }}
        onClick={() => { mutations.setScreen({ type: "dreamscape" }); }}
      >
        Back to Dreamscape
      </button>
    </div>
  );
}

/** Placeholder for the quest complete screen (built in a later task). */
function QuestCompletePlaceholder() {
  const { mutations } = useQuest();
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <h2 className="text-3xl font-bold" style={{ color: "#fbbf24" }}>
        Quest Complete!
      </h2>
      <p className="opacity-50">This screen will be implemented in a later task.</p>
      <button
        className="rounded-lg px-4 py-2 font-medium text-white"
        style={{ backgroundColor: "#7c3aed" }}
        onClick={() => { mutations.resetQuest(); }}
      >
        New Quest
      </button>
    </div>
  );
}
