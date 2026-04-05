import type { BattleView, Command, CommandSequence } from "../types/battle";

export function extractBattleView(commands: CommandSequence): BattleView | null {
  let lastBattle: BattleView | null = null;
  for (const group of commands.groups) {
    for (const cmd of group.commands) {
      if (isUpdateBattle(cmd)) {
        lastBattle = cmd.UpdateBattle.battle;
      }
    }
  }
  return lastBattle;
}

function isUpdateBattle(
  cmd: Command,
): cmd is { UpdateBattle: { battle: BattleView; update_sound?: unknown } } {
  return "UpdateBattle" in cmd;
}
