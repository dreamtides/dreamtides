import { cn } from "@nextui-org/react";
import { CardView } from "../../bindings";

const ASPECT_RATIO = 16 / 9;
const WIDTH = 24;

type CardProps = {
  card: CardView;
  className?: string;
  onBattlefield?: boolean;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export function Card({ card, className, onBattlefield }: CardProps) {
  const id = JSON.stringify(card.id);
  const width = onBattlefield ? WIDTH * 0.7 : WIDTH;
  return (
    <div
      key={id}
      className={cn("flex bg-purple-600 rounded-xl", className)}
      style={{ height: `${width * ASPECT_RATIO}dvw`, width: `${width}dvw` }}
    >
      {id.substring(11, 13)}
    </div>
  );
}
