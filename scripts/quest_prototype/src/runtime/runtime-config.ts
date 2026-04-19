export interface RuntimeConfig {
  battleMode: "auto" | "playable";
  seedOverride: number | null;
  startInBattle: boolean;
}

export function parseRuntimeConfig(search: string): RuntimeConfig {
  const params = new URLSearchParams(search);
  const battleParams = params.getAll("battle");
  const battleMode =
    battleParams.length === 1 && battleParams[0] === "playable"
      ? "playable"
      : "auto";

  return {
    battleMode,
    seedOverride: parseSeedOverride(battleMode, params.get("seed")),
    startInBattle:
      battleMode === "playable" && params.get("startInBattle") === "1",
  };
}

function parseSeedOverride(
  battleMode: RuntimeConfig["battleMode"],
  rawSeed: string | null,
): number | null {
  if (battleMode !== "playable" || rawSeed === null || rawSeed === "") {
    return null;
  }

  if (!/^\d+$/.test(rawSeed)) {
    return null;
  }

  const parsed = Number(rawSeed);
  if (!Number.isFinite(parsed) || !Number.isInteger(parsed) || parsed < 0) {
    return null;
  }

  return parsed;
}
