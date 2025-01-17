import { CardView } from "../../bindings";
import { Card } from "../../components/cards/Card";

type BattleDeckProps = {
  cards: CardView[];
};

const CARD_WIDTH = 60;

export default function BattleDeck({ cards }: BattleDeckProps) {
  return (
    <div
      className="relative bg-slate-600 my-1"
      style={{ width: "16dvh", height: "10dvh" }}
    >
      {cards.map((card) => (
        <Card
          card={card}
          width={CARD_WIDTH}
          className="absolute origin-top-left -rotate-90"
          style={{
            top: `${CARD_WIDTH}px`,
          }}
        />
      ))}
    </div>
  );
}
