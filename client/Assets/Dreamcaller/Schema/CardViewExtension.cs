#nullable enable

namespace Dreamcaller.Schema
{
  public partial class CardView
  {
    public string ClientId() => $"{Id.Idx}-{Id.Version}";
  }
}