import { Button } from "@nextui-org/react";
import { ClientBattleId, commands } from "../../bindings";
import { Localized } from "@fluent/react";

type EnemyHandProps = {
  battleId: ClientBattleId;
};

export default function EnemyHand({ battleId }: EnemyHandProps) {

  const handleFetch = () => {
    console.log("fetching");
    commands.handleAction("123", 1);
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
