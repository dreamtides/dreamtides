import { CardView } from "../../bindings";
import { Card } from "../../components/cards/Card";
import { useRef, useLayoutEffect, useState } from "react";

type UserHandProps = {
  cards: CardView[];
};

export default function UserHand({ cards }: UserHandProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [cardWidth, setCardWidth] = useState(100);

  useLayoutEffect(() => {
    const updateCardWidth = () => {
      if (containerRef.current) {
        const containerWidth = containerRef.current.offsetWidth;
        const marginSpace = 2 * 2;
        const newCardWidth = Math.floor((containerWidth - marginSpace) / 3);
        setCardWidth(newCardWidth);
      }
    };

    updateCardWidth();
    window.addEventListener("resize", updateCardWidth);
    return () => window.removeEventListener("resize", updateCardWidth);
  }, []);

  const renderCards = () => {
    if (cards.length < 5) {
      return cards.map((card) => (
        <Card
          key={JSON.stringify(card.id)}
          card={card}
          width={90}
          style={{ margin: "1px" }}
        />
      ));
    }

    const midIndex = Math.floor(cards.length / 2);
    const visibleCards = cards.slice(midIndex - 1, midIndex + 2);
    const leftStack = cards.slice(0, midIndex - 1);
    const rightStack = cards.slice(midIndex + 2);

    return (
      <>
        {leftStack.length > 0 && (
          <div style={{ position: "absolute", left: 0, bottom: 0, width: 100 }}>
            {leftStack.map((card, index) => (
              <Card
                key={JSON.stringify(card.id)}
                card={card}
                width={90}
                style={{ position: "absolute", left: 0, bottom: 0 }}
              />
            ))}
          </div>
        )}
        {visibleCards.map((card) => (
          <Card
            key={JSON.stringify(card.id)}
            card={card}
            width={90}
            style={{ margin: "1px", zIndex: 10 }}
          />
        ))}
        {rightStack.length > 0 && (
          <div
            style={{ position: "absolute", right: 0, bottom: 0, width: 100 }}
          >
            {rightStack.map((card, index) => (
              <Card
                key={JSON.stringify(card.id)}
                card={card}
                width={90}
                style={{ position: "absolute", right: 0, bottom: 0 }}
              />
            ))}
          </div>
        )}
      </>
    );
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
      {renderCards()}
    </div>
  );
}
