import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface BattlefieldZoneProps {
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function BattlefieldZone({
  cards,
  onAction,
  disabled,
}: BattlefieldZoneProps) {
  const sorted = [...cards].sort((a, b) => a.position.sorting_key - b.position.sorting_key);
  return (
    <div
      className="flex gap-2 justify-center items-center py-2 min-h-[100px]"
    >
      {sorted.length === 0 && (
        <span style={{ color: "var(--color-text-dim)", fontSize: 12 }}>
          No characters
        </span>
      )}
      {sorted.map((card) => (
        <CardDisplay
          key={card.id}
          card={card}
          onAction={onAction}
          disabled={disabled}
          compact
        />
      ))}
    </div>
  );
}
