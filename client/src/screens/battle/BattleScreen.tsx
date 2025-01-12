import NavigationBar from "../../components/common/NavigationBar";
import Battlefield from "./Battlefield";
import BattlePlayerStatus from "./BattlePlayerStatus";
import EnemyHand from "./EnemyHand";
import UserHand from "./UserHand";

type BattleScreenProps = {};

export default function BattleScreen({}: BattleScreenProps) {
  return (
    <div className="flex flex-col h-screen w-screen">
      <NavigationBar>
        <EnemyHand />
      </NavigationBar>
      <BattlePlayerStatus />
      <Battlefield />
      <BattlePlayerStatus />
      <UserHand position="default" />
    </div>
  );
}
