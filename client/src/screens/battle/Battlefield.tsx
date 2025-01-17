import { CardView, DisplayPlayer } from "../../bindings";
import { Card } from "../../components/cards/Card";

type BattlefieldProps = {
  owner: DisplayPlayer;
  cards: CardView[];
};

export default function Battlefield({ owner, cards }: BattlefieldProps) {
  if (cards.length === 0) {
    return (
      <div
        key={owner}
        className="flex bg-orange-600  items-center justify-center"
        style={{ height: "22dvh" }}
      />
    );
  }

  return (
    <>
      <div
        key={owner}
        className="flex bg-green-600  items-center justify-center"
        style={{ height: "11dvh" }}
      >
        {cards.slice(1).map((card) => (
          <Card
            key={JSON.stringify(card.id)}
            card={card}
            width={50}
            layout="battlefield"
            className="m-[1px]"
          />
        ))}
      </div>
      <div
        key={`${owner}-2`}
        className="flex bg-green-600  items-center justify-center"
        style={{ height: "11dvh" }}
      >
        <Card
          key={JSON.stringify(cards[0].id)}
          card={cards[0]}
          width={50}
          layout="battlefield"
          className="m-[1px]"
        />
      </div>
    </>
  );
}
