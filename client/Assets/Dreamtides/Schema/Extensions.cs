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

  /// <summary>
  /// QuickType randomly changes the name of this struct whenver it feels like
  /// it. Often you can fix it by introducing a wrapper struct.
  /// </summary>
  public partial struct OnClickUnion
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
          DebugAction = OnClickClass?.DebugAction,
          BattleAction = OnClickClass?.BattleAction,
          BattleDisplayAction = OnClickClass?.BattleDisplayAction,
          Undo = OnClickClass?.Undo,
        },
      };
    }
  }
}
