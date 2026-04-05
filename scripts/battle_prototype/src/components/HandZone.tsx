import type { CardView, GameAction } from "../types/battle";
import { CardDisplay } from "./CardDisplay";

interface HandZoneProps {
  cards: CardView[];
  onAction: (action: GameAction) => void;
  disabled: boolean;
}

export function HandZone({ cards, onAction, disabled }: HandZoneProps) {
  const sorted = [...cards].sort(
    (a, b) => a.position.sorting_key - b.position.sorting_key,
  );

  return (
    <div className="flex gap-1 justify-center items-end flex-wrap" style={{ maxWidth: "100%", padding: "4px 4px" }}>
      {sorted.map((card) => (
        <CardDisplay
          key={card.id}
          card={card}
          onAction={onAction}
          disabled={disabled}
        />
      ))}
    </div>
  );
}
