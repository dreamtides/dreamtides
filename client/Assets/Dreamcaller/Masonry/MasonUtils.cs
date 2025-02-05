#nullable enable

using Dreamcaller.Schema;

namespace Dreamcaller.Masonry
{
  public static class MasonUtils
  {
    public static UserAction? ToUserAction(OnClickClass? onClick)
    {
      if (onClick?.BattleAction != null)
      {
        return new UserAction { BattleAction = onClick.BattleAction };
      }

      if (onClick?.DebugAction != null)
      {
        return new UserAction { DebugAction = onClick.DebugAction };
      }

      return null;
    }
  }
}
