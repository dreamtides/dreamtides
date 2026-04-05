import type { PlayerView } from "../types/battle";

interface PlayerStatusProps {
  player: PlayerView;
  label: string;
  deckCount: number;
  voidCount: number;
  banishedCount: number;
}

export function PlayerStatus({
  player,
  label,
  deckCount,
  voidCount,
  banishedCount,
}: PlayerStatusProps) {
  return (
    <div
      className="flex items-center justify-between px-4 py-2"
      style={{
        background: "var(--color-surface)",
        borderBottom: "1px solid var(--color-border)",
      }}
    >
      <span className="font-bold">{label}</span>
      <div className="flex gap-4 text-sm">
        <span>
          Score:{" "}
          <span style={{ color: "var(--color-gold)" }}>{player.score}</span>
        </span>
        <span>
          Energy:{" "}
          <span style={{ color: "var(--color-primary-light)" }}>
            {player.energy}/{player.produced_energy}
          </span>
        </span>
        <span>
          Spark:{" "}
          <span style={{ color: "var(--color-gold-light)" }}>
            {player.total_spark}
          </span>
        </span>
        <span style={{ color: "var(--color-text-dim)" }}>
          Deck: {deckCount}
        </span>
        <span style={{ color: "var(--color-text-dim)" }}>
          Void: {voidCount}
        </span>
        {banishedCount > 0 && (
          <span style={{ color: "var(--color-text-dim)" }}>
            Banished: {banishedCount}
          </span>
        )}
      </div>
    </div>
  );
}
