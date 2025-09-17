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
            return "DebugAction";
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

                if (actionClass.SelectOrderForDeckCard != null)
                {
                    return "DeckCardSelectedOrder";
                }

                if (actionClass.Debug != null)
                {
                    return "DebugBattleAction";
                }
            }

            return "UnknownBattleAction";
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