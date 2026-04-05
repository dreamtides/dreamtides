import { useEffect } from "react";
import { BattleProvider, useBattle } from "./state/battle-context.tsx";
import { BattleScreen } from "./components/BattleScreen.tsx";

function BattleApp() {
  const { battle, isPolling, error, sendAction, sendDebugAction, reconnect, events, yourTurnCounter } = useBattle();

  useEffect(() => {
    reconnect("Core11");
  }, [reconnect]);

  if (!battle && error) {
    return (
      <div className="p-4">
        <p className="text-red-400">Error: {error}</p>
        <button
          className="mt-2 px-4 py-2 rounded"
          style={{ background: "var(--color-primary)" }}
          onClick={() => reconnect("Core11")}
        >
          Retry
        </button>
      </div>
    );
  }

  if (!battle) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <p style={{ color: "var(--color-text-dim)" }}>Connecting...</p>
      </div>
    );
  }

  return (
    <>
      {error && (
        <div
          className="px-4 py-2 text-sm flex items-center justify-between"
          style={{ background: "#7f1d1d", color: "#fecaca" }}
        >
          <span>Error: {error}</span>
          <button
            className="px-3 py-1 rounded text-xs"
            style={{ background: "var(--color-primary)", color: "var(--color-text)" }}
            onClick={() => reconnect("Core11")}
          >
            Reconnect
          </button>
        </div>
      )}
      <BattleScreen
        battle={battle}
        onAction={sendAction}
        onDebugAction={sendDebugAction}
        onReconnect={() => reconnect("Core11")}
        events={events}
        disabled={isPolling}
        yourTurnCounter={yourTurnCounter}
      />
    </>
  );
}

export default function App() {
  return (
    <BattleProvider>
      <BattleApp />
    </BattleProvider>
  );
}
