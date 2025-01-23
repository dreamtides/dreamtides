import { CardView } from "../../bindings";
import { Card } from "../../components/cards/Card";
import { useRef, useLayoutEffect, useState } from "react";

type UserHandProps = {
  cards: CardView[];
};

const CARD_MARGIN = 1;
const MAX_CARDS_SIDE_BY_SIDE = 4;
const SCROLL_THRESHOLD = 6;

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
    const totalWidth = containerRef.current?.offsetWidth ?? 0;
    const availableWidth = totalWidth - cardWidth;

    if (cards.length > SCROLL_THRESHOLD) {
      // Use the same spacing as with 6 cards
      const spacingWith6Cards = availableWidth / (SCROLL_THRESHOLD - 1);
      return index * spacingWith6Cards;
    }

    const totalCardsWidth =
      Math.min(cards.length, MAX_CARDS_SIDE_BY_SIDE) * cardWidth;
    const startX = (totalWidth - totalCardsWidth) / 2;
    if (cards.length <= MAX_CARDS_SIDE_BY_SIDE) {
      return startX + index * cardWidth;
    }
    return (availableWidth / (cards.length - 1)) * index;
  };

  const getVerticalOffset = (index: number) => {
    const middleIndex = (cards.length - 1) / 2;
    const distanceFromMiddle = Math.abs(index - middleIndex);
    const maxDistance = Math.max(middleIndex, cards.length - 1 - middleIndex);
    return -5 * (1 - distanceFromMiddle / maxDistance);
  };

  const getRotation = (index: number) => {
    return -5 + (10 * index) / (cards.length - 1);
  };

  const containerStyle: React.CSSProperties = {
    position: "relative",
    height: "26dvh",
    backgroundColor: "rgb(37, 99, 235)",
    ...(cards.length > SCROLL_THRESHOLD
      ? {
          overflowX: "auto",
          overflowY: "hidden",
          display: "block",
        }
      : {
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }),
  };

  const getScrollableWidth = () => {
    const totalWidth = containerRef.current?.offsetWidth ?? 0;
    const availableWidth = totalWidth - cardWidth;
    const spacingWith6Cards = availableWidth / (SCROLL_THRESHOLD - 1);
    return cardWidth + spacingWith6Cards * (cards.length - 1);
  };

  const innerContainerStyle: React.CSSProperties =
    cards.length > SCROLL_THRESHOLD
      ? {
          position: "relative",
          height: "100%",
          width: `${getScrollableWidth()}px`,
        }
      : {};

  return (
    <div ref={containerRef} style={containerStyle}>
      <div style={innerContainerStyle}>
        {cards.map((card, index) => (
          <Card
            key={JSON.stringify(card.id)}
            card={card}
            width={cardWidth}
            rotate={getRotation(index) * 0.5}
            style={{
              margin: `${CARD_MARGIN}px`,
              position: "absolute",
              left: getCardOffset(index),
              top: getVerticalOffset(index) + 10,
            }}
          />
        ))}
      </div>
    </div>
  );
}
