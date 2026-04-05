import {
  createContext,
  useCallback,
  useContext,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type {
  BattleView,
  CardView,
  GameAction,
  TestDeckName,
} from "../types/battle";
import * as api from "../api/client";
import { resetUserId } from "../api/client";
import { extractBattleView } from "../util/command-parser";

function getPosition(card: CardView): [string, string | null] {
  const pos = card.position.position;
  if (typeof pos === "string") return [pos, null];
  const key = Object.keys(pos)[0];
  const val = (pos as Record<string, unknown>)[key];
  return [key, typeof val === "string" ? val : null];
}

function generateEvents(oldBattle: BattleView, newBattle: BattleView): string[] {
  const events: string[] = [];
  const oldMap = new Map(oldBattle.cards.map((c) => [c.id, c]));

  for (const card of newBattle.cards) {
    const old = oldMap.get(card.id);
    if (!old) continue;
    const [oldPos] = getPosition(old);
    const [newPos, newPlayer] = getPosition(card);
    if (oldPos === newPos) continue;
    const name = card.revealed?.name ?? old.revealed?.name;
    if (!name) continue;

    if (newPos === "OnBattlefield" && newPlayer === "Enemy" && oldPos !== "OnBattlefield") {
      events.push(`Enemy materialized ${name}`);
    } else if (oldPos === "OnBattlefield" && newPos === "InVoid") {
      events.push(`${name} was dissolved`);
    } else if (oldPos === "OnBattlefield" && newPos === "InBanished") {
      events.push(`${name} was banished`);
    } else if (oldPos === "InHand" && newPlayer === "Enemy" && newPos === "OnStack") {
      events.push(`Enemy played ${name}`);
    }
  }

  if (newBattle.enemy.score !== oldBattle.enemy.score) {
    events.push(`Enemy score: ${oldBattle.enemy.score} \u2192 ${newBattle.enemy.score}`);
  }
  if (newBattle.user.score !== oldBattle.user.score) {
    events.push(`Your score: ${oldBattle.user.score} \u2192 ${newBattle.user.score}`);
  }

  return events;
}

interface BattleContextValue {
  battle: BattleView | null;
  isPolling: boolean;
  error: string | null;
  events: string[];
  sendAction: (action: GameAction) => void;
  sendDebugAction: (action: GameAction) => void;
  reconnect: (deck?: TestDeckName) => void;
}

const BattleContext = createContext<BattleContextValue | null>(null);

export function useBattle(): BattleContextValue {
  const ctx = useContext(BattleContext);
  if (!ctx) throw new Error("useBattle must be used within BattleProvider");
  return ctx;
}

const POLL_INTERVAL_MS = 200;

export function BattleProvider({ children }: { children: ReactNode }) {
  const [battle, setBattle] = useState<BattleView | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<string[]>([]);
  const responseVersionRef = useRef<string | undefined>(undefined);
  const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const prevBattleRef = useRef<BattleView | null>(null);
  // Generation counter to invalidate stale in-flight polls
  const pollGenerationRef = useRef(0);

  const stopPolling = useCallback(() => {
    pollGenerationRef.current++;
    if (pollIntervalRef.current != null) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    setIsPolling(false);
  }, []);

  const startPolling = useCallback(() => {
    // Clear any existing polling interval before starting a new one
    if (pollIntervalRef.current != null) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    pollGenerationRef.current++;
    const myGeneration = pollGenerationRef.current;
    setIsPolling(true);
    let pollInFlight = false;
    const interval = setInterval(() => {
      if (pollInFlight) return;
      if (pollGenerationRef.current !== myGeneration) {
        clearInterval(interval);
        return;
      }
      pollInFlight = true;
      void (async () => {
        try {
          const pollRes = await api.poll();
          // Check if this poll generation is still current
          if (pollGenerationRef.current !== myGeneration) return;
          if (pollRes.commands) {
            const view = extractBattleView(pollRes.commands);
            if (view) {
              if (prevBattleRef.current) {
                const newEvents = generateEvents(prevBattleRef.current, view);
                if (newEvents.length > 0) {
                  setEvents((prev) => [...prev, ...newEvents]);
                }
              }
              prevBattleRef.current = view;
              setBattle(view);
              if (view.user.can_act) {
                stopPolling();
              }
            }
          }
          if (pollRes.response_version) {
            responseVersionRef.current = pollRes.response_version;
          }
          if (pollRes.response_type === "Final") {
            stopPolling();
          }
        } catch (e) {
          stopPolling();
          setError(e instanceof Error ? e.message : "Poll failed");
        } finally {
          pollInFlight = false;
        }
      })();
    }, POLL_INTERVAL_MS);
    pollIntervalRef.current = interval;
  }, [stopPolling]);

  const sendAction = useCallback(
    (action: GameAction) => {
      if (isPolling) return;
      setEvents([]);
      void (async () => {
        try {
          setError(null);
          const res = await api.performAction(
            action,
            responseVersionRef.current,
          );
          const view = extractBattleView(res.commands);
          if (view) {
            prevBattleRef.current = view;
            setBattle(view);
            if (view.user.can_act) return;
          }
          startPolling();
        } catch (e) {
          setError(e instanceof Error ? e.message : "Action failed");
        }
      })();
    },
    [isPolling, startPolling],
  );

  const sendDebugAction = useCallback(
    (action: GameAction) => {
      // Stop any background polling and invalidate in-flight polls.
      stopPolling();
      void (async () => {
        try {
          setError(null);
          const res = await api.performAction(action, undefined);
          let view = extractBattleView(res.commands);
          if (view) {
            setBattle(view);
          }
          // Poll with retries to get the updated state.
          for (let attempt = 0; attempt < 8; attempt++) {
            await new Promise((r) => setTimeout(r, 150 + attempt * 100));
            const pollRes = await api.poll();
            if (pollRes.commands) {
              const pollView = extractBattleView(pollRes.commands);
              if (pollView) {
                setBattle(pollView);
                view = pollView;
              }
            }
            if (pollRes.response_version) {
              responseVersionRef.current = pollRes.response_version;
            }
            if (pollRes.response_type === "Final") {
              break;
            }
            // If the response was consumed by a stale in-flight poll,
            // re-send the action to generate a new response.
            if (pollRes.response_type === "None" && attempt === 0) {
              await api.performAction(action, undefined);
              continue;
            }
            if (pollRes.response_type === "None") {
              break;
            }
          }
          if (view && !view.user.can_act) {
            startPolling();
          }
        } catch (e) {
          setError(e instanceof Error ? e.message : "Debug action failed");
        }
      })();
    },
    [stopPolling, startPolling],
  );

  const reconnect = useCallback(
    (deck?: TestDeckName) => {
      void (async () => {
        try {
          setError(null);
          setEvents([]);
          stopPolling();
          setBattle(null);
          resetUserId();
          const res = await api.connect(deck);
          responseVersionRef.current = res.response_version;
          const view = extractBattleView(res.commands);
          if (view) {
            prevBattleRef.current = view;
            setBattle(view);
          }
        } catch (e) {
          setError(e instanceof Error ? e.message : "Connect failed");
        }
      })();
    },
    [stopPolling],
  );

  return (
    <BattleContext.Provider
      value={{ battle, isPolling, error, events, sendAction, sendDebugAction, reconnect }}
    >
      {children}
    </BattleContext.Provider>
  );
}
