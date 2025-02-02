#nullable enable

using DG.Tweening;

namespace Dreamcaller.Utils
{
  public static class TweenUtils
  {
    public const float GlobalAnimationMultiplier = 10.0f;
    public const float MoveAnimationDurationSeconds = 0.3f * GlobalAnimationMultiplier;
    public const float FlipAnimationDurationSeconds = 0.4f * GlobalAnimationMultiplier;

    public static Sequence Sequence(string name)
    {
      var result = DOTween.Sequence();
      result.stringId = name;
      return result;
    }

  }
}