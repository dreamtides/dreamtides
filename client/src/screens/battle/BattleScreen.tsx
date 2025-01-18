import { LayoutGroup } from "motion/react";
import {
  BattleView,
  CardView,
  ClientBattleId,
  commands,
  Position,
} from "../../bindings";
import { ErrorState } from "../../components/common/ErrorState";
import { Loading } from "../../components/common/Loading";
import NavigationBar from "../../components/common/NavigationBar";
import EnemyHand from "./EnemyHand";
import useSWR from "swr";
import BattlePlayerStatus from "./BattlePlayerStatus";
import Battlefield from "./Battlefield";
import UserHand from "./UserHand";
import { useState } from "react";

type BattleFetchResult =
  | { battle: BattleView }
  | { error: Error }
  | { isLoading: boolean };

function useBattle(id: ClientBattleId, scene: number): BattleFetchResult {
  const { data, error, isLoading } = useSWR([id, scene], ([id, scene]) =>
    commands.fetchBattle(id, scene),
  );

  if (isLoading) {
    return { isLoading: true };
  } else if (error || data == null) {
    return { error };
  } else {
    return { battle: data };
  }
}

type BattleScreenProps = {};

export default function BattleScreen({}: BattleScreenProps) {
  const [sceneNumber, setSceneNumber] = useState(0);
  const result = useBattle("123", sceneNumber);

  const handleSceneChange = () => {
    setSceneNumber((prev) => (prev + 1) % 3);
  };

  if ("isLoading" in result) {
    return <Loading />;
  } else if ("error" in result) {
    return <ErrorState />;
  }

  const cards = buildCardMap(result.battle);
  return (
    <div className="flex flex-col h-screen w-screen">
      <LayoutGroup>
        <NavigationBar>
          <EnemyHand battleId="123" onSceneChange={handleSceneChange} />
        </NavigationBar>
        <BattlePlayerStatus
          owner="enemy"
          deck={cards.get(positionKey({ inDeck: "enemy" })) ?? []}
          void={cards.get(positionKey({ inVoid: "enemy" })) ?? []}
        />
        <Battlefield
          owner="enemy"
          cards={cards.get(positionKey({ onBattlefield: "enemy" })) ?? []}
        />
        <Battlefield
          owner="user"
          cards={cards.get(positionKey({ onBattlefield: "user" })) ?? []}
        />
        <BattlePlayerStatus
          owner="user"
          deck={cards.get(positionKey({ inDeck: "user" })) ?? []}
          void={cards.get(positionKey({ inVoid: "user" })) ?? []}
        />
        <UserHand cards={cards.get(positionKey({ inHand: "user" })) ?? []} />
      </LayoutGroup>
    </div>
  );
}

type PositionKey = string;

function positionKey(position: Position): PositionKey {
  return JSON.stringify(position);
}

function buildCardMap(battle: BattleView): Map<PositionKey, CardView[]> {
  const map = new Map<PositionKey, CardView[]>();
  for (const card of battle.cards) {
    map.set(positionKey(card.position.position), [
      ...(map.get(positionKey(card.position.position)) ?? []),
      card,
    ]);
  }
  return map;
}
