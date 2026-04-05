import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface StackZoneProps {
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function StackZone({ cards, onAction, disabled }: StackZoneProps) {
  if (cards.length === 0) return null;

  const sorted = [...cards].sort(
    (a, b) => b.position.sorting_key - a.position.sorting_key,
  );

  return (
    <div
      className="flex gap-2 justify-center items-center py-2 px-4 mx-auto rounded"
      style={{
        border: "2px solid var(--color-gold)",
        background: "rgba(212, 160, 23, 0.05)",
        maxWidth: "80%",
      }}
    >
      <span
        className="text-xs font-bold mr-2"
        style={{ color: "var(--color-gold)" }}
      >
        STACK
      </span>
      {sorted.map((card, i) => (
        <div key={card.id} className="relative">
          {i === 0 && (
            <div
              className="absolute -top-3 left-1/2 -translate-x-1/2 text-[8px] px-1 rounded"
              style={{ background: "var(--color-gold)", color: "#000" }}
            >
              newest
            </div>
          )}
          <CardDisplay
            card={card}
            onAction={onAction}
            disabled={disabled}
            compact
          />
        </div>
      ))}
    </div>
  );
}
