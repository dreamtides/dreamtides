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
        const marginSpace = 3 * 2; // 1px margin on each side between 4 cards = 3 gaps
        const newCardWidth = Math.floor((containerWidth - marginSpace) / 4);
        setCardWidth(newCardWidth);
      }
    };

    updateCardWidth();
    window.addEventListener("resize", updateCardWidth);
    return () => window.removeEventListener("resize", updateCardWidth);
  }, []);

  return (
    <div
      ref={containerRef}
      className="flex bg-blue-600 items-center justify-center"
      style={{ height: "26dvh" }}
    >
      {cards.map((card) => (
        <Card
          key={JSON.stringify(card.id)}
          card={card}
          width={cardWidth}
          className="m-[1px]"
        />
      ))}
    </div>
  );
}
