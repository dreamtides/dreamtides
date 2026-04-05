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

  const startPolling = useCallback(() => {
    setIsPolling(true);
    let pollInFlight = false;
    const interval = setInterval(() => {
      if (pollInFlight) return;
      pollInFlight = true;
      void (async () => {
        try {
          const pollRes = await api.poll();
          if (pollRes.commands) {
            const view = extractBattleView(pollRes.commands);
            if (view) {
              setBattle(view);
              if (view.user.can_act) {
                clearInterval(interval);
                setIsPolling(false);
              }
            }
          }
          if (pollRes.response_version) {
            responseVersionRef.current = pollRes.response_version;
          }
          if (pollRes.response_type === "Final") {
            clearInterval(interval);
            setIsPolling(false);
          }
        } catch (e) {
          clearInterval(interval);
          setIsPolling(false);
          setError(e instanceof Error ? e.message : "Poll failed");
        } finally {
          pollInFlight = false;
        }
      })();
    }, POLL_INTERVAL_MS);
  }, []);

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
      void (async () => {
        try {
          setError(null);
          const res = await api.performAction(
            action,
            responseVersionRef.current,
          );
          const view = extractBattleView(res.commands);
          if (view) setBattle(view);
          startPolling();
        } catch (e) {
          setError(e instanceof Error ? e.message : "Debug action failed");
        }
      })();
    },
    [startPolling],
  );

  const reconnect = useCallback(
    (deck?: TestDeckName) => {
      void (async () => {
        try {
          setError(null);
          setIsPolling(false);
          const res = await api.connect(deck);
          responseVersionRef.current = res.response_version;
          const view = extractBattleView(res.commands);
          if (view) setBattle(view);
        } catch (e) {
          setError(e instanceof Error ? e.message : "Connect failed");
        }
      })();
    },
    [],
  );

  return (
    <BattleContext.Provider
      value={{ battle, isPolling, error, sendAction, sendDebugAction, reconnect }}
    >
      {children}
    </BattleContext.Provider>
  );
}
