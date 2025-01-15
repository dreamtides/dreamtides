import { CardView } from "../../bindings";
import { Card } from "../../components/cards/Card";

type UserHandProps = {
  cards: CardView[];
};

export default function UserHand({ cards }: UserHandProps) {
  return (
    <div
      className="flex bg-blue-600 items-center justify-center"
      style={{ height: "30dvh" }}
    >
      {cards.map((card) => (
        <Card card={card} size={{ vw: 24, vh: 29 }} className="m-[1px]" />
      ))}
    </div>
  );
}
