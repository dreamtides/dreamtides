import type { MouseEvent as ReactMouseEvent } from "react";

import type { BattleCommand } from "../debug/commands";
import type { BattleMutableState } from "../types";
import { BattleCardView, battleCardVisualFromInstance } from "./BattleCardView";

export function BattleHandTray({
  canInteract,
  currentEnergy,
  hand,
  onHandCardAction: _onHandCardAction,
  openingHandSize: _openingHandSize,
  playerDrawSkipsTurnOne: _playerDrawSkipsTurnOne,
  selectedCardId,
  state,
  onCardClick,
  onCardContextMenu,
  onCardDoubleClick,
  onCardDragEnd,
  onCardDragStart,
}: {
  canInteract: boolean;
  currentEnergy: number;
  hand: string[];
  onHandCardAction: (command: BattleCommand) => void;
  openingHandSize: number;
  playerDrawSkipsTurnOne: boolean;
  selectedCardId: string | null;
  state: BattleMutableState;
  onCardClick: (battleCardId: string) => void;
  onCardContextMenu?: (battleCardId: string, event: ReactMouseEvent<HTMLDivElement>) => void;
  onCardDoubleClick: (battleCardId: string) => void;
  onCardDragEnd?: () => void;
  onCardDragStart?: (battleCardId: string) => void;
}) {
  return (
    <section data-battle-region="player-hand-tray" className="hand-row">
      <div className="hand-cards">
        {hand.map((battleCardId) => {
          const instance = state.cardInstances[battleCardId];
          if (instance === undefined) {
            return null;
          }
          return (
            <BattleCardView
              key={battleCardId}
              battleCardId={battleCardId}
              variant="hand"
              dataBattleHandCard
              data={battleCardVisualFromInstance(instance)}
              playable={canInteract && instance.definition.energyCost <= currentEnergy}
              selected={selectedCardId === battleCardId}
              unaffordable={instance.definition.energyCost > currentEnergy}
              draggable={canInteract && instance.definition.energyCost <= currentEnergy}
              onClick={() => onCardClick(battleCardId)}
              onDoubleClick={() => onCardDoubleClick(battleCardId)}
              onContextMenu={(event) => {
                event.preventDefault();
                onCardContextMenu?.(battleCardId, event);
              }}
              onDragStart={() => onCardDragStart?.(battleCardId)}
              onDragEnd={() => onCardDragEnd?.()}
            />
          );
        })}
      </div>
    </section>
  );
}
