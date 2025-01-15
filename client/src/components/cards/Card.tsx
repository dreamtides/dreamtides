import { cn } from "@nextui-org/react";
import { CardView } from "../../bindings";
import { motion } from "motion/react";

const ASPECT_RATIO = 1.6;
const WIDTH = 24;
const HEIGHT = WIDTH * ASPECT_RATIO;

export type CardSize = {
  vw: number;
  vh: number;
};

export type CardProps = {
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
      className={cn(
        "flex rounded-xl border-1 border-white relative",
        backgroundColor,
        className
      )}
      style={{
        width: `${WIDTH}dvw`,
        height: `${HEIGHT}dvw`,
      }}
    >
      {card.revealed && <EnergyCost cost={card.revealed.cost} />}
    </motion.div>
  );
}

function EnergyCost({ cost }: { cost: number }) {
  return (
    <div
      className="absolute"
      style={{
        top: length(0.5),
        left: length(0.5),
        width: length(22),
        height: length(22),
        backgroundImage: "url('/assets/energy_cost_background.png')",
        backgroundSize: "cover",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <span
        style={{
          color: "white",
          fontFamily: "Impact",
          fontSize: "15px",
          lineHeight: 1,
        }}
      >
        {cost}
      </span>
    </div>
  );
}

function length(value: number) {
  return `${value * (WIDTH / 100)}dvw`;
}
