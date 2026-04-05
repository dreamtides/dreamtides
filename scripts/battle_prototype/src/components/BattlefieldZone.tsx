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
      className="flex gap-1 justify-center items-center py-1 flex-wrap"
      style={{ minHeight: sorted.length === 0 ? 40 : 60 }}
    >
      {sorted.length === 0 && (
        <span style={{ color: "var(--color-text-dim)", fontSize: 11 }}>
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
