import { cn } from "@nextui-org/react";
import {
  CardView,
  DisplayImage,
  RevealedCardView,
  CardFrame,
} from "../../bindings";
import { motion } from "motion/react";
import { memo } from "react";
import isEqual from "lodash/isEqual";

export const CARD_ASPECT_RATIO = 1.6;
const BASE_WIDTH = 200;
const BASE_HEIGHT = BASE_WIDTH * CARD_ASPECT_RATIO;

export type CardSize = {
  vw: number;
  vh: number;
};

export type CardLayout = "default" | "battlefield";

export type CardProps = {
  card: CardView;
  rotate?: boolean;
  style?: React.CSSProperties;
  width?: number;
  layout?: CardLayout;
};

/**
 * Renders a primary game card.
 *
 * This does not include other types of cards, such as dreamsigns, path cards,
 * etc which have their own components.
 */
export const Card = memo(function Card({
  card,
  rotate = false,
  style,
  width = BASE_WIDTH,
  layout = "default",
}: CardProps) {
  const id = JSON.stringify(card.id);
  const px = (x: number) => `${x * (width / BASE_WIDTH)}px`;

  return (
    <motion.div
      layoutId={id}
      initial={{
        rotate: rotate ? -90 : 0,
        originX: 0,
        originY: 0,
      }}
      animate={{
        rotate: rotate ? -90 : 0,
        originX: 0,
        originY: 0,
      }}
      style={{
        display: "flex",
        position: "relative",
        flexShrink: 0,
        width: px(BASE_WIDTH),
        height: layout === "battlefield" ? px(260) : px(BASE_HEIGHT),
        ...style,
      }}
    >
      {!card.revealed ? (
        <HiddenCard />
      ) : layout === "battlefield" ? (
        <BattlefieldCard card={card.revealed} px={px} />
      ) : (
        <RevealedCard card={card.revealed} px={px} />
      )}
    </motion.div>
  );
},
isEqual);

const RevealedCard = memo(function RevealedCard({
  card,
  px,
}: {
  card: RevealedCardView;
  px: (x: number) => string;
}) {
  return (
    <>
      <CardImage image={card.image} px={px} />
      <RulesText text={card.rulesText} px={px} />
      <EnergyCost cost={card.cost} px={px} />
      <FrameDecoration side="left" frame={card.frame} px={px} />
      <FrameDecoration side="right" frame={card.frame} px={px} />
      {card.spark && <SparkValue spark={card.spark} px={px} />}
      <CardName
        name={card.name}
        cardType={card.cardType}
        frame={card.frame}
        px={px}
      />
    </>
  );
},
isEqual);

const BattlefieldCard = memo(function BattlefieldCard({
  card,
  px,
}: {
  card: RevealedCardView;
  px: (x: number) => string;
}) {
  return (
    <>
      <CardImage image={card.image} px={px} />
      {card.spark != null && (
        <SparkValue spark={card.spark} px={px} size={80} />
      )}
    </>
  );
},
isEqual);

const HiddenCard = memo(function HiddenCard() {
  return (
    <div
      style={{
        position: "absolute",
        width: "100%",
        height: "100%",
        borderRadius: "12px",
        backgroundImage: "url('/assets/card_back.png')",
        backgroundSize: "cover",
      }}
    />
  );
}, isEqual);

const EnergyCost = memo(function EnergyCost({
  cost,
  px,
}: {
  cost: number;
  px: (x: number) => string;
}) {
  return (
    <div
      style={{
        position: "absolute",
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
},
isEqual);

const FrameDecoration = memo(function FrameDecoration({
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
      style={{
        position: "absolute",
        bottom: 0,
        [side]: 0,
        width: "100%",
        height: px(95),
        backgroundRepeat: "no-repeat",
        backgroundImage: `url('${getFrameAssetUrl(
          frame,
          side === "left" ? "frame_left" : "frame_right"
        )}')`,
        backgroundSize: "contain",
        transform: side === "right" ? "scaleX(-1)" : undefined,
      }}
    />
  );
},
isEqual);

const CardName = memo(function CardName({
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
      style={{
        position: "absolute",
        width: "100%",
        display: "flex",
        alignItems: "center",
        bottom: px(60),
        backgroundImage: `url('${getFrameAssetUrl(frame, "name_background")}')`,
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        height: px(35),
      }}
    >
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          width: "100%",
          alignItems: "center",
        }}
      >
        <span
          style={{
            color: "white",
            fontFamily: "'EB Garamond', serif",
            paddingLeft: px(10),
            fontSize: px(13),
            fontWeight: 500,
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
},
isEqual);

const RulesText = memo(function RulesText({
  text,
  px,
}: {
  text: string;
  px: (x: number) => string;
}) {
  return (
    <div
      style={{
        position: "absolute",
        display: "flex",
        alignItems: "center",
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
},
isEqual);

const CardImage = memo(function CardImage({
  image,
  px,
}: {
  image: DisplayImage;
  px: (x: number) => string;
}) {
  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        width: "100%",
        borderRadius: "12px",
        overflow: "hidden",
        height: px(260),
        backgroundImage: `url("${image.image}")`,
        backgroundSize: "cover",
        backgroundRepeat: "no-repeat",
        backgroundPosition: `${image.imageOffsetX ?? 50}% ${
          image.imageOffsetY ?? 50
        }%`,
      }}
    />
  );
},
isEqual);

const SparkValue = memo(function SparkValue({
  spark,
  px,
  size = 30,
}: {
  spark: number;
  px: (x: number) => string;
  size?: number;
}) {
  return (
    <div
      style={{
        position: "absolute",
        width: px(size),
        height: px(size),
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
          fontSize: px((size * 2) / 3),
          lineHeight: 1,
          WebkitTextStroke: "0.1em black",
          paintOrder: "stroke",
        }}
      >
        {spark}
      </span>
    </div>
  );
},
isEqual);

function getFrameAssetUrl(
  frame: CardFrame,
  assetType: "frame_left" | "frame_right" | "name_background"
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
