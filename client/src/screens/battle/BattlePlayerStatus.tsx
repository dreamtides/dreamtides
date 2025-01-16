import { DisplayPlayer } from "../../bindings";

type BattlePlayerStatusProps = {
  owner: DisplayPlayer;
};

/**
 * Displays a player's deck, void, and current score
 */
export default function BattlePlayerStatus({ owner }: BattlePlayerStatusProps) {
  return (
    <div key={owner} className="flex bg-red-600" style={{ height: "10dvh" }}>
      Player Status
    </div>
  );
}
