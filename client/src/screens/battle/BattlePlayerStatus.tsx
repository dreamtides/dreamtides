import { DisplayPlayer, CardView } from "../../bindings";
import BattleDeck from "./BattleDeck";

type BattlePlayerStatusProps = {
  owner: DisplayPlayer;
  deck: CardView[];
};

export default function BattlePlayerStatus({
  owner,
  deck,
}: BattlePlayerStatusProps) {
  return (
    <div
      key={owner}
      className="flex bg-red-600 items-center px-4"
      style={{ height: "10dvh" }}
    >
      <BattleDeck cards={deck} />
    </div>
  );
}
