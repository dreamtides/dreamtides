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

export type CardLayout = "default" | "battlefield";

export type CardProps = {
  card: CardView;
  className?: string;
  width?: number;
  layout?: CardLayout;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export function Card({ card, className, width = BASE_WIDTH }: CardProps) {
  const id = JSON.stringify(card.id);
  const px = (x: number,) => `${x * (width / BASE_WIDTH)}px`;

  return (
    <motion.div
      id={id}
      layoutId={id}
      className={cn("flex relative m-2", className)}
      style={{
        width: px(BASE_WIDTH),
        height: px(BASE_HEIGHT),
      }}
    >
      {card.revealed ? <RevealedCard card={card.revealed} px={px} /> : <HiddenCard />}
    </motion.div>
  );
}

function RevealedCard({ card, px }: { card: RevealedCardView, px: (x: number) => string }) {
  return (
    <>
      <CardImage image={card.image} px={px} />
      <RulesText text={card.rulesText} px={px} />
      <EnergyCost cost={card.cost} px={px} />
      <FrameDecoration side="left" frame={card.frame} px={px} />
      <FrameDecoration side="right" frame={card.frame} px={px} />
      {card.spark && <SparkValue spark={card.spark} px={px} />}
      <CardName name={card.name} cardType={card.cardType} frame={card.frame} px={px} />
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

function EnergyCost({ cost, px }: { cost: number, px: (x: number) => string }) {
  return (
    <div
      className="absolute"
      style={{
        width: px(45),
        height: px(45),
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
          fontSize: px(35),
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
  px,
}: {
  side: "left" | "right";
  frame: CardFrame;
  px: (x: number) => string;
}) {
  return (
    <div
      className="absolute"
      style={{
        bottom: 0,
        [side]: 0,
        width: "100%",
        height: px(95),
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
  px,
}: {
  name: string;
  cardType: string;
  frame: CardFrame;
  px: (x: number) => string;
}) {
  return (
    <div
      className="absolute w-full flex items-center"
      style={{
        bottom: px(60),
        backgroundImage: `url('${getFrameAssetUrl(frame, "name_background")}')`,
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        height: px(35),
      }}
    >
      <div className="flex justify-between w-full items-center">
        <span
          className="font-medium"
          style={{
            color: "white",
            fontFamily: "'EB Garamond', serif",
            paddingLeft: px(10),
            fontSize: px(13),
          }}
        >
          {name}
        </span>
        <span
          style={{
            color: "white",
            fontFamily: "'EB Garamond', serif",
            paddingRight: px(10),
            fontSize: px(10),
          }}
        >
          {cardType}
        </span>
      </div>
    </div>
  );
}

function RulesText({ text, px }: { text: string, px: (x: number) => string }) {
  return (
    <div
      className="absolute flex items-center"
      style={{
        bottom: px(2),
        backgroundImage: "url('/assets/rules_text_background.png')",
        backgroundSize: "cover",
        backgroundRepeat: "no-repeat",
        height: px(70),
        left: px(10),
        right: px(10),
      }}
    >
      <span
        style={{
          color: "black",
          fontFamily: "'Libre Baskerville', Georgia, 'Times New Roman', serif",
          paddingLeft: px(25),
          paddingRight: px(15),
          fontSize: px(10),
          lineHeight: "1.0",
        }}
      >
        {text}
      </span>
    </div>
  );
}

function CardImage({ image, px }: { image: DisplayImage, px: (x: number) => string }) {
  return (
    <div
      className="absolute top-0 w-full rounded-xl overflow-hidden"
      style={{
        height: px(260),
        width: "100%",
        backgroundImage: `url("${image.image}")`,
        backgroundSize: "cover",
        backgroundRepeat: "no-repeat",
        backgroundPosition: `${image.imageOffsetX ?? 50}% ${image.imageOffsetY ?? 50}%`,
      }}
    />
  );
}

function SparkValue({ spark, px }: { spark: number, px: (x: number) => string }) {
  return (
    <div
      className="absolute"
      style={{
        width: px(30),
        height: px(30),
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
          fontSize: px(20),
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
