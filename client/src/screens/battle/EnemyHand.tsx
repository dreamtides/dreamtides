import { Button } from "@nextui-org/react";
import { useSWRConfig } from "swr";
import { ClientBattleId } from "../../bindings";
import { Localized } from "@fluent/react";

type EnemyHandProps = {
  battleId: ClientBattleId;
};

export default function EnemyHand({ battleId }: EnemyHandProps) {
  const { mutate } = useSWRConfig();
  return (
    <div
      className="flex bg-yellow-600 px-4 items-center justify-center"
      style={{ height: "10dvh" }}
    >
      <Button onPress={() => mutate(battleId)}>
        <Localized id="fetch">Fetch</Localized>
      </Button>
    </div>
  );
}
