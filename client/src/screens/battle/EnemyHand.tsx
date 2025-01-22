import { Button } from "@nextui-org/react";
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
    setSceneNumber((sceneNumber + 1) % 3);
    commands.handleAction("123", sceneNumber);
  };

  return (
    <div
      className="flex bg-yellow-600 px-4 items-center justify-center"
      style={{ height: "10dvh" }}
    >
      <Button onPress={handleFetch}>
        <Localized id="fetch">Fetch</Localized>
      </Button>
    </div>
  );
}
