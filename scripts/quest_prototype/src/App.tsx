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
  const { state, questContent } = useQuest();
  const showHud = state.screen.type !== "questStart";
  const [deckViewerOpen, setDeckViewerOpen] = useState(false);
  const [debugScreenOpen, setDebugScreenOpen] = useState(false);

  const hasDraftData = state.resolvedPackage !== null;

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
        resolvedPackage={state.resolvedPackage}
        remainingDreamsignPool={state.remainingDreamsignPool}
        dreamsignTemplates={questContent.dreamsignTemplates}
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
        <div
          role="alert"
          className="max-w-3xl w-full rounded-lg border border-red-500/60 bg-red-950/40 p-6 shadow-lg"
        >
          <h1 className="mb-3 text-xl font-semibold text-red-200">
            Quest content failed to load
          </h1>
          <pre className="max-h-[60vh] overflow-auto whitespace-pre-wrap rounded bg-black/40 p-4 font-mono text-xs text-red-100">
            {loadError}
          </pre>
          <div className="mt-4 flex gap-3">
            <button
              type="button"
              onClick={() => {
                window.location.reload();
              }}
              className="rounded bg-red-500 px-4 py-2 text-sm font-medium text-white hover:bg-red-400"
            >
              Retry
            </button>
            <button
              type="button"
              onClick={() => {
                void navigator.clipboard?.writeText(loadError);
              }}
              className="rounded border border-red-400/50 px-4 py-2 text-sm font-medium text-red-100 hover:bg-red-500/20"
            >
              Copy details
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (questContent === null) {
    return (
      <div className="flex h-screen flex-col items-center justify-center gap-3 p-8">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-slate-500 border-t-transparent" />
        <p className="text-lg opacity-80">Loading quest content...</p>
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
