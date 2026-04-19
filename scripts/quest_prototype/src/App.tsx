import { useCallback, useEffect, useRef, useState } from "react";
import type { CardData } from "./types/cards";
import type { QuestContent } from "./data/quest-content";
import { loadQuestContent } from "./data/quest-content";
import { QuestProvider, useQuest } from "./state/quest-context";
import { ScreenRouter } from "./components/ScreenRouter";
import { HUD } from "./components/HUD";
import { DeckViewer } from "./components/DeckViewer";
import { DebugScreen } from "./screens/DebugScreen";
import { CardSourceOverlay } from "./screens/CardSourceOverlay";
import { STARTER_CARD_NUMBERS } from "./data/starter-cards";
import type { RuntimeConfig } from "./runtime/runtime-config";
import { createPlayableBattleBootstrapController } from "./runtime/playable-battle-bootstrap";
import type { QuestState } from "./types/quest";

/** Inner component that renders the screen router and HUD. */
export function QuestApp({
  cardDatabase,
  runtimeConfig,
}: {
  cardDatabase: Map<number, CardData>;
  runtimeConfig: RuntimeConfig;
}) {
  const { state, mutations, questContent } = useQuest();
  const showHud = state.screen.type !== "questStart"
    && !isPlayableBattleHudHidden(state, runtimeConfig);
  const [deckViewerOpen, setDeckViewerOpen] = useState(false);
  const [starterDeckIntroOpen, setStarterDeckIntroOpen] = useState(false);
  const [debugScreenOpen, setDebugScreenOpen] = useState(false);
  const [cardSourceOverlayOpen, setCardSourceOverlayOpen] = useState(false);
  const previousScreenTypeRef = useRef(state.screen.type);
  const playableBattleBootstrapRef = useRef(createPlayableBattleBootstrapController());

  useEffect(() => {
    if (
      runtimeConfig.battleMode !== "playable" ||
      !runtimeConfig.startInBattle ||
      playableBattleBootstrapRef.current.isDone()
    ) {
      return;
    }

    playableBattleBootstrapRef.current.advance({
      state,
      mutations,
      questContent,
      cardDatabase,
    });
  }, [
    runtimeConfig.battleMode,
    runtimeConfig.startInBattle,
    state,
    mutations,
    questContent,
    cardDatabase,
  ]);

  const hasDraftData = state.resolvedPackage !== null;
  const hasCardSourceDebug = state.cardSourceDebug !== null;

  useEffect(() => {
    // FIND-01-6 (Stage 4): do NOT auto-open the deck viewer when leaving the
    // quest-start screen. The mid-quest-start deck overlay hid the first
    // site beneath a blocking modal. The starter-deck reference is one
    // "View Deck" click away on the HUD; let the player land on the site
    // unobstructed. We still observe the transition to keep the ref
    // up-to-date in case future logic needs it.
    //
    // `STARTER_CARD_NUMBERS` import retained for future starter-deck flows
    // (e.g. per-dreamcaller tutorials); underscore prefix silences unused
    // warnings without deleting the import, which other tests still rely on.
    void STARTER_CARD_NUMBERS;
    previousScreenTypeRef.current = state.screen.type;
  }, [state.deck, state.dreamcaller, state.screen.type]);

  useEffect(() => {
    if (!hasCardSourceDebug) {
      setCardSourceOverlayOpen(false);
    }
  }, [hasCardSourceDebug]);

  const handleOpenDeckViewer = useCallback(() => {
    setDeckViewerOpen(true);
    setStarterDeckIntroOpen(false);
  }, []);

  const handleCloseDeckViewer = useCallback(() => {
    setDeckViewerOpen(false);
    setStarterDeckIntroOpen(false);
  }, []);

  const handleBeginQuest = useCallback(() => {
    setDeckViewerOpen(false);
    setStarterDeckIntroOpen(false);
  }, []);

  const handleOpenDebugScreen = useCallback(() => {
    setDebugScreenOpen(true);
  }, []);

  const handleCloseDebugScreen = useCallback(() => {
    setDebugScreenOpen(false);
  }, []);

  const handleToggleCardSourceOverlay = useCallback(() => {
    setCardSourceOverlayOpen((prev) => !prev);
  }, []);

  const handleCloseCardSourceOverlay = useCallback(() => {
    setCardSourceOverlayOpen(false);
  }, []);

  return (
    <div style={{ paddingBottom: showHud ? "48px" : "0" }}>
      <ScreenRouter runtimeConfig={runtimeConfig} />
      {showHud && (
        <HUD
          onOpenDeckViewer={handleOpenDeckViewer}
          onOpenDebugScreen={handleOpenDebugScreen}
          onToggleCardSourceOverlay={handleToggleCardSourceOverlay}
          hasDraftData={hasDraftData}
          hasCardSourceDebug={hasCardSourceDebug}
          isCardSourceOverlayOpen={cardSourceOverlayOpen}
        />
      )}
      <DeckViewer
        isOpen={deckViewerOpen}
        onClose={handleCloseDeckViewer}
        cardDatabase={cardDatabase}
        introMode={starterDeckIntroOpen}
        onBeginQuest={handleBeginQuest}
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
      <CardSourceOverlay
        cardSourceDebug={state.cardSourceDebug}
        isOpen={cardSourceOverlayOpen}
        onClose={handleCloseCardSourceOverlay}
      />
    </div>
  );
}

function isPlayableBattleHudHidden(
  state: QuestState,
  runtimeConfig: RuntimeConfig,
): boolean {
  if (runtimeConfig.battleMode !== "playable" || state.screen.type !== "site") {
    return false;
  }

  if (state.currentDreamscape === null) {
    return false;
  }

  const siteId = state.screen.siteId;
  return state.atlas.nodes[state.currentDreamscape]
    ?.sites.some((site) => site.id === siteId && site.type === "Battle")
    ?? false;
}

export default function App({ runtimeConfig }: { runtimeConfig: RuntimeConfig }) {
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
      runtimeConfig={runtimeConfig}
    >
      <QuestApp
        cardDatabase={questContent.cardDatabase}
        runtimeConfig={runtimeConfig}
      />
    </QuestProvider>
  );
}
