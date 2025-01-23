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
        style={{
          display: "flex",
          backgroundColor: "rgb(234, 88, 12)",
          alignItems: "center",
          justifyContent: "center",
          height: "22dvh",
        }}
      />
    );
  }

  return (
    <>
      <div
        key={owner}
        style={{
          display: "flex",
          backgroundColor: "rgb(22, 163, 74)",
          alignItems: "center",
          justifyContent: "center",
          height: "11dvh",
        }}
      >
        {cards.slice(1).map((card) => (
          <Card
            key={JSON.stringify(card.id)}
            card={card}
            width={50}
            layout="battlefield"
            style={{ margin: "1px" }}
          />
        ))}
      </div>
      <div
        key={`${owner}-2`}
        style={{
          display: "flex",
          backgroundColor: "rgb(22, 163, 74)",
          alignItems: "center",
          justifyContent: "center",
          height: "11dvh",
        }}
      >
        <Card
          key={JSON.stringify(cards[0].id)}
          card={cards[0]}
          width={50}
          layout="battlefield"
          style={{ margin: "1px" }}
        />
      </div>
    </>
  );
}
