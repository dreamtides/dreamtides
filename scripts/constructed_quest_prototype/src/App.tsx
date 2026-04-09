import { useCallback, useEffect, useState } from "react";
import type { CardData } from "./types/cards";
import { loadCardDatabase } from "./data/card-database";
import { QuestProvider, useQuest } from "./state/quest-context";
import { ScreenRouter } from "./components/ScreenRouter";
import { HUD } from "./components/HUD";
import { DeckEditor } from "./components/DeckEditor";

/** Inner component that renders the screen router and HUD. */
function QuestApp({
  cardDatabase,
}: {
  cardDatabase: Map<number, CardData>;
}) {
  const { state } = useQuest();
  const showHud = state.screen.type !== "questStart" && state.screen.type !== "viewStartingDeck";
  const [deckEditorOpen, setDeckEditorOpen] = useState(false);

  const handleOpenDeckEditor = useCallback(() => {
    setDeckEditorOpen(true);
  }, []);

  const handleCloseDeckEditor = useCallback(() => {
    setDeckEditorOpen(false);
  }, []);

  return (
    <div style={{ paddingBottom: showHud ? "48px" : "0" }}>
      <ScreenRouter />
      {showHud && (
        <HUD
          onOpenDeckEditor={handleOpenDeckEditor}
        />
      )}
      <DeckEditor
        isOpen={deckEditorOpen}
        onClose={handleCloseDeckEditor}
        cardDatabase={cardDatabase}
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
