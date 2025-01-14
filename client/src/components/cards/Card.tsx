import { cn } from "@nextui-org/react";
import { CardView } from "../../bindings";
import { motion } from "motion/react";

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
  const width = WIDTH;

  let backgroundColor = "bg-purple-600";
  if (id.includes("#0")) {
    backgroundColor = "bg-green-600";
  } else if (id.includes("#1")) {
    backgroundColor = "bg-yellow-600";
  } else if (id.includes("#2")) {
    backgroundColor = "bg-blue-600";
  } else if (id.includes("#3")) {
    backgroundColor = "bg-red-600";
  } else if (id.includes("#4")) {
    backgroundColor = "bg-pink-600";
  } else if (id.includes("#5")) {
    backgroundColor = "bg-orange-600";
  }

  return (
    <motion.div
      key={id}
      layoutId={id}
      initial={{ scale: 1 }}
      animate={{ scale: onBattlefield ? 0.7 : 1 }}
      className={cn(
        "flex rounded-xl border-1 border-white",
        backgroundColor,
        className,
      )}
      style={{ height: `${width * ASPECT_RATIO}dvw`, width: `${width}dvw` }}
    >
      <p className="text-lg font-bold">{id.substring(11, 13)}</p>
    </motion.div>
  );
}
