import { cn } from "@nextui-org/react";
import { CardView, DisplayImage } from "../../bindings";
import { motion } from "motion/react";

const ASPECT_RATIO = 1.6;
const WIDTH = 200;
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
        "flex relative m-2",
        className
      )}
      style={{
        width: `${WIDTH}px`,
        height: `${HEIGHT}px`,
      }}
    >
      {card.revealed && <CardImage image={card.revealed.image} />}
      {card.revealed && <RulesText text={card.revealed.rulesText} />}
      {card.revealed && <EnergyCost cost={card.revealed.cost} />}
      {card.revealed && <CardName name={card.revealed.name} />}
      <FrameDecoration side="left" />
      <FrameDecoration side="right" />
    </motion.div>
  );
}

function EnergyCost({ cost }: { cost: number }) {
  return (
    <div
      className="absolute"
      style={{
        width: "45px",
        height: "45px",
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
          fontSize: "35px",
          lineHeight: 1,
        }}
      >
        {cost}
      </span>
    </div>
  );
}

function FrameDecoration({ side }: { side: "left" | "right" }) {
  return (
    <div
      className="absolute"
      style={{
        bottom: 0,
        [side]: 0,
        width: "100%",
        height: "95px",
        backgroundRepeat: "no-repeat",
        backgroundImage: `url('/assets/card_frame_decoration_${side}.png')`,
        backgroundSize: "contain",
        transform: side === "right" ? "scaleX(-1)" : undefined,
      }}
    />
  );
}

function CardName({ name }: { name: string }) {
  return (
    <div
      className="absolute w-full flex items-center"
      style={{
        bottom: "60px",
        backgroundImage: "url('/assets/card_name_background.png')",
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        height: "35px",
      }}
    >
      <span
        className="font-medium"
        style={{
          color: "white",
          fontFamily: "Garamond",
          paddingLeft: "10px",
          display: "block",
          fontSize: "13px",
        }}
      >
        {name}
      </span>
    </div>
  );
}

function RulesText({ text }: { text: string }) {
  return (
    <div
      className="absolute flex items-center"
      style={{
        bottom: "0.25dvw",
        backgroundImage: "url('/assets/rules_text_background.png')",
        backgroundSize: "cover",
        backgroundRepeat: "no-repeat",
        height: "70px",
        left: "10px",
        right: "10px",
      }}
    >
      <span
        style={{
          color: "black",
          fontFamily: "'Libre Baskerville', Georgia, 'Times New Roman', serif",
          paddingLeft: "25px",
          paddingRight: "15px",
          fontSize: "10px",
          lineHeight: "1.0",
        }}
      >
        {text}
      </span>
    </div>
  );
}

function CardImage({ image }: { image: DisplayImage }) {
  return (
    <div
      className="absolute top-0 w-full rounded-xl overflow-hidden"
      style={{
        height: "260px",
        width: "100%",
        backgroundImage: `url("${image.image}")`,
        backgroundSize: "cover",
        backgroundRepeat: "no-repeat",
        backgroundPosition: `${image.imageOffsetX ?? 50}% ${image.imageOffsetY ?? 50}%`,
      }}
    />
  );
}
