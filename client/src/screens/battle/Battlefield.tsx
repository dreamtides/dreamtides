import { CardView, DisplayPlayer } from "../../bindings";
import { Card } from "../../components/cards/Card";

type BattlefieldProps = {
  owner: DisplayPlayer;
  cards: CardView[];
};

export default function Battlefield({ cards }: BattlefieldProps) {
  return (
    <div
      className="flex bg-green-600  items-center justify-center"
      style={{ height: "20dvh" }}
    >
      {cards.map((card) => (
        <Card
          card={card}
          size={{ vw: 24, vh: 19 }}
          className="m-[1px]"
          onBattlefield={true}
        />
      ))}
    </div>
  );
}
