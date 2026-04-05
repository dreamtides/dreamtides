import type { CardView, DisplayPlayer, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

const SLOT_COUNT = 8;

interface RankZoneProps {
  label: string;
  cards: CardView[];
  player: DisplayPlayer;
  rank: "Front" | "Back";
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

function cardsForRank(
  cards: CardView[],
  player: DisplayPlayer,
  rank: "Front" | "Back",
): CardView[] {
  return cards.filter((c) => {
    const pos = c.position.position;
    if (typeof pos === "string") return false;
    if ("OnBattlefield" in pos) {
      const [p, r] = (pos as { OnBattlefield: [DisplayPlayer, string, number] }).OnBattlefield;
      return p === player && r === rank;
    }
    return false;
  });
}

function slotIndex(card: CardView): number {
  const pos = card.position.position;
  if (typeof pos !== "string" && "OnBattlefield" in pos) {
    return (pos as { OnBattlefield: [DisplayPlayer, string, number] }).OnBattlefield[2];
  }
  return 0;
}

export function RankZone({
  label,
  cards,
  player,
  rank,
  onAction,
  disabled,
}: RankZoneProps) {
  const rankCards = cardsForRank(cards, player, rank);
  const cardBySlot: (CardView | null)[] = Array.from({ length: SLOT_COUNT }, () => null);
  for (const card of rankCards) {
    const idx = slotIndex(card);
    if (idx >= 0 && idx < SLOT_COUNT) {
      cardBySlot[idx] = card;
    }
  }

  return (
    <div className="relative py-0.5 px-2">
      <span
        className="absolute left-2 text-[10px]"
        style={{ color: "var(--color-text-dim)", top: 0 }}
      >
        {label}
      </span>
      <div className="flex gap-1 justify-center items-center pt-3">
        {cardBySlot.map((card, i) =>
          card ? (
            <CardDisplay
              key={card.id}
              card={card}
              onAction={onAction}
              disabled={disabled}
              compact
              battlefield
            />
          ) : (
            <div
              key={`empty-${i}`}
              style={{
                width: 100,
                height: 60,
                border: "1px dashed var(--color-border)",
                borderRadius: 4,
              }}
            />
          ),
        )}
      </div>
    </div>
  );
}
