#nullable enable

using Dreamtides.Schema;

namespace Dreamtides.Utils
{
    public static class GameActionHelper
    {
        public static string GetActionName(GameAction action)
        {
            if (action.Enum.HasValue)
            {
                return action.Enum.Value switch
                {
                    GameActionEnum.NoOp => "NoOp",
                    _ => action.Enum.Value.ToString()
                };
            }

            if (action.GameActionClass != null)
            {
                if (action.GameActionClass.DebugAction != null)
                {
                    return GetDebugActionName(action.GameActionClass.DebugAction.Value);
                }

                if (action.GameActionClass.BattleAction != null)
                {
                    return GetBattleActionName(action.GameActionClass.BattleAction.Value);
                }

                if (action.GameActionClass.BattleDisplayAction != null)
                {
                    return GetBattleDisplayActionName(action.GameActionClass.BattleDisplayAction.Value);
                }

                if (action.GameActionClass.Undo != null)
                {
                    return $"Undo{action.GameActionClass.Undo.Value}";
                }
            }

            return "Unknown";
        }

        private static string GetDebugActionName(DebugAction debugAction)
        {
            if (debugAction.Enum.HasValue)
            {
                return debugAction.Enum.Value switch
                {
                    DebugActionEnum.ApplyTestScenarioAction => "ApplyTestScenarioAction",
                    DebugActionEnum.RestartBattle => "RestartBattle",
                    _ => debugAction.Enum.Value.ToString()
                };
            }

            if (debugAction.DebugActionClass != null)
            {
                return "SetOpponentAgent";
            }

            return "UnknownDebugAction";
        }

        private static string GetBattleActionName(BattleAction battleAction)
        {
            if (battleAction.Enum.HasValue)
            {
                return battleAction.Enum.Value switch
                {
                    BattleActionEnum.EndTurn => "EndTurn",
                    BattleActionEnum.PassPriority => "PassPriority",
                    BattleActionEnum.StartNextTurn => "StartNextTurn",
                    BattleActionEnum.SubmitMulligan => "SubmitMulligan",
                    BattleActionEnum.ToggleOrderSelectorVisibility => "ToggleOrderSelectorVisibility",
                    _ => battleAction.Enum.Value.ToString()
                };
            }

            if (battleAction.BattleActionClass != null)
            {
                var actionClass = battleAction.BattleActionClass;

                if (actionClass.PlayCardFromHand.HasValue)
                {
                    return "PlayCardFromHand";
                }

                if (actionClass.SelectCharacterTarget.HasValue)
                {
                    return "SelectCharacterTarget";
                }

                if (actionClass.SelectStackCardTarget.HasValue)
                {
                    return "SelectStackCardTarget";
                }

                if (actionClass.SelectPromptChoice.HasValue)
                {
                    return "SelectPromptChoice";
                }

                if (actionClass.SelectEnergyAdditionalCost.HasValue)
                {
                    return "SelectEnergyAdditionalCost";
                }

                if (actionClass.SelectCardOrder != null)
                {
                    return "SelectCardOrder";
                }

                if (actionClass.Debug != null)
                {
                    return GetDebugBattleActionName(actionClass.Debug);
                }
            }

            return "UnknownBattleAction";
        }

        private static string GetDebugBattleActionName(DebugBattleAction debugBattleAction)
        {
            if (debugBattleAction.DrawCard.HasValue)
            {
                return "DrawCard";
            }

            if (debugBattleAction.SetEnergy != null && debugBattleAction.SetEnergy.Count > 0)
            {
                return "SetEnergy";
            }

            if (debugBattleAction.AddCardToHand != null && debugBattleAction.AddCardToHand.Count > 0)
            {
                return "AddCardToHand";
            }

            return "UnknownDebugBattleAction";
        }

        private static string GetBattleDisplayActionName(BattleDisplayAction battleDisplayAction)
        {
            if (battleDisplayAction.Enum.HasValue)
            {
                return battleDisplayAction.Enum.Value switch
                {
                    BattleDisplayActionEnum.CloseCardBrowser => "CloseCardBrowser",
                    BattleDisplayActionEnum.CloseCurrentPanel => "CloseCurrentPanel",
                    _ => battleDisplayAction.Enum.Value.ToString()
                };
            }

            if (battleDisplayAction.BattleDisplayActionClass != null)
            {
                var actionClass = battleDisplayAction.BattleDisplayActionClass;

                if (actionClass.BrowseCards.HasValue)
                {
                    return "BrowseCards";
                }

                if (actionClass.SetSelectedEnergyAdditionalCost.HasValue)
                {
                    return "SetSelectedEnergyAdditionalCost";
                }

                if (actionClass.OpenPanel.HasValue)
                {
                    return "OpenPanel";
                }
            }

            return "UnknownBattleDisplayAction";
        }
    }
}