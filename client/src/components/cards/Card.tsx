import { cn } from "@nextui-org/react";
import {
  CardView,
  DisplayImage,
  RevealedCardView,
  CardFrame,
} from "../../bindings";
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
        {card.revealed ? <RevealedCard card={card.revealed} /> : <HiddenCard />}
      </motion.div>
    </div>
  );
}

function RevealedCard({ card }: { card: RevealedCardView }) {
  return (
    <>
      <CardImage image={card.image} />
      <RulesText text={card.rulesText} />
      <EnergyCost cost={card.cost} />
      <FrameDecoration side="left" frame={card.frame} />
      <FrameDecoration side="right" frame={card.frame} />
      {card.spark && <SparkValue spark={card.spark} />}
      <CardName name={card.name} cardType={card.cardType} frame={card.frame} />
    </>
  );
}

function HiddenCard() {
  return (
    <div
      className="absolute w-full h-full rounded-xl"
      style={{
        backgroundImage: "url('/assets/card_back.png')",
        backgroundSize: "cover",
      }}
    />
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

function FrameDecoration({
  side,
  frame,
}: {
  side: "left" | "right";
  frame: CardFrame;
}) {
  return (
    <div
      className="absolute"
      style={{
        bottom: 0,
        [side]: 0,
        width: "100%",
        height: "95px",
        backgroundRepeat: "no-repeat",
        backgroundImage: `url('${getFrameAssetUrl(frame, side === "left" ? "frame_left" : "frame_right")}')`,
        backgroundSize: "contain",
        transform: side === "right" ? "scaleX(-1)" : undefined,
      }}
    />
  );
}

function CardName({
  name,
  cardType,
  frame,
}: {
  name: string;
  cardType: string;
  frame: CardFrame;
}) {
  return (
    <div
      className="absolute w-full flex items-center"
      style={{
        bottom: "60px",
        backgroundImage: `url('${getFrameAssetUrl(frame, "name_background")}')`,
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

function getFrameAssetUrl(
  frame: CardFrame,
  assetType: "frame_left" | "frame_right" | "name_background",
): string {
  const prefix = (() => {
    switch (frame) {
      case "event":
        return "event_";
      case "character":
        return "";
      default:
        return "";
    }
  })();

  const assetName = (() => {
    switch (assetType) {
      case "frame_left":
        return "card_frame_decoration_left";
      case "frame_right":
        return "card_frame_decoration_right";
      case "name_background":
        return "card_name_background";
    }
  })();

  return `/assets/${prefix}${assetName}.png`;
}
