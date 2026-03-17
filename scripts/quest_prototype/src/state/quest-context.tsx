import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { CardData, Tide } from "../types/cards";
import type {
  DeckEntry,
  Dreamcaller,
  Dreamsign,
  QuestState,
  Screen,
  TransfigurationType,
} from "../types/quest";
import { logEvent, resetLog } from "../logging";

const MAX_DREAMSIGNS = 12;

/** Mutation functions exposed by the quest context. */
export interface QuestMutations {
  changeEssence: (delta: number, source: string) => void;
  addCard: (cardNumber: number, source: string) => void;
  removeCard: (deckIndex: number, source: string) => void;
  transfigureCard: (deckIndex: number, type: TransfigurationType) => void;
  setDreamcaller: (dreamcaller: Dreamcaller) => void;
  addDreamsign: (dreamsign: Dreamsign) => void;
  removeDreamsign: (index: number) => void;
  addTideCrystal: (tide: Tide, count: number) => void;
  incrementCompletionLevel: () => void;
  setScreen: (screen: Screen) => void;
  markSiteVisited: (siteId: string) => void;
  setCurrentDreamscape: (nodeId: string | null) => void;
  resetQuest: () => void;
}

/** The value provided by the quest context. */
export interface QuestContextValue {
  state: QuestState;
  mutations: QuestMutations;
}

const QuestContext = createContext<QuestContextValue | null>(null);

function screenName(screen: Screen): string {
  return screen.type === "site" ? `site:${screen.siteId}` : screen.type;
}

function createDefaultState(): QuestState {
  return {
    essence: 250,
    deck: [],
    dreamcaller: null,
    dreamsigns: [],
    tideCrystals: {
      Bloom: 0,
      Arc: 0,
      Ignite: 0,
      Pact: 0,
      Umbra: 0,
      Rime: 0,
      Surge: 0,
      Wild: 0,
    },
    completionLevel: 0,
    atlas: {
      nodes: {},
      edges: [],
      nexusId: "",
    },
    currentDreamscape: null,
    visitedSites: [],
    draftState: null,
    screen: { type: "questStart" },
    activeSiteId: null,
  };
}

/** Provides quest state and mutation functions to the component tree. */
export function QuestProvider({
  children,
  cardDatabase,
}: {
  children: ReactNode;
  cardDatabase: Map<number, CardData>;
}) {
  const [state, setState] = useState<QuestState>(createDefaultState);

  const changeEssence = useCallback((delta: number, source: string) => {
    setState((prev) => {
      const oldValue = prev.essence;
      const newValue = oldValue + delta;
      logEvent("essence_changed", {
        oldValue,
        newValue,
        delta,
        source,
      });
      return { ...prev, essence: newValue };
    });
  }, []);

  const addCard = useCallback(
    (cardNumber: number, source: string) => {
      const card = cardDatabase.get(cardNumber);
      const cardName = card?.name ?? `Unknown Card #${String(cardNumber)}`;
      logEvent("card_added", {
        cardNumber,
        cardName,
        source,
      });
      setState((prev) => {
        const entry: DeckEntry = {
          cardNumber,
          transfiguration: null,
          isBane: false,
        };
        return { ...prev, deck: [...prev.deck, entry] };
      });
    },
    [cardDatabase],
  );

  const removeCard = useCallback(
    (deckIndex: number, source: string) => {
      setState((prev) => {
        const entry = prev.deck[deckIndex];
        if (!entry) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        const cardName =
          card?.name ?? `Unknown Card #${String(entry.cardNumber)}`;
        logEvent("card_removed", {
          cardNumber: entry.cardNumber,
          cardName,
          source,
        });
        const deck = prev.deck.filter((_, i) => i !== deckIndex);
        return { ...prev, deck };
      });
    },
    [cardDatabase],
  );

  const transfigureCard = useCallback(
    (deckIndex: number, type: TransfigurationType) => {
      setState((prev) => {
        const entry = prev.deck[deckIndex];
        if (!entry) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        const cardName =
          card?.name ?? `Unknown Card #${String(entry.cardNumber)}`;
        logEvent("card_transfigured", {
          cardNumber: entry.cardNumber,
          cardName,
          transfigurationType: type,
        });
        const deck = prev.deck.map((e, i) =>
          i === deckIndex ? { ...e, transfiguration: type } : e,
        );
        return { ...prev, deck };
      });
    },
    [cardDatabase],
  );

  const setDreamcaller = useCallback((dreamcaller: Dreamcaller) => {
    logEvent("dreamcaller_selected", {
      name: dreamcaller.name,
      tide: dreamcaller.tide,
      essenceBonus: dreamcaller.essenceBonus,
    });
    setState((prev) => ({ ...prev, dreamcaller }));
  }, []);

  const addDreamsign = useCallback((dreamsign: Dreamsign) => {
    setState((prev) => {
      if (prev.dreamsigns.length >= MAX_DREAMSIGNS) return prev;
      logEvent("dreamsign_acquired", {
        name: dreamsign.name,
        tide: dreamsign.tide,
        isBane: dreamsign.isBane,
      });
      return { ...prev, dreamsigns: [...prev.dreamsigns, dreamsign] };
    });
  }, []);

  const removeDreamsign = useCallback((index: number) => {
    setState((prev) => {
      const dreamsign = prev.dreamsigns[index];
      if (!dreamsign) return prev;
      logEvent("dreamsign_removed", {
        name: dreamsign.name,
        tide: dreamsign.tide,
      });
      const dreamsigns = prev.dreamsigns.filter((_, i) => i !== index);
      return { ...prev, dreamsigns };
    });
  }, []);

  const addTideCrystal = useCallback((tide: Tide, count: number) => {
    logEvent("tide_crystal_added", { tide, count });
    setState((prev) => ({
      ...prev,
      tideCrystals: {
        ...prev.tideCrystals,
        [tide]: prev.tideCrystals[tide] + count,
      },
    }));
  }, []);

  const incrementCompletionLevel = useCallback(() => {
    setState((prev) => {
      const newLevel = prev.completionLevel + 1;
      logEvent("battle_won", {
        completionLevel: newLevel,
      });
      return { ...prev, completionLevel: newLevel };
    });
  }, []);

  const setScreen = useCallback((screen: Screen) => {
    setState((prev) => {
      logEvent("screen_transition", {
        from: screenName(prev.screen),
        to: screenName(screen),
      });
      return { ...prev, screen };
    });
  }, []);

  const markSiteVisited = useCallback((siteId: string) => {
    setState((prev) => {
      if (prev.visitedSites.includes(siteId)) return prev;
      return { ...prev, visitedSites: [...prev.visitedSites, siteId] };
    });
  }, []);

  const setCurrentDreamscape = useCallback((nodeId: string | null) => {
    setState((prev) => {
      if (nodeId !== null) {
        const node = prev.atlas.nodes[nodeId];
        logEvent("dreamscape_entered", {
          dreamscapeId: nodeId,
          biomeName: node?.biomeName ?? "unknown",
        });
      } else if (prev.currentDreamscape !== null) {
        logEvent("dreamscape_completed", {
          dreamscapeId: prev.currentDreamscape,
          sitesVisitedCount: prev.visitedSites.length,
        });
      }
      return {
        ...prev,
        currentDreamscape: nodeId,
        visitedSites: nodeId !== null ? [] : prev.visitedSites,
      };
    });
  }, []);

  const resetQuest = useCallback(() => {
    resetLog();
    setState(createDefaultState());
  }, []);

  const mutations = useMemo<QuestMutations>(
    () => ({
      changeEssence,
      addCard,
      removeCard,
      transfigureCard,
      setDreamcaller,
      addDreamsign,
      removeDreamsign,
      addTideCrystal,
      incrementCompletionLevel,
      setScreen,
      markSiteVisited,
      setCurrentDreamscape,
      resetQuest,
    }),
    [
      changeEssence,
      addCard,
      removeCard,
      transfigureCard,
      setDreamcaller,
      addDreamsign,
      removeDreamsign,
      addTideCrystal,
      incrementCompletionLevel,
      setScreen,
      markSiteVisited,
      setCurrentDreamscape,
      resetQuest,
    ],
  );

  const value = useMemo<QuestContextValue>(
    () => ({ state, mutations }),
    [state, mutations],
  );

  return (
    <QuestContext.Provider value={value}>{children}</QuestContext.Provider>
  );
}

/** Hook to access the quest state and mutation functions. */
export function useQuest(): QuestContextValue {
  const context = useContext(QuestContext);
  if (context === null) {
    throw new Error("useQuest must be used within a QuestProvider");
  }
  return context;
}
