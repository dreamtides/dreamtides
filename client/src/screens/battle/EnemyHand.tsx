import { ClientBattleId, commands } from "../../bindings";
import { Localized } from "@fluent/react";
import { useState } from "react";

type EnemyHandProps = {
  battleId: ClientBattleId;
};

export default function EnemyHand({}: EnemyHandProps) {
  const [sceneNumber, setSceneNumber] = useState(1);

  const handleFetch = () => {
    console.log("fetching");
    setSceneNumber((sceneNumber + 1) % 15);
    commands.handleAction("123", sceneNumber);
  };

  return (
    <div
      style={{
        display: "flex",
        backgroundColor: "rgb(202, 138, 4)",
        paddingLeft: "1rem",
        paddingRight: "1rem",
        alignItems: "center",
        justifyContent: "center",
        height: "10dvh",
      }}
    >
      <button onClick={handleFetch}>
        <Localized id="fetch">Fetch</Localized>
      </button>
    </div>
  );
}
