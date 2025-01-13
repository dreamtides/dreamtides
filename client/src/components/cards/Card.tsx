import { cn } from "@nextui-org/react";
import { CardView } from "../../bindings";

const ASPECT_RATIO = 16 / 9;
const HEIGHT = 25;
const WIDTH = HEIGHT / ASPECT_RATIO;

type CardProps = {
  card: CardView;
  className?: string;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export function Card({ card, className }: CardProps) {
  const id = JSON.stringify(card.id);
  return (
    <div
      key={id}
      className={cn("flex bg-purple-600 rounded-xl", className)}
      style={{ height: `${HEIGHT}dvh`, width: `${WIDTH}dvh` }}
    />
  );
}
