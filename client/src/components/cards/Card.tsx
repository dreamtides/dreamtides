import { CardView } from "../../bindings";

const ASPECT_RATIO = 0.65;
const HEIGHT = 25;
const WIDTH = HEIGHT * ASPECT_RATIO;

type CardProps = {
  card: CardView;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export function Card({ card }: CardProps) {
  const id = JSON.stringify(card.id);
  return (
    <div
      key={id}
      className="flex bg-purple-600"
      style={{ height: `${HEIGHT}dvh`, width: `${WIDTH}dvh` }}
    >
      {id}
    </div>
  );
}
