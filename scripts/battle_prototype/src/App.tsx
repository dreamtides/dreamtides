import { useEffect } from "react";
import { BattleProvider, useBattle } from "./state/battle-context.tsx";
import { BattleScreen } from "./components/BattleScreen.tsx";

function BattleApp() {
  const { battle, isPolling, error, sendAction, reconnect } = useBattle();

  useEffect(() => {
    reconnect("Core11");
  }, [reconnect]);

  if (error) {
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
    <BattleScreen
      battle={battle}
      onAction={sendAction}
      disabled={isPolling}
    />
  );
}

export default function App() {
  return (
    <BattleProvider>
      <BattleApp />
    </BattleProvider>
  );
}
