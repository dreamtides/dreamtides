import type {
  BattleView,
  Command,
  CommandSequence,
  DisplayDreamwellActivationCommand,
} from "../types/battle";

export interface ParsedBattleCommands {
  battle: BattleView | null;
  dreamwellActivations: DisplayDreamwellActivationCommand[];
}

export function parseCommandSequence(commands: CommandSequence): ParsedBattleCommands {
  let lastBattle: BattleView | null = null;
  const dreamwellActivations: DisplayDreamwellActivationCommand[] = [];
  for (const group of commands.groups) {
    for (const cmd of group.commands) {
      if (isUpdateBattle(cmd)) {
        lastBattle = cmd.UpdateBattle.battle;
      }
      if (isDisplayDreamwellActivation(cmd)) {
        dreamwellActivations.push(cmd.DisplayDreamwellActivation);
      }
    }
  }
  return { battle: lastBattle, dreamwellActivations };
}

export function extractBattleView(commands: CommandSequence): BattleView | null {
  return parseCommandSequence(commands).battle;
}

function isUpdateBattle(
  cmd: Command,
): cmd is { UpdateBattle: { battle: BattleView; update_sound?: unknown } } {
  return "UpdateBattle" in cmd;
}

function isDisplayDreamwellActivation(
  cmd: Command,
): cmd is { DisplayDreamwellActivation: DisplayDreamwellActivationCommand } {
  return "DisplayDreamwellActivation" in cmd;
}
