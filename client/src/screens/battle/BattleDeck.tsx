import { CardView } from "../../bindings";
import { Card, CARD_ASPECT_RATIO } from "../../components/cards/Card";

type BattleDeckProps = {
  cards: CardView[];
};

export const DECK_CARD_WIDTH = 55;

export default function BattleDeck({ cards }: BattleDeckProps) {
  return (
    <div
      style={{
        position: "relative",
        marginTop: "0.25rem",
        marginBottom: "0.25rem",
        width: `${DECK_CARD_WIDTH * CARD_ASPECT_RATIO}px`,
        height: `${DECK_CARD_WIDTH}px`,
        transform: `translateY(${DECK_CARD_WIDTH}px)`,
      }}
    >
      {cards.map((card, index) => (
        <Card
          key={JSON.stringify(card.id)}
          card={card}
          width={DECK_CARD_WIDTH}
          rotate={true}
          style={{
            left: `${getCardOffset(index)}px`,
            position: "absolute",
          }}
        />
      ))}
    </div>
  );
}

function getCardOffset(index: number) {
  if (index < 5) {
    return 0;
  } else if (index < 10) {
    return 1;
  } else if (index < 15) {
    return 2;
  } else if (index < 20) {
    return 3;
  } else if (index < 25) {
    return 4;
  } else {
    return 5;
  }
}
