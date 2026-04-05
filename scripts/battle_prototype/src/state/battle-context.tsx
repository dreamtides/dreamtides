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
  GameAction,
  TestDeckName,
} from "../types/battle";
import * as api from "../api/client";
import { extractBattleView } from "../util/command-parser";

interface BattleContextValue {
  battle: BattleView | null;
  isPolling: boolean;
  error: string | null;
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
  const responseVersionRef = useRef<string | undefined>(undefined);
  const pollIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
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
      void (async () => {
        try {
          setError(null);
          const res = await api.performAction(
            action,
            responseVersionRef.current,
          );
          const view = extractBattleView(res.commands);
          if (view) {
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
          stopPolling();
          const res = await api.connect(deck);
          responseVersionRef.current = res.response_version;
          const view = extractBattleView(res.commands);
          if (view) setBattle(view);
        } catch (e) {
          setError(e instanceof Error ? e.message : "Connect failed");
        }
      })();
    },
    [stopPolling],
  );

  return (
    <BattleContext.Provider
      value={{ battle, isPolling, error, sendAction, sendDebugAction, reconnect }}
    >
      {children}
    </BattleContext.Provider>
  );
}
