import { CardView } from "../../bindings";
import { Card } from "../../components/cards/Card";
import { useRef, useLayoutEffect, useState } from "react";

type UserHandProps = {
  cards: CardView[];
};

const CARD_MARGIN = 1;
const MAX_CARDS_SIDE_BY_SIDE = 4;

export default function UserHand({ cards }: UserHandProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [cardWidth, setCardWidth] = useState(100);

  useLayoutEffect(() => {
    const updateCardWidth = () => {
      if (containerRef.current) {
        const containerWidth = containerRef.current.offsetWidth;
        const marginSpace = 2 * 2;
        const newCardWidth = Math.floor((containerWidth - marginSpace) / 4);
        setCardWidth(newCardWidth);
      }
    };

    updateCardWidth();
    window.addEventListener("resize", updateCardWidth);
    return () => window.removeEventListener("resize", updateCardWidth);
  }, []);

  const getCardOffset = (index: number) => {
    if (cards.length <= MAX_CARDS_SIDE_BY_SIDE) return 0;
    const totalWidth = containerRef.current?.offsetWidth ?? 0;
    const availableWidth = totalWidth - cardWidth;
    const offset = (availableWidth / (cards.length - 1)) * index;
    return offset;
  };

  const getVerticalOffset = (index: number) => {
    const middleIndex = (cards.length - 1) / 2;
    const distanceFromMiddle = Math.abs(index - middleIndex);
    const maxDistance = Math.max(middleIndex, cards.length - 1 - middleIndex);
    return -10 * (1 - distanceFromMiddle / maxDistance);
  };

  return (
    <div
      ref={containerRef}
      style={{
        display: "flex",
        backgroundColor: "rgb(37, 99, 235)",
        alignItems: "center",
        justifyContent: "center",
        position: "relative",
        height: "26dvh",
      }}
    >
      {cards.map((card, index) => (
        <Card
          key={JSON.stringify(card.id)}
          card={card}
          width={cardWidth}
          style={{
            margin: `${CARD_MARGIN}px`,
            position:
              cards.length > MAX_CARDS_SIDE_BY_SIDE ? "absolute" : "relative",
            left:
              cards.length > MAX_CARDS_SIDE_BY_SIDE
                ? getCardOffset(index)
                : undefined,
            top: getVerticalOffset(index),
          }}
        />
      ))}
    </div>
  );
}
