import { CardDisplay } from "../../components/CardDisplay";
import type { BattleCardInstance } from "../types";
import { battleCardDisplayFromInstance } from "./BattleCardView";

const PREVIEW_HEIGHT = 520;
const PREVIEW_WIDTH = 320;
const VIEWPORT_MARGIN = 20;

export function BattleCardHoverPreview({
  card,
  pointer,
}: {
  card: BattleCardInstance;
  pointer: {
    x: number;
    y: number;
  };
}) {
  const maxLeft = Math.max(VIEWPORT_MARGIN, window.innerWidth - PREVIEW_WIDTH - VIEWPORT_MARGIN);
  const maxTop = Math.max(VIEWPORT_MARGIN, window.innerHeight - PREVIEW_HEIGHT - VIEWPORT_MARGIN);
  const left = Math.min(Math.max(VIEWPORT_MARGIN, pointer.x + 24), maxLeft);
  const top = Math.min(Math.max(VIEWPORT_MARGIN, pointer.y - 28), maxTop);

  return (
    <div
      data-battle-hover-preview=""
      className="battle-hover-preview"
      style={{ left: `${String(left)}px`, top: `${String(top)}px` }}
    >
      <CardDisplay
        card={battleCardDisplayFromInstance(card)}
        className="h-full w-full"
        large
      />
    </div>
  );
}
