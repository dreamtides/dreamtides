import { Button } from "@nextui-org/react";
import { useSWRConfig } from "swr";
import { ClientBattleId } from "../../bindings";

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
      <Button onClick={() => mutate(battleId)}>Fetch Battle</Button>
    </div>
  );
}
