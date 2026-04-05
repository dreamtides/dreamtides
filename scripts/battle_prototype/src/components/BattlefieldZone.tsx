import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface BattlefieldZoneProps {
  label: string;
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function BattlefieldZone({
  label,
  cards,
  onAction,
  disabled,
}: BattlefieldZoneProps) {
  const sorted = [...cards].sort((a, b) => a.position.sorting_key - b.position.sorting_key);
  return (
    <div
      className="flex gap-1 justify-center items-center py-1 flex-wrap relative"
      style={{ minHeight: sorted.length === 0 ? 40 : 60 }}
    >
      <span
        className="absolute left-2 text-[10px]"
        style={{ color: "var(--color-text-dim)", top: 2 }}
      >
        {label}
      </span>
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
          battlefield
        />
      ))}
    </div>
  );
}
