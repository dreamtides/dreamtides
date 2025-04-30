#nullable enable

namespace Dreamtides.Schema
{
  public partial class CardView
  {
    public string ClientId() => Id.ClientId();
  }

  public partial class CardId
  {
    public string ClientId() => $"{Idx}-{Version}";
  }

  public partial class Milliseconds
  {
    public float ToSeconds() => MillisecondsValue / 1000f;
  }

  public partial struct ActionUnion
  {
    public GameAction ToGameAction() => new GameAction
    {
      Enum = Enum,
      GameActionClass = new()
      {
        DebugAction = ActionClass.DebugAction,
        BattleAction = ActionClass.BattleAction,
        OpenPanel = ActionClass.OpenPanel,
      }
    };
  }
}
