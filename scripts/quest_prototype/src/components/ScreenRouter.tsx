import { useCallback } from "react";
import { AnimatePresence } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { AtlasScreen } from "../screens/AtlasScreen";
import { QuestStartScreen } from "../screens/QuestStartScreen";
import { generateNewNodes } from "../atlas/atlas-generator";

/** Shared layout for placeholder screens that will be built in later tasks. */
function PlaceholderScreen({
  title,
  titleColor = "#a855f7",
  titleSize = "text-2xl",
  buttonLabel,
  onButtonClick,
}: {
  title: string;
  titleColor?: string;
  titleSize?: string;
  buttonLabel: string;
  onButtonClick: () => void;
}) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4 p-8">
      <h2 className={`${titleSize} font-bold`} style={{ color: titleColor }}>
        {title}
      </h2>
      <p className="opacity-50">This screen will be implemented in a later task.</p>
      <button
        className="rounded-lg px-4 py-2 font-medium text-white"
        style={{ backgroundColor: "#7c3aed" }}
        onClick={onButtonClick}
      >
        {buttonLabel}
      </button>
    </div>
  );
}

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
  const { state, mutations } = useQuest();

  const handleCompleteDreamscape = useCallback(() => {
    const completedNodeId = state.currentDreamscape;
    if (completedNodeId) {
      const updatedAtlas = generateNewNodes(
        state.atlas,
        completedNodeId,
        state.completionLevel,
      );
      mutations.updateAtlas(updatedAtlas);
      mutations.setCurrentDreamscape(null);
    }
    mutations.setScreen({ type: "atlas" });
  }, [state.atlas, state.currentDreamscape, state.completionLevel, mutations]);

  return (
    <PlaceholderScreen
      title="Dreamscape View"
      buttonLabel="Complete Dreamscape"
      onButtonClick={handleCompleteDreamscape}
    />
  );
}

/** Placeholder for the site screen (built in a later task). */
function SitePlaceholder() {
  const { mutations } = useQuest();
  return (
    <PlaceholderScreen
      title="Site View"
      buttonLabel="Back to Dreamscape"
      onButtonClick={() => { mutations.setScreen({ type: "dreamscape" }); }}
    />
  );
}

/** Placeholder for the quest complete screen (built in a later task). */
function QuestCompletePlaceholder() {
  const { mutations } = useQuest();
  return (
    <PlaceholderScreen
      title="Quest Complete!"
      titleColor="#fbbf24"
      titleSize="text-3xl"
      buttonLabel="New Quest"
      onButtonClick={() => { mutations.resetQuest(); }}
    />
  );
}
