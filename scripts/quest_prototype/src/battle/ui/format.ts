import type { BattlePhase, BattleResult, BattleSide, BattleZoneId, BrowseableZone } from "../types";

export function formatSideLabel(side: BattleSide): string {
  return side === "player" ? "Player" : "Enemy";
}

export function formatPhaseLabel(phase: BattlePhase): string {
  switch (phase) {
    case "startOfTurn":
      return "Start Of Turn";
    case "judgment":
      return "Judgment";
    case "draw":
      return "Draw";
    case "main":
      return "Main";
    case "endOfTurn":
      return "End Of Turn";
  }
}

export function formatResultLabel(result: BattleResult | null): string {
  if (result === null) {
    return "In Progress";
  }
  switch (result) {
    case "victory":
      return "Victory";
    case "defeat":
      return "Defeat";
    case "draw":
      return "Draw";
  }
}

export function formatZoneLabel(zone: BattleZoneId): string {
  switch (zone) {
    case "deck":
      return "Deck";
    case "hand":
      return "Hand";
    case "void":
      return "Void";
    case "banished":
      return "Banished";
    case "reserve":
      return "Reserve";
    case "deployed":
      return "Deployed";
  }
}

export function formatBrowseableZoneLabel(zone: BrowseableZone): string {
  return formatZoneLabel(zone);
}
