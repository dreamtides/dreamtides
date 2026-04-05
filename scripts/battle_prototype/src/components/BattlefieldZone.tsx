import { useState } from "react";
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
  const [dragOverSlot, setDragOverSlot] = useState<number | null>(null);
  const [isDragging, setIsDragging] = useState(false);

  const rankCards = cardsForRank(cards, player, rank);
  const cardBySlot: (CardView | null)[] = Array.from({ length: SLOT_COUNT }, () => null);
  for (const card of rankCards) {
    const idx = slotIndex(card);
    if (idx >= 0 && idx < SLOT_COUNT) {
      cardBySlot[idx] = card;
    }
  }

  const canDrag = !disabled && player === "User";

  const handleDragStart = (e: React.DragEvent, card: CardView) => {
    if (!canDrag) {
      e.preventDefault();
      return;
    }
    e.dataTransfer.setData("text/plain", JSON.stringify({ cardId: card.id }));
    e.dataTransfer.effectAllowed = "move";
    setIsDragging(true);
  };

  const handleDragEnd = () => {
    setIsDragging(false);
    setDragOverSlot(null);
  };

  const handleDrop = (e: React.DragEvent, targetPosition: number) => {
    e.preventDefault();
    setDragOverSlot(null);
    setIsDragging(false);
    try {
      const data = JSON.parse(e.dataTransfer.getData("text/plain")) as { cardId: string };
      const characterId = parseInt(data.cardId, 10);
      if (isNaN(characterId)) return;
      if (rank === "Front") {
        onAction({ BattleAction: { MoveCharacterToFrontRank: [characterId, targetPosition] } });
      } else {
        onAction({ BattleAction: { MoveCharacterToBackRank: [characterId, targetPosition] } });
      }
    } catch {
      // Ignore invalid drag data
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
  };

  const handleDragEnter = (e: React.DragEvent, slotIndex: number) => {
    e.preventDefault();
    setDragOverSlot(slotIndex);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    const relatedTarget = e.relatedTarget as HTMLElement | null;
    if (!relatedTarget || !(e.currentTarget as HTMLElement).contains(relatedTarget)) {
      setDragOverSlot(null);
    }
  };

  const slotDropStyle = (i: number): React.CSSProperties => {
    if (dragOverSlot !== i) return {};
    return {
      boxShadow: "0 0 6px 2px rgba(34, 197, 94, 0.6)",
      borderColor: "rgb(34, 197, 94)",
    };
  };

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
            <div
              key={card.id}
              onDrop={(e) => handleDrop(e, i)}
              onDragOver={handleDragOver}
              onDragEnter={(e) => handleDragEnter(e, i)}
              onDragLeave={handleDragLeave}
              style={{
                borderRadius: 4,
                transition: "box-shadow 0.15s, border-color 0.15s",
                ...slotDropStyle(i),
              }}
            >
              <CardDisplay
                card={card}
                onAction={onAction}
                disabled={disabled}
                compact
                battlefield
                draggable={canDrag}
                onDragStart={(e) => handleDragStart(e, card)}
                onDragEnd={handleDragEnd}
              />
            </div>
          ) : (
            <div
              key={`empty-${i}`}
              onDrop={(e) => handleDrop(e, i)}
              onDragOver={handleDragOver}
              onDragEnter={(e) => handleDragEnter(e, i)}
              onDragLeave={handleDragLeave}
              style={{
                width: 90,
                height: 48,
                border: `1px dashed ${dragOverSlot === i ? "rgb(34, 197, 94)" : "var(--color-border)"}`,
                borderRadius: 4,
                transition: "box-shadow 0.15s, border-color 0.15s",
                ...slotDropStyle(i),
              }}
            />
          ),
        )}
      </div>
      {isDragging && (
        <div
          className="absolute inset-0 pointer-events-none rounded"
          style={{
            border: "1px solid rgba(34, 197, 94, 0.3)",
            borderRadius: 4,
          }}
        />
      )}
    </div>
  );
}
