import { LayoutGroup } from "motion/react";
import {
  BattleView,
  CardView,
  commands,
  events,
  Position,
} from "../../bindings";
import { Loading } from "../../components/common/Loading";
import NavigationBar from "../../components/common/NavigationBar";
import EnemyHand from "./EnemyHand";
import BattlePlayerStatus from "./BattlePlayerStatus";
import Battlefield from "./Battlefield";
import UserHand from "./UserHand";
import { useEffect, useState } from "react";

type BattleScreenProps = {};

export default function BattleScreen({}: BattleScreenProps) {
  const [isAnimating, _setIsAnimating] = useState(false);
  const [_updateQueue, setUpdateQueue] = useState<BattleView[]>([]);
  const [battleView, setBattleView] = useState<BattleView | null>(null);

  useEffect(() => {
    console.log("connecting");
    commands.connect("123");
  }, []);

  useEffect(() => {
    console.log("listening");
    events.updateEvent.listen((event) => {
      console.log("got update", event.payload.id);
      if (isAnimating) {
        setUpdateQueue((prev) => [...prev, event.payload]);
      } else {
        setBattleView(event.payload);
      }
    });
    return () => {
      console.log("unlistening");
    };
  }, []);

  if (battleView == null) {
    return <Loading />;
  }

  const cards = buildCardMap(battleView);
  return (
    <div className="flex flex-col h-screen w-screen">
      <LayoutGroup>
        <NavigationBar>
          <EnemyHand battleId="123" />
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
