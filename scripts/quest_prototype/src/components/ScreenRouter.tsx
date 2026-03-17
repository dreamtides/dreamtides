import { useCallback } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useQuest } from "../state/quest-context";
import { AtlasScreen } from "../screens/AtlasScreen";
import { QuestStartScreen } from "../screens/QuestStartScreen";
import { QuestCompleteScreen } from "../screens/QuestCompleteScreen";
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
        return <DreamscapePlaceholder />;
      case "site":
        return <SitePlaceholder />;
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
