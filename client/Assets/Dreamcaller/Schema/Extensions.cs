#nullable enable

namespace Dreamcaller.Schema
{
  public partial class CardView
  {
    public string ClientId() => $"{Id.Idx}-{Id.Version}";
  }

  public partial class Milliseconds
  {
    public float ToSeconds() => MillisecondsValue / 1000f;
  }
}