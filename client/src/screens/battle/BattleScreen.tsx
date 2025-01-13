import { BattleView, ClientBattleId, commands } from "../../bindings";
import { ErrorState } from "../../components/common/ErrorState";
import { Loading } from "../../components/common/Loading";
import NavigationBar from "../../components/common/NavigationBar";
import Battlefield from "./Battlefield";
import BattlePlayerStatus from "./BattlePlayerStatus";
// import EnemyHand from "./EnemyHand";
import UserHand from "./UserHand";
import useSWR from "swr";

type BattleFetchResult =
  | { battle: BattleView }
  | { error: Error }
  | { isLoading: boolean };

export function useBattle(id: ClientBattleId): BattleFetchResult {
  const { data, error, isLoading } = useSWR(id, commands.fetchBattle);

  if (isLoading) {
    return { isLoading: true };
  } else if (error || data == null) {
    return { error };
  } else {
    return { battle: data };
  }
}

type BattleScreenProps = {};

export default function BattleScreen({}: BattleScreenProps) {
  const result = useBattle("123");
  if ("isLoading" in result) {
    return <Loading />;
  } else if ("error" in result) {
    return <ErrorState />;
  }

  return (
    <div className="flex flex-col h-screen w-screen">
      <NavigationBar>
        {result.battle.id}
        {/* <EnemyHand /> */}
      </NavigationBar>
      <BattlePlayerStatus />
      <Battlefield />
      <BattlePlayerStatus />
      <UserHand position="default" />
    </div>
  );
}
