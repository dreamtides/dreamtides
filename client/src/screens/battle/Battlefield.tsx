import { CardView, DisplayPlayer } from "../../bindings";
import { Card } from "../../components/cards/Card";

type BattlefieldProps = {
  owner: DisplayPlayer;
  cards: CardView[];
};

export default function Battlefield({ owner, cards }: BattlefieldProps) {
  return (
    <div
      key={owner}
      className="flex bg-green-600  items-center justify-center"
      style={{ height: "20dvh" }}
    >
      {cards.map((card) => (
        <Card
          key={JSON.stringify(card.id)}
          card={card}
          width={50}
          layout="battlefield"
          className="m-[1px]"
        />
      ))}
    </div>
  );
}
