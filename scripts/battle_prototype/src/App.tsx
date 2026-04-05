import { useEffect } from "react";
import { BattleProvider, useBattle } from "./state/battle-context.tsx";

function BattleApp() {
  const { battle, isPolling, error, reconnect } = useBattle();

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
    <div className="p-4">
      <h1 className="text-lg font-bold mb-2">
        Battle Prototype {isPolling ? "(polling...)" : ""}
      </h1>
      <div className="grid grid-cols-2 gap-4 mb-4">
        <div>
          <h2 className="font-bold">You</h2>
          <p>Score: {battle.user.score} | Energy: {battle.user.energy}/{battle.user.produced_energy} | Spark: {battle.user.total_spark}</p>
        </div>
        <div>
          <h2 className="font-bold">Enemy</h2>
          <p>Score: {battle.enemy.score} | Energy: {battle.enemy.energy}/{battle.enemy.produced_energy} | Spark: {battle.enemy.total_spark}</p>
        </div>
      </div>
      <p>Turn: {battle.turn_number} | Cards: {battle.cards.length}</p>
      <pre className="mt-4 text-xs overflow-auto max-h-96 p-2 rounded"
           style={{ background: "var(--color-surface)" }}>
        {JSON.stringify(battle, null, 2)}
      </pre>
    </div>
  );
}

export default function App() {
  return (
    <BattleProvider>
      <BattleApp />
    </BattleProvider>
  );
}
