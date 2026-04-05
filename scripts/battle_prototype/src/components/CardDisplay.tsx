import { useEffect, useState } from "react";
import type { CardView, DisplayColor, GameAction } from "../types/battle";

interface CardDisplayProps {
  card: CardView;
  onAction?: (action: GameAction) => void;
  disabled?: boolean;
  compact?: boolean;
}

function colorToCSS(c: DisplayColor): string {
  return `rgba(${Math.round(c.red * 255)}, ${Math.round(c.green * 255)}, ${Math.round(c.blue * 255)}, ${c.alpha})`;
}

// Map from imageNumber (in sprite address) to cardNumber (filename)
let imageNumberToCardNumber: Record<string, number> | null = null;
let loadingPromise: Promise<void> | null = null;

function loadCardData(): Promise<void> {
  if (imageNumberToCardNumber) return Promise.resolve();
  if (loadingPromise) return loadingPromise;
  loadingPromise = fetch("/card-data.json")
    .then((r) => r.json())
    .then((data: Array<{ imageNumber: number; cardNumber: number }>) => {
      const map: Record<string, number> = {};
      for (const card of data) {
        map[String(card.imageNumber)] = card.cardNumber;
      }
      imageNumberToCardNumber = map;
    })
    .catch(() => {
      imageNumberToCardNumber = {};
    });
  return loadingPromise;
}

function getCardImageUrl(card: CardView): string | null {
  if (!card.revealed) return null;
  const img = card.revealed.image;
  if ("Sprite" in img) {
    const sprite = img.Sprite.sprite;
    const match = /(\d+)/.exec(sprite);
    if (match && imageNumberToCardNumber) {
      const cardNum = imageNumberToCardNumber[match[1]];
      if (cardNum != null) {
        return `/cards/${cardNum}.webp`;
      }
    }
  }
  return null;
}

/** Strip ALL tags for plain text display */
function stripAllTags(text: string): string {
  return text.replace(/<\/?[^>]+>/gi, "").trim();
}

/** Strip Unity-specific rich text tags but keep valid HTML like <b>, <i> */
function stripUnityTags(text: string): string {
  return text
    .replace(/<color=[^>]*>/gi, "")
    .replace(/<\/color>/gi, "")
    .replace(/<size=[^>]*>/gi, "")
    .replace(/<\/size>/gi, "")
    .replace(/<line-height=[^>]*>/gi, "")
    .replace(/<\/line-height=[^>]*>/gi, "");
}

export function CardDisplay({
  card,
  onAction,
  disabled,
  compact,
}: CardDisplayProps) {
  const [dataLoaded, setDataLoaded] = useState(imageNumberToCardNumber != null);

  useEffect(() => {
    if (!dataLoaded) {
      void loadCardData().then(() => setDataLoaded(true));
    }
  }, [dataLoaded]);

  const revealed = card.revealed;
  const isFaceDown = card.card_facing === "FaceDown" || !revealed;

  const selectOrderCardId = revealed?.actions.can_select_order as number | undefined;
  const clickAction = revealed?.actions.can_play ?? revealed?.actions.on_click;
  const isClickable = !disabled && (clickAction != null || selectOrderCardId != null);

  const outlineColor = revealed?.outline_color
    ? colorToCSS(revealed.outline_color)
    : "var(--color-border)";

  const handleClick = () => {
    if (!isClickable || !onAction) return;
    if (clickAction) {
      onAction(clickAction);
    } else if (selectOrderCardId != null) {
      const pos = card.position.position;
      const target = typeof pos !== "string" && "CardOrderSelector" in pos
        ? (pos as Record<string, string>).CardOrderSelector
        : "Deck";
      onAction({
        BattleAction: {
          SelectOrderForDeckCard: { card_id: selectOrderCardId, target },
        },
      });
    }
  };

  if (isFaceDown) {
    return (
      <div
        className="rounded flex items-center justify-center text-xs"
        style={{
          width: compact ? 60 : 120,
          height: compact ? 36 : 80,
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text-dim)",
        }}
      >
        {card.prefab}
      </div>
    );
  }

  const imageUrl = getCardImageUrl(card);

  return (
    <div
      onClick={handleClick}
      className="rounded overflow-hidden flex flex-col"
      style={{
        width: compact ? 100 : 140,
        minHeight: compact ? 60 : 180,
        background: "var(--color-surface)",
        border: `2px solid ${outlineColor}`,
        cursor: isClickable ? "pointer" : "default",
        opacity: disabled ? 0.5 : 1,
      }}
    >
      {imageUrl && (
        <img
          src={imageUrl}
          alt={revealed.name}
          className="w-full object-cover"
          style={{ height: compact ? 40 : 80 }}
        />
      )}
      <div className="p-1 flex flex-col gap-0.5" style={{ fontSize: compact ? 9 : 11 }}>
        <div className="flex justify-between items-center">
          <span className="font-bold truncate" style={{ maxWidth: "70%" }}>
            {revealed.name}
          </span>
          {revealed.cost != null && (
            <span style={{ color: "var(--color-gold)" }}>{revealed.cost}</span>
          )}
        </div>
        <div className="flex justify-between" style={{ color: "var(--color-text-dim)", fontSize: compact ? 8 : 9 }}>
          <span>{stripAllTags(revealed.card_type)}</span>
          {revealed.spark != null && <span>&#x2351;{revealed.spark}</span>}
        </div>
        {revealed.is_fast && (
          <span style={{ color: "var(--color-gold-light)", fontSize: compact ? 7 : 8 }}>
            &#x21AF; Fast
          </span>
        )}
        {!compact && revealed.rules_text && (
          <div
            className="mt-1"
            style={{
              fontSize: 9,
              color: "var(--color-text-dim)",
              lineHeight: 1.3,
            }}
            dangerouslySetInnerHTML={{ __html: stripUnityTags(revealed.rules_text) }}
          />
        )}
      </div>
    </div>
  );
}
