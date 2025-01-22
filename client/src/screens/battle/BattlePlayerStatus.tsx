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
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        paddingLeft: "0.5rem",
        paddingRight: "0.5rem",
        height: "10dvh"
      }}
    >
      <BattleDeck cards={deck} />
      <Void cards={discardPile} />
    </div>
  );
}
