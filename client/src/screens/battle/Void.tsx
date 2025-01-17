import { CardView } from "../../bindings";
import { Card, CARD_ASPECT_RATIO } from "../../components/cards/Card";
import { DECK_CARD_WIDTH } from "./BattleDeck";

type DiscardPileProps = {
  cards: CardView[];
};

export default function Void({ cards }: DiscardPileProps) {
  return (
    <div
      className="relative my-1"
      style={{
        width: `${DECK_CARD_WIDTH * CARD_ASPECT_RATIO}px`,
        height: `${DECK_CARD_WIDTH}px`,
      }}
    >
      {cards.map((card, index) => (
        <Card
          card={card}
          width={DECK_CARD_WIDTH}
          className="absolute origin-top-left -rotate-90"
          style={{
            top: `${DECK_CARD_WIDTH - getCardOffset(index)}px`,
            left: `${getCardOffset(index)}px`,
          }}
        />
      ))}
    </div>
  );
}

function getCardOffset(index: number) {
  if (index < 1) {
    return 0;
  } else if (index < 2) {
    return 1;
  } else if (index < 3) {
    return 2;
  } else if (index < 4) {
    return 3;
  } else if (index < 5) {
    return 4;
  } else {
    return 5;
  }
}
