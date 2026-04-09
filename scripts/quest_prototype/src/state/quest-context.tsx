import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { CardData, NamedTide, Tide } from "../types/cards";
import type {
  DeckEntry,
  DreamAtlas,
  Dreamcaller,
  Dreamsign,
  QuestState,
  Screen,
  TransfigurationType,
} from "../types/quest";
import type { DraftState } from "../types/draft";
import { logEvent, resetLog } from "../logging";

const MAX_DREAMSIGNS = 12;

/** Mutation functions exposed by the quest context. */
export interface QuestMutations {
  changeEssence: (delta: number, source: string) => void;
  addCard: (cardNumber: number, source: string) => void;
  chooseStartingTide: (
    startingTide: NamedTide,
    starterCardNumbers: number[],
    tideCardNumbers: number[],
    neutralCardNumbers: number[],
    consumedRandomCardNumbers: number[],
  ) => void;
  addBaneCard: (cardNumber: number, source: string) => void;
  removeCard: (entryId: string, source: string) => void;
  transfigureCard: (
    entryId: string,
    type: TransfigurationType,
    effectDescription: string,
    effectDetails: Record<string, unknown>,
  ) => void;
  setDreamcaller: (dreamcaller: Dreamcaller) => void;
  addDreamsign: (dreamsign: Dreamsign, sourceSiteType: string) => void;
  removeDreamsign: (index: number, reason: string) => void;
  addTideCrystal: (tide: Tide, count: number) => void;
  incrementCompletionLevel: (
    essenceReward: number,
    rewardCardNumber: number | null,
    rewardCardName: string | null,
    isMiniboss: boolean,
  ) => void;
  setScreen: (screen: Screen) => void;
  markSiteVisited: (siteId: string) => void;
  setCurrentDreamscape: (nodeId: string | null) => void;
  updateAtlas: (atlas: DreamAtlas) => void;
  setDraftState: (draftState: DraftState) => void;
  setExcludedTides: (tides: Tide[]) => void;
  resetQuest: () => void;
}

/** The value provided by the quest context. */
export interface QuestContextValue {
  state: QuestState;
  mutations: QuestMutations;
  cardDatabase: Map<number, CardData>;
}

const QuestContext = createContext<QuestContextValue | null>(null);

function screenName(screen: Screen): string {
  return screen.type === "site" ? `site:${screen.siteId}` : screen.type;
}

function createDefaultState(): QuestState {
  return {
    essence: 250,
    deck: [],
    startingTide: null,
    consumedStartingCardNumbers: [],
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
      Neutral: 0,
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
    excludedTides: [],
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
  const entryIdCounter = useRef(0);

  function nextEntryId(): string {
    entryIdCounter.current += 1;
    return `deck-${String(entryIdCounter.current)}`;
  }

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
      const entryId = nextEntryId();
      setState((prev) => {
        const entry: DeckEntry = {
          entryId,
          cardNumber,
          transfiguration: null,
          isBane: false,
        };
        return { ...prev, deck: [...prev.deck, entry] };
      });
    },
    [cardDatabase],
  );

  const chooseStartingTide = useCallback(
    (
      startingTide: NamedTide,
      starterCardNumbers: number[],
      tideCardNumbers: number[],
      neutralCardNumbers: number[],
      consumedRandomCardNumbers: number[],
    ) => {
      const allCardNumbers = [
        ...starterCardNumbers,
        ...tideCardNumbers,
        ...neutralCardNumbers,
      ];
      const addedDeckEntries: DeckEntry[] = allCardNumbers.map(
        (cardNumber) => ({
          entryId: nextEntryId(),
          cardNumber,
          transfiguration: null,
          isBane: false,
        }),
      );

      logEvent("starting_tide_selected", {
        startingTide,
        grantedCrystal: startingTide,
      });
      logEvent("starting_deck_initialized", {
        startingTide,
        starterCardNumbers,
        tideCardNumbers,
        neutralCardNumbers,
        totalDeckSize: allCardNumbers.length,
      });

      setState((prev) => ({
        ...prev,
        startingTide,
        consumedStartingCardNumbers: consumedRandomCardNumbers,
        deck: [...prev.deck, ...addedDeckEntries],
        tideCrystals: {
          ...prev.tideCrystals,
          [startingTide]: prev.tideCrystals[startingTide] + 1,
        },
      }));
    },
    [],
  );

  const addBaneCard = useCallback(
    (cardNumber: number, source: string) => {
      const card = cardDatabase.get(cardNumber);
      const cardName = card?.name ?? `Unknown Card #${String(cardNumber)}`;
      logEvent("card_added", {
        cardNumber,
        cardName,
        source,
        isBane: true,
      });
      const entryId = nextEntryId();
      setState((prev) => {
        const entry: DeckEntry = {
          entryId,
          cardNumber,
          transfiguration: null,
          isBane: true,
        };
        return { ...prev, deck: [...prev.deck, entry] };
      });
    },
    [cardDatabase],
  );

  const removeCard = useCallback(
    (entryId: string, source: string) => {
      setState((prev) => {
        const entry = prev.deck.find((e) => e.entryId === entryId);
        if (!entry) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        const cardName =
          card?.name ?? `Unknown Card #${String(entry.cardNumber)}`;
        logEvent("card_removed", {
          cardNumber: entry.cardNumber,
          cardName,
          source,
        });
        const deck = prev.deck.filter((e) => e.entryId !== entryId);
        return { ...prev, deck };
      });
    },
    [cardDatabase],
  );

  const transfigureCard = useCallback(
    (
      entryId: string,
      type: TransfigurationType,
      effectDescription: string,
      effectDetails: Record<string, unknown>,
    ) => {
      setState((prev) => {
        const entry = prev.deck.find((e) => e.entryId === entryId);
        if (!entry) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        const cardName =
          card?.name ?? `Unknown Card #${String(entry.cardNumber)}`;
        logEvent("card_transfigured", {
          cardNumber: entry.cardNumber,
          cardName,
          transfigurationType: type,
          effectDescription,
          modifiedFields: effectDetails,
        });
        const deck = prev.deck.map((e) =>
          e.entryId === entryId ? { ...e, transfiguration: type } : e,
        );
        return { ...prev, deck };
      });
    },
    [cardDatabase],
  );

  const setDreamcaller = useCallback((dreamcaller: Dreamcaller) => {
    logEvent("dreamcaller_selected", {
      name: dreamcaller.name,
      tides: dreamcaller.tides,
      essenceBonus: dreamcaller.essenceBonus,
      tideCrystalGrant: dreamcaller.tideCrystalGrant,
    });
    setState((prev) => ({ ...prev, dreamcaller }));
  }, []);

  const addDreamsign = useCallback(
    (dreamsign: Dreamsign, sourceSiteType: string) => {
      setState((prev) => {
        if (prev.dreamsigns.length >= MAX_DREAMSIGNS) return prev;
        logEvent("dreamsign_acquired", {
          name: dreamsign.name,
          tide: dreamsign.tide,
          isBane: dreamsign.isBane,
          sourceSiteType,
        });
        return { ...prev, dreamsigns: [...prev.dreamsigns, dreamsign] };
      });
    },
    [],
  );

  const removeDreamsign = useCallback((index: number, reason: string) => {
    setState((prev) => {
      const dreamsign = prev.dreamsigns[index];
      if (!dreamsign) return prev;
      logEvent("dreamsign_removed", {
        name: dreamsign.name,
        tide: dreamsign.tide,
        reason,
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

  const incrementCompletionLevel = useCallback(
    (
      essenceReward: number,
      rewardCardNumber: number | null,
      rewardCardName: string | null,
      isMiniboss: boolean,
    ) => {
      setState((prev) => {
        const newLevel = prev.completionLevel + 1;
        logEvent("battle_won", {
          completionLevel: newLevel,
          essenceReward,
          rewardCardNumber,
          rewardCardName,
          isMiniboss,
        });
        const screen: Screen =
          newLevel >= 7 ? { type: "questComplete" } : prev.screen;
        if (newLevel >= 7) {
          logEvent("screen_transition", {
            from: screenName(prev.screen),
            to: screenName(screen),
          });
        }
        return { ...prev, completionLevel: newLevel, screen };
      });
    },
    [],
  );

  const setScreen = useCallback((screen: Screen) => {
    setState((prev) => {
      logEvent("screen_transition", {
        from: screenName(prev.screen),
        to: screenName(screen),
      });
      const activeSiteId =
        screen.type === "site" ? screen.siteId : null;
      return { ...prev, screen, activeSiteId };
    });
  }, []);

  const markSiteVisited = useCallback((siteId: string) => {
    setState((prev) => {
      if (prev.visitedSites.includes(siteId)) return prev;
      logEvent("site_visited", { siteId });
      const updatedNodes = { ...prev.atlas.nodes };
      for (const [nodeId, node] of Object.entries(updatedNodes)) {
        const siteIndex = node.sites.findIndex((s) => s.id === siteId);
        if (siteIndex !== -1) {
          const updatedSites = node.sites.map((s, i) =>
            i === siteIndex ? { ...s, isVisited: true } : s,
          );
          updatedNodes[nodeId] = { ...node, sites: updatedSites };
          break;
        }
      }
      return {
        ...prev,
        visitedSites: [...prev.visitedSites, siteId],
        atlas: { ...prev.atlas, nodes: updatedNodes },
      };
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
      }
      return {
        ...prev,
        currentDreamscape: nodeId,
        visitedSites: nodeId !== null ? [] : prev.visitedSites,
      };
    });
  }, []);

  const updateAtlas = useCallback((atlas: DreamAtlas) => {
    setState((prev) => ({ ...prev, atlas }));
  }, []);

  const setDraftState = useCallback((draftState: DraftState) => {
    setState((prev) => ({ ...prev, draftState }));
  }, []);

  const setExcludedTides = useCallback((tides: Tide[]) => {
    logEvent("tides_excluded", { excludedTides: tides });
    setState((prev) => ({ ...prev, excludedTides: tides }));
  }, []);

  const resetQuest = useCallback(() => {
    resetLog();
    entryIdCounter.current = 0;
    setState(createDefaultState());
  }, []);

  const mutations = useMemo<QuestMutations>(
    () => ({
      changeEssence,
      addCard,
      chooseStartingTide,
      addBaneCard,
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
      updateAtlas,
      setDraftState,
      setExcludedTides,
      resetQuest,
    }),
    [
      changeEssence,
      addCard,
      chooseStartingTide,
      addBaneCard,
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
      updateAtlas,
      setDraftState,
      setExcludedTides,
      resetQuest,
    ],
  );

  const value = useMemo<QuestContextValue>(
    () => ({ state, mutations, cardDatabase }),
    [state, mutations, cardDatabase],
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
