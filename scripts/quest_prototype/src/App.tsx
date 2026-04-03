import { useCallback, useEffect, useState } from "react";
import type { CardData } from "./types/cards";
import { loadCardDatabase } from "./data/card-database";
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
        excludedTides={state.excludedTides}
      />
    </div>
  );
}

export default function App() {
  const [cardDatabase, setCardDatabase] =
    useState<Map<number, CardData> | null>(null);

  useEffect(() => {
    loadCardDatabase()
      .then(setCardDatabase)
      .catch(() => {
        setCardDatabase(new Map());
      });
  }, []);

  if (cardDatabase === null) {
    return (
      <div className="flex h-screen items-center justify-center p-8">
        <p className="text-lg opacity-60">Loading card database...</p>
      </div>
    );
  }

  return (
    <QuestProvider cardDatabase={cardDatabase}>
      <QuestApp cardDatabase={cardDatabase} />
    </QuestProvider>
  );
}
