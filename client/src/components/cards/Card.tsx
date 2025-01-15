import { cn } from "@nextui-org/react";
import { CardView } from "../../bindings";
import { motion } from "motion/react";

const ASPECT_RATIO = 1.6;

export type CardSize = {
  vw: number;
  vh: number;
};

export type CardProps = {
  card: CardView;
  size: CardSize;
  className?: string;
  onBattlefield?: boolean;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export function Card({ card, size, className, onBattlefield }: CardProps) {
  const id = JSON.stringify(card.id);

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
      style={getSizeStyle(size)}
    >
      <p className="text-lg font-bold">{id.substring(11, 13)}</p>
    </motion.div>
  );
}

/**
 * Returns a size style which maintains ASPECT_RATIO while not exceeding the
 * given percentages of the viewport height and the viewport width.
 * @param size
 */
function getSizeStyle(size: CardSize) {
  const vwPixels = (window.innerWidth * size.vw) / 100;
  const vhPixels = (window.innerHeight * size.vh) / 100;

  const heightFromWidth = vwPixels * ASPECT_RATIO;
  const widthFromHeight = vhPixels / ASPECT_RATIO;

  const width = Math.min(vwPixels, widthFromHeight);
  const height = Math.min(vhPixels, heightFromWidth);

  return {
    width: `${width}px`,
    height: `${height}px`,
  };
}
