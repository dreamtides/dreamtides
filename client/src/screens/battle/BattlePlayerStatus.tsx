import { DisplayPlayer, CardView } from "../../bindings";
import BattleDeck from "./BattleDeck";
import Void from "./Void";

type BattlePlayerStatusProps = {
  owner: DisplayPlayer;
  deck: CardView[];
  void: CardView[];
};

export default function BattlePlayerStatus({
  owner,
  deck,
  void: discardPile,
}: BattlePlayerStatusProps) {
  return (
    <div
      key={owner}
      className="flex items-center justify-between px-2"
      style={{ height: "10dvh" }}
    >
      <BattleDeck cards={deck} />
      <Void cards={discardPile} />
    </div>
  );
}
