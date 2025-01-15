import { cn } from "@nextui-org/react";
import { CardView, DisplayImage } from "../../bindings";
import { motion } from "motion/react";

const BASE_WIDTH = 200;
const ASPECT_RATIO = 1.6;
const BASE_HEIGHT = BASE_WIDTH * ASPECT_RATIO;

export type CardSize = {
  vw: number;
  vh: number;
};

export type CardProps = {
  card: CardView;
  className?: string;
  width?: number;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export function Card({ card, className, width = BASE_WIDTH }: CardProps) {
  const id = JSON.stringify(card.id);
  const scale = width / BASE_WIDTH;
  const height = width * ASPECT_RATIO;

  return (
    <div
      className={cn("flex relative m-2", className)}
      style={{
        width: `${width}px`,
        height: `${height}px`,
      }}
    >
      <motion.div
        key={id}
        layoutId={id}
        className="origin-top-left"
        style={{
          width: `${BASE_WIDTH}px`,
          height: `${BASE_HEIGHT}px`,
          transform: `scale(${scale})`,
        }}
      >
        {card.revealed && <CardImage image={card.revealed.image} />}
        {card.revealed && <RulesText text={card.revealed.rulesText} />}
        {card.revealed && <EnergyCost cost={card.revealed.cost} />}
        <FrameDecoration side="left" />
        <FrameDecoration side="right" />
        {card.revealed && card.revealed.spark && (
          <SparkValue spark={card.revealed.spark} />
        )}
        {card.revealed && (
          <CardName name={card.revealed.name} cardType={card.revealed.cardType} />
        )}
      </motion.div>
    </div>
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
          fontFamily: "Anton, serif",
          fontSize: "35px",
          lineHeight: 1,
          WebkitTextStroke: "0.1em black",
          paintOrder: "stroke",
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

function CardName({ name, cardType }: { name: string; cardType: string }) {
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
      <div className="flex justify-between w-full items-center">
        <span
          className="font-medium"
          style={{
            color: "white",
            fontFamily: "'EB Garamond', serif",
            paddingLeft: "10px",
            fontSize: "13px",
          }}
        >
          {name}
        </span>
        <span
          style={{
            color: "white",
            fontFamily: "'EB Garamond', serif",
            paddingRight: "10px",
            fontSize: "10px",
          }}
        >
          {cardType}
        </span>
      </div>
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

function SparkValue({ spark }: { spark: number }) {
  return (
    <div
      className="absolute"
      style={{
        width: "30px",
        height: "30px",
        backgroundImage: "url('/assets/spark_background.png')",
        backgroundSize: "cover",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        bottom: 0,
        right: 0,
      }}
    >
      <span
        style={{
          color: "white",
          fontFamily: "Anton, serif",
          fontSize: "20px",
          lineHeight: 1,
          WebkitTextStroke: "0.1em black",
          paintOrder: "stroke",
        }}
      >
        {spark}
      </span>
    </div>
  );
}
