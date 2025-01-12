type BattlePlayerStatusProps = {};

/**
 * Displays a player's deck, void, and current score
 */
export default function BattlePlayerStatus({}: BattlePlayerStatusProps) {
  return (
    <div className="flex bg-red-600" style={{ height: "10dvh" }}>
      Player Status
    </div>
  );
}
