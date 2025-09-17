#nullable enable

namespace Dreamtides.Schema
{
  public partial class CardView
  {
    public string ClientId() => Id.ToString();
  }

  public partial class Milliseconds
  {
    public float ToSeconds() => MillisecondsValue / 1000f;
  }

  public partial struct ActionUnion
  {
    public GameAction? ToGameAction()
    {
      if (IsNull)
      {
        return null;
      }

      return new GameAction
      {
        Enum = Enum,
        GameActionClass = new()
        {
          DebugAction = ActionClass?.DebugAction,
          BattleAction = ActionClass?.BattleAction,
          BattleDisplayAction = ActionClass?.BattleDisplayAction,
          Undo = ActionClass?.Undo,
        },
      };
    }
  }
}
