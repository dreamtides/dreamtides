import { useEffect, useState } from "react";
import type { CardData } from "./types/cards";
import { loadCardDatabase } from "./data/card-database";
import { QuestProvider, useQuest } from "./state/quest-context";

function QuestTestPanel() {
  const { state, mutations } = useQuest();

  return (
    <div className="flex flex-col items-center gap-4 p-8">
      <h1 className="text-4xl font-bold md:text-5xl lg:text-6xl">
        Dreamtides Quest Prototype
      </h1>
      <div className="mt-4 rounded border border-gray-600 p-4 text-sm">
        <p>Essence: {String(state.essence)}</p>
        <p>Deck size: {String(state.deck.length)}</p>
        <p>
          Dreamcaller: {state.dreamcaller?.name ?? "none"}
        </p>
        <p>Dreamsigns: {String(state.dreamsigns.length)}</p>
        <p>Completion level: {String(state.completionLevel)}</p>
        <p>Screen: {state.screen.type}</p>
      </div>
      <div className="flex flex-wrap gap-2">
        <button
          className="rounded bg-blue-600 px-3 py-1 text-white"
          onClick={() => {
            mutations.changeEssence(100, "test_source");
          }}
        >
          +100 Essence
        </button>
        <button
          className="rounded bg-blue-600 px-3 py-1 text-white"
          onClick={() => {
            mutations.changeEssence(-50, "shop_purchase");
          }}
        >
          -50 Essence
        </button>
        <button
          className="rounded bg-green-600 px-3 py-1 text-white"
          onClick={() => {
            mutations.addCard(1, "draft_pick");
          }}
        >
          Add Card #1
        </button>
        <button
          className="rounded bg-purple-600 px-3 py-1 text-white"
          onClick={() => {
            mutations.setScreen({ type: "atlas" });
          }}
        >
          Go to Atlas
        </button>
      </div>
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
    return <div className="p-8">Loading card database...</div>;
  }

  return (
    <QuestProvider cardDatabase={cardDatabase}>
      <QuestTestPanel />
    </QuestProvider>
  );
}
