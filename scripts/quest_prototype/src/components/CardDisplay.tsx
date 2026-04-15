import { useEffect, useState, type ReactNode } from "react";
import type { CardData } from "../types/cards";
import {
  cardAccentTide,
  cardImageUrl,
  TIDE_COLORS,
} from "../data/card-database";
import { tokenizeRulesText, formatTypeLine } from "./card-text";

/** Color used for each symbol type when rendering rules text. */
const SYMBOL_COLORS: Readonly<Record<string, string>> = {
  energy: "#fbbf24",
  spark: "#c084fc",
  trigger: "#f97316",
  fast: "#facc15",
};
const EVENT_CHROME_COLOR = "#c084fc";

/** Props for the CardDisplay component. */
interface CardDisplayProps {
  card: CardData;
  onClick?: () => void;
  selected?: boolean;
  selectionColor?: string;
  /** When set, tints the card's stat values and rules text in this color. */
  tintColor?: string;
  /** Additional CSS class name for the root element. */
  className?: string;
  /** Use larger text sizes for rules text, name, type line, and stats. */
  large?: boolean;
}

/** Renders styled rules text, replacing special symbols with colored spans. */
function renderRulesText(text: string): ReactNode[] {
  return tokenizeRulesText(text).map((segment, i) => {
    if (segment.kind === "text") {
      return <span key={i}>{segment.value}</span>;
    }
    return (
      <span
        key={i}
        className="font-bold"
        style={{ color: SYMBOL_COLORS[segment.symbol] }}
      >
        {segment.char}
      </span>
    );
  });
}

/**
 * Renders a Dreamtides card with tide-driven chrome.
 */
export function CardDisplay({
  card,
  onClick,
  selected = false,
  selectionColor = "#f97316",
  tintColor,
  className,
  large = false,
}: CardDisplayProps) {
  const [imageError, setImageError] = useState(false);

  useEffect(() => {
    setImageError(false);
  }, [card.cardNumber]);

  const accentTide = cardAccentTide(card);
  const accentColor = TIDE_COLORS[accentTide];
  const chromeColor =
    card.cardType === "Event" ? EVENT_CHROME_COLOR : accentColor;
  const borderColor =
    card.cardType === "Event"
      ? chromeColor
      : accentTide === "Neutral"
      ? "rgba(255, 255, 255, 0.18)"
      : `${chromeColor}55`;
  const nameColor =
    accentTide === "Neutral" ? "#f8fafc" : accentColor;

  const borderStyle = selected
    ? { boxShadow: `0 0 0 3px ${selectionColor}, 0 0 12px ${selectionColor}` }
    : card.cardType === "Event"
      ? { boxShadow: `0 0 18px ${EVENT_CHROME_COLOR}26` }
      : {};

  const isInteractive = onClick !== undefined;

  return (
    <div
      className={`relative flex flex-col overflow-hidden rounded-lg transition-transform duration-200${isInteractive ? " cursor-pointer hover:scale-[1.02]" : ""}${className ? ` ${className}` : ""}`}
      style={{
        aspectRatio: "2 / 3",
        background: "linear-gradient(145deg, #1a1025 0%, #0f0a18 60%, #0d0814 100%)",
        border: `1px solid ${borderColor}`,
        ...borderStyle,
      }}
      onClick={onClick}
      {...(isInteractive
        ? {
            role: "button" as const,
            tabIndex: 0,
            onKeyDown: (e: React.KeyboardEvent) => {
              if (e.key === "Enter" || e.key === " ") {
                onClick();
              }
            },
          }
        : {})}
    >
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-1"
        style={{
          background: `linear-gradient(90deg, rgba(255, 255, 255, 0.08) 0%, ${chromeColor} 50%, rgba(255, 255, 255, 0.08) 100%)`,
          opacity: card.cardType === "Event" || accentTide !== "Neutral" ? 0.8 : 0.35,
        }}
      />

      {/* Energy cost badge */}
      <div className={`absolute ${large ? "top-2 left-2" : "top-1.5 left-1.5"} z-10 flex flex-col items-center gap-1`}>
        <div
          className={`flex items-center gap-0.5 rounded-full ${large ? "px-2.5 py-1 text-base" : "px-1.5 py-0.5 text-xs"} font-bold shadow-md`}
          style={{
            background: "rgba(0, 0, 0, 0.75)",
            border: "1px solid rgba(251, 191, 36, 0.5)",
          }}
        >
          <span style={{ color: tintColor ?? "#fbbf24" }}>{"\u25CF"}</span>
          <span style={{ color: tintColor ?? "#ffffff" }}>
            {card.energyCost !== null ? String(card.energyCost) : "X"}
          </span>
        </div>
      </div>

      {/* Fast badge */}
      {card.isFast && (
        <div
          className={`absolute ${large ? "top-2 right-2 px-2.5 py-1 text-base" : "top-1.5 right-1.5 px-1.5 py-0.5 text-xs"} z-10 flex items-center rounded-full font-bold shadow-md`}
          style={{
            background: "rgba(0, 0, 0, 0.75)",
            border: "1px solid rgba(250, 204, 21, 0.5)",
            color: "#facc15",
          }}
        >
          {"\u21AF"}
        </div>
      )}

      {/* Card art area */}
      <div className="relative w-full" style={{ height: "45%" }}>
        {!imageError ? (
          <img
            src={cardImageUrl(card.cardNumber)}
            alt={card.name}
            className="h-full w-full object-cover"
            onError={() => {
              setImageError(true);
            }}
            loading="lazy"
          />
        ) : (
          <div
            className="flex h-full w-full items-center justify-center p-2"
            style={{
              background: `linear-gradient(135deg, ${accentColor}24, rgba(255, 255, 255, 0.05))`,
            }}
          >
            <span
              className="text-center text-sm font-medium opacity-60"
              style={{ color: nameColor }}
            >
              {card.name}
            </span>
          </div>
        )}
        {/* Gradient overlay at bottom of art */}
        <div
          className="pointer-events-none absolute inset-x-0 bottom-0 h-4"
          style={{
            background: "linear-gradient(transparent, #1a1025)",
          }}
        />
      </div>

      {/* Card info area */}
      <div className={`flex min-h-0 flex-1 flex-col ${large ? "px-3 pt-2 pb-2" : "px-2 pt-1 pb-1.5"}`}>
        {/* Card name */}
        <h3
          className={`truncate ${large ? "text-xl" : "text-sm"} leading-tight font-bold`}
          style={{ color: nameColor }}
        >
          {card.name}
        </h3>

        <div className="mt-0.5 flex items-center gap-1.5">
          <span
            className={`truncate ${large ? "text-sm" : "text-[10px]"} opacity-50`}
            style={{ color: "#e2e8f0" }}
          >
            {formatTypeLine(card)}
          </span>
        </div>

        {/* Rules text */}
        <div
          className={`mt-1 min-h-0 flex-1 overflow-y-auto ${large ? "text-base leading-snug" : "text-[10px] leading-tight"} opacity-80`}
          style={{ color: tintColor ?? "#e2e8f0" }}
        >
          {renderRulesText(card.renderedText)}
        </div>

        {/* Spark badge for Characters */}
        {card.spark !== null && (
          <div className="mt-auto flex items-center justify-end pt-0.5">
            <div
              className={`flex items-center gap-0.5 rounded-full ${large ? "px-2.5 py-1 text-sm" : "px-1.5 py-0.5 text-[10px]"} font-bold`}
              style={{
                background: "rgba(0, 0, 0, 0.5)",
                border: "1px solid rgba(192, 132, 252, 0.5)",
              }}
            >
              <span style={{ color: tintColor ?? "#c084fc" }}>{"\u234F"}</span>
              <span style={{ color: tintColor ?? "#ffffff" }}>{String(card.spark)}</span>
            </div>
          </div>
        )}
      </div>

      {/* Bottom accent */}
      <div
        className="pointer-events-none absolute inset-x-0 bottom-0 h-px"
        style={{ background: borderColor }}
      />
    </div>
  );
}
