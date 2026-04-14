import { useCallback, useEffect, useState } from "react";
import type { CardData } from "./types/cards";
import type { QuestContent } from "./data/quest-content";
import { loadQuestContent } from "./data/quest-content";
import { QuestProvider, useQuest } from "./state/quest-context";
import { ScreenRouter } from "./components/ScreenRouter";
import { HUD } from "./components/HUD";
import { DeckViewer } from "./components/DeckViewer";
import { DebugScreen } from "./screens/DebugScreen";

/** Inner component that renders the screen router and HUD. */
function QuestApp({
  cardDatabase,
}: {
  cardDatabase: Map<number, CardData>;
}) {
  const { state } = useQuest();
  const showHud = state.screen.type !== "questStart";
  const [deckViewerOpen, setDeckViewerOpen] = useState(false);
  const [debugScreenOpen, setDebugScreenOpen] = useState(false);

  const hasDraftData = state.draftState !== null &&
    state.draftState.draftedCards.length > 0;

  const handleOpenDeckViewer = useCallback(() => {
    setDeckViewerOpen(true);
  }, []);

  const handleCloseDeckViewer = useCallback(() => {
    setDeckViewerOpen(false);
  }, []);

  const handleOpenDebugScreen = useCallback(() => {
    setDebugScreenOpen(true);
  }, []);

  const handleCloseDebugScreen = useCallback(() => {
    setDebugScreenOpen(false);
  }, []);

  return (
    <div style={{ paddingBottom: showHud ? "48px" : "0" }}>
      <ScreenRouter />
      {showHud && (
        <HUD
          onOpenDeckViewer={handleOpenDeckViewer}
          onOpenDebugScreen={handleOpenDebugScreen}
          hasDraftData={hasDraftData}
        />
      )}
      <DeckViewer
        isOpen={deckViewerOpen}
        onClose={handleCloseDeckViewer}
        cardDatabase={cardDatabase}
      />
      <DebugScreen
        isOpen={debugScreenOpen}
        onClose={handleCloseDebugScreen}
        draftState={state.draftState}
        cardDatabase={cardDatabase}
        chosenTide={state.chosenTide}
      />
    </div>
  );
}

export default function App() {
  const [questContent, setQuestContent] = useState<QuestContent | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    loadQuestContent()
      .then((content) => {
        setQuestContent(content);
        setLoadError(null);
      })
      .catch((error) => {
        setLoadError(
          error instanceof Error ? error.message : "Failed to load quest content.",
        );
      });
  }, []);

  if (loadError !== null) {
    return (
      <div className="flex h-screen items-center justify-center p-8">
        <p className="max-w-2xl text-center text-sm whitespace-pre-wrap opacity-70">
          {loadError}
        </p>
      </div>
    );
  }

  if (questContent === null) {
    return (
      <div className="flex h-screen items-center justify-center p-8">
        <p className="text-lg opacity-60">Loading quest content...</p>
      </div>
    );
  }

  return (
    <QuestProvider
      cardDatabase={questContent.cardDatabase}
      questContent={questContent}
    >
      <QuestApp cardDatabase={questContent.cardDatabase} />
    </QuestProvider>
  );
}
