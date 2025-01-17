import { Button } from "@nextui-org/react";
import { useSWRConfig } from "swr";
import { ClientBattleId } from "../../bindings";
import { Localized } from "@fluent/react";

type EnemyHandProps = {
  battleId: ClientBattleId;
  onSceneChange: () => void;
};

export default function EnemyHand({ battleId, onSceneChange }: EnemyHandProps) {
  const { mutate } = useSWRConfig();

  const handleFetch = () => {
    onSceneChange();
    mutate(battleId);
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
