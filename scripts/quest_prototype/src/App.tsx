import { useEffect, useState } from "react";
import type { CardData } from "./types/cards";
import { loadCardDatabase } from "./data/card-database";
import { QuestProvider } from "./state/quest-context";
import { ScreenRouter } from "./components/ScreenRouter";

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
      <div className="flex h-screen flex-col">
        <ScreenRouter />
      </div>
    </QuestProvider>
  );
}
