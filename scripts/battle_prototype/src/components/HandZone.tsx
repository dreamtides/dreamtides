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
    <div className="flex gap-2 justify-center items-end py-2 flex-wrap">
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
