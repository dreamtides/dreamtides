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
  AnteState,
  DeckEntry,
  DreamAtlas,
  Dreamcaller,
  Dreamsign,
  QuestState,
  Screen,
  SiteState,
  TransfigurationType,
} from "../types/quest";
import { logEvent, resetLog } from "../logging";
import type { QuestConfig } from "./quest-config";
import { generateInitialAtlas } from "../atlas/atlas-generator";
import { DREAMSIGNS } from "../data/dreamsigns";
import { NAMED_TIDES } from "../data/card-database";

const MAX_DREAMSIGNS = 12;

/** Mutation functions exposed by the quest context. */
export interface QuestMutations {
  changeEssence: (delta: number, source: string) => void;
  addToPool: (cardNumber: number, source: string) => void;
  addBaneToPool: (cardNumber: number, source: string) => void;
  removeCard: (entryId: string, source: string) => void;
  removeFromPool: (entryId: string, source: string) => void;
  transfigureCard: (
    entryId: string,
    type: TransfigurationType,
    effectDescription: string,
    effectDetails: Record<string, unknown>,
  ) => void;
  moveToDeck: (entryId: string) => void;
  moveToPool: (entryId: string) => void;
  moveAllToDeck: () => void;
  moveAllToPool: () => void;
  setDreamcaller: (dreamcaller: Dreamcaller) => void;
  addDreamsign: (dreamsign: Dreamsign, sourceSiteType: string) => void;
  removeDreamsign: (index: number, reason: string) => void;
  addTideCrystal: (tide: Tide, count: number) => void;
  incrementCompletionLevel: (isMiniboss: boolean) => void;
  setScreen: (screen: Screen) => void;
  markSiteVisited: (siteId: string) => void;
  setCurrentDreamscape: (nodeId: string | null) => void;
  updateAtlas: (atlas: DreamAtlas) => void;
  setStartingTide: (tide: NamedTide) => void;
  setAnteState: (anteState: AnteState | null) => void;
  addProvisionedSite: (dreamscapeId: string, site: SiteState) => void;
  initializeDeckFromPool: () => void;
  resetQuest: () => void;
  initializeQuest: (cardDatabase: Map<number, CardData>, config: QuestConfig) => void;
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
    pool: [],
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
    screen: { type: "dreamscape" },
    activeSiteId: null,
    startingTide: null,
    anteState: null,
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

  const addToPool = useCallback(
    (cardNumber: number, source: string) => {
      const card = cardDatabase.get(cardNumber);
      const cardName = card?.name ?? `Unknown Card #${String(cardNumber)}`;
      logEvent("card_added", {
        cardNumber,
        cardName,
        source,
      });
      setState((prev) => {
        const entryId = nextEntryId();
        const entry: DeckEntry = {
          entryId,
          cardNumber,
          transfiguration: null,
          isBane: false,
        };
        return { ...prev, pool: [...prev.pool, entry] };
      });
    },
    [cardDatabase],
  );

  const addBaneToPool = useCallback(
    (cardNumber: number, source: string) => {
      const card = cardDatabase.get(cardNumber);
      const cardName = card?.name ?? `Unknown Card #${String(cardNumber)}`;
      logEvent("card_added", {
        cardNumber,
        cardName,
        source,
        isBane: true,
      });
      setState((prev) => {
        const entryId = nextEntryId();
        const entry: DeckEntry = {
          entryId,
          cardNumber,
          transfiguration: null,
          isBane: true,
        };
        return {
          ...prev,
          deck: [...prev.deck, entry],
        };
      });
    },
    [cardDatabase],
  );

  const removeCard = useCallback(
    (entryId: string, source: string) => {
      setState((prev) => {
        const entry =
          prev.deck.find((e) => e.entryId === entryId) ??
          prev.pool.find((e) => e.entryId === entryId);
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
        const pool = prev.pool.filter((e) => e.entryId !== entryId);
        return { ...prev, deck, pool };
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

  const moveToDeck = useCallback(
    (entryId: string) => {
      setState((prev) => {
        const entry = prev.pool.find((e) => e.entryId === entryId);
        if (!entry) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        logEvent("card_moved_to_deck", {
          cardNumber: entry.cardNumber,
          cardName: card?.name ?? `Unknown Card #${String(entry.cardNumber)}`,
        });
        return {
          ...prev,
          pool: prev.pool.filter((e) => e.entryId !== entryId),
          deck: [...prev.deck, entry],
        };
      });
    },
    [cardDatabase],
  );

  const moveToPool = useCallback(
    (entryId: string) => {
      setState((prev) => {
        const entry = prev.deck.find((e) => e.entryId === entryId);
        if (!entry) return prev;
        if (entry.isBane) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        logEvent("card_moved_to_pool", {
          cardNumber: entry.cardNumber,
          cardName: card?.name ?? `Unknown Card #${String(entry.cardNumber)}`,
        });
        return {
          ...prev,
          deck: prev.deck.filter((e) => e.entryId !== entryId),
          pool: [...prev.pool, entry],
        };
      });
    },
    [cardDatabase],
  );

  const moveAllToDeck = useCallback(() => {
    setState((prev) => {
      if (prev.pool.length === 0) return prev;
      logEvent("all_cards_moved_to_deck", { count: prev.pool.length });
      return {
        ...prev,
        deck: [...prev.deck, ...prev.pool],
        pool: [],
      };
    });
  }, []);

  const moveAllToPool = useCallback(() => {
    setState((prev) => {
      const nonBane = prev.deck.filter((e) => !e.isBane);
      if (nonBane.length === 0) return prev;
      const banes = prev.deck.filter((e) => e.isBane);
      logEvent("all_cards_moved_to_pool", { count: nonBane.length });
      return {
        ...prev,
        pool: [...prev.pool, ...nonBane],
        deck: banes,
      };
    });
  }, []);

  const setDreamcaller = useCallback((dreamcaller: Dreamcaller) => {
    logEvent("dreamcaller_selected", {
      name: dreamcaller.name,
      tides: dreamcaller.tides,
      essenceBonus: dreamcaller.essenceBonus,
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

  const incrementCompletionLevel = useCallback((isMiniboss: boolean) => {
    setState((prev) => {
      const newLevel = prev.completionLevel + 1;
      logEvent("battle_won", {
        completionLevel: newLevel,
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
  }, []);

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

  const setStartingTide = useCallback((tide: NamedTide) => {
    logEvent("starting_tide_set", { tide });
    setState((prev) => ({ ...prev, startingTide: tide }));
  }, []);

  const setAnteState = useCallback((anteState: AnteState | null) => {
    logEvent("ante_state_changed", { anteState });
    setState((prev) => ({ ...prev, anteState }));
  }, []);

  const addProvisionedSite = useCallback(
    (dreamscapeId: string, site: SiteState) => {
      logEvent("site_provisioned", {
        dreamscapeId,
        siteId: site.id,
        siteType: site.type,
      });
      setState((prev) => {
        const node = prev.atlas.nodes[dreamscapeId];
        if (!node) return prev;
        const sites = [...node.sites];
        const lastSite = sites[sites.length - 1];
        const insertIdx = lastSite?.type === "Battle" ? sites.length - 1 : sites.length;
        sites.splice(insertIdx, 0, site);
        const updatedNode = { ...node, sites };
        return {
          ...prev,
          atlas: {
            ...prev.atlas,
            nodes: { ...prev.atlas.nodes, [dreamscapeId]: updatedNode },
          },
        };
      });
    },
    [],
  );

  const initializeDeckFromPool = useCallback(() => {
    setState((prev) => {
      logEvent("deck_initialized_from_pool", { count: prev.pool.length });
      return { ...prev, deck: [...prev.pool], pool: [] };
    });
  }, []);

  const removeFromPool = useCallback(
    (entryId: string, source: string) => {
      setState((prev) => {
        const entry = prev.pool.find((e) => e.entryId === entryId);
        if (!entry) return prev;
        const card = cardDatabase.get(entry.cardNumber);
        const cardName =
          card?.name ?? `Unknown Card #${String(entry.cardNumber)}`;
        logEvent("card_removed_from_pool", {
          cardNumber: entry.cardNumber,
          cardName,
          source,
        });
        return {
          ...prev,
          pool: prev.pool.filter((e) => e.entryId !== entryId),
          deck: prev.deck.filter((e) => e.entryId !== entryId),
        };
      });
    },
    [cardDatabase],
  );

  const resetQuest = useCallback(() => {
    resetLog();
    entryIdCounter.current = 0;
    setState(createDefaultState());
  }, []);

  /** Card numbers for each starter card and the number of copies in the fixed deck. */
  const STARTER_DECK_ALLOCATION: Array<[number, number]> = [
    [718, 4], // Glimpse of What Was (1-cost Event)
    [720, 3], // Worlds Await (1-cost Event)
    [711, 4], // Nocturne Strummer (2-cost Character)
    [717, 2], // Flashpoint Detonation (2-cost Event)
    [719, 2], // Sign of Arrival (2-cost Event)
    [715, 3], // Final Witness (3-cost Character)
    [712, 3], // Ringwatcher (3-cost Character)
    [713, 4], // Marked Direwolf (4-cost Character)
    [714, 3], // Runebound Champion (5-cost Character)
    [716, 2], // Wildflower Colossus (6-cost Character)
  ];

  const initializeQuest = useCallback(
    (db: Map<number, CardData>, config: QuestConfig) => {
      setState((prev) => {
        // Build fixed starter deck
        const deck: DeckEntry[] = [];
        for (const [cardNumber, copies] of STARTER_DECK_ALLOCATION) {
          for (let i = 0; i < copies; i++) {
            entryIdCounter.current += 1;
            deck.push({
              entryId: `deck-${String(entryIdCounter.current)}`,
              cardNumber,
              transfiguration: null,
              isBane: false,
            });
          }
        }

        logEvent("starting_deck_initialized", {
          totalDeckSize: deck.length,
          allocation: STARTER_DECK_ALLOCATION.map(([cn, copies]) => ({
            cardNumber: cn,
            name: db.get(cn)?.name ?? "Unknown",
            copies,
          })),
        });

        // Pick a random tide for the first dreamscape
        const dreamscapeTide = NAMED_TIDES[
          Math.floor(Math.random() * NAMED_TIDES.length)
        ];

        logEvent("dreamscape_tide_selected", { tide: dreamscapeTide });

        // Generate initial atlas (startingTide is null since player has no tide)
        const atlas = generateInitialAtlas(0, {
          cardDatabase: db,
          dreamsignPool: DREAMSIGNS,
          playerHasBanes: false,
          startingTide: null,
          playerPool: [],
          config,
          dreamscapeTide: dreamscapeTide as NamedTide,
        });

        const firstNodeId = atlas.edges[0]?.[1] ?? null;

        logEvent("quest_started", {
          initialEssence: config.startingEssence,
          dreamscapeTide,
          startingDeckSize: deck.length,
        });

        return {
          ...prev,
          essence: config.startingEssence,
          deck,
          pool: [],
          atlas,
          currentDreamscape: firstNodeId,
          visitedSites: [],
          screen: { type: "dreamscape" } as Screen,
          startingTide: null,
        };
      });
    },
    [],
  );

  const mutations = useMemo<QuestMutations>(
    () => ({
      changeEssence,
      addToPool,
      addBaneToPool,
      removeCard,
      removeFromPool,
      transfigureCard,
      moveToDeck,
      moveToPool,
      moveAllToDeck,
      moveAllToPool,
      setDreamcaller,
      addDreamsign,
      removeDreamsign,
      addTideCrystal,
      incrementCompletionLevel,
      setScreen,
      markSiteVisited,
      setCurrentDreamscape,
      updateAtlas,
      setStartingTide,
      setAnteState,
      addProvisionedSite,
      initializeDeckFromPool,
      resetQuest,
      initializeQuest,
    }),
    [
      changeEssence,
      addToPool,
      addBaneToPool,
      removeCard,
      removeFromPool,
      transfigureCard,
      moveToDeck,
      moveToPool,
      moveAllToDeck,
      moveAllToPool,
      setDreamcaller,
      addDreamsign,
      removeDreamsign,
      addTideCrystal,
      incrementCompletionLevel,
      setScreen,
      markSiteVisited,
      setCurrentDreamscape,
      updateAtlas,
      setStartingTide,
      setAnteState,
      addProvisionedSite,
      initializeDeckFromPool,
      resetQuest,
      initializeQuest,
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
