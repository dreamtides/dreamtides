#nullable enable

using System;
using DG.Tweening;
using UnityEngine;

namespace Dreamcaller.Utils
{
  public static class TweenUtils
  {
    public const float GlobalAnimationMultiplier = 1.0f;
    public const float MoveAnimationDurationSeconds = 0.3f * GlobalAnimationMultiplier;
    public const float FlipAnimationDurationSeconds = 0.4f * GlobalAnimationMultiplier;

    public static Sequence Sequence(string name)
    {
      var result = DOTween.Sequence();
      result.stringId = name;
      return result;
    }

    public static Sequence FadeIn(SpriteRenderer spriteRenderer)
    {
      spriteRenderer.color = new Color(spriteRenderer.color.r, spriteRenderer.color.g, spriteRenderer.color.b, 0);
      var result = Sequence($"FadeIn{spriteRenderer.gameObject.name}");
      result.Insert(0, spriteRenderer.DOFade(1, MoveAnimationDurationSeconds));
      return result;
    }

    public static Sequence FadeOut(SpriteRenderer spriteRenderer)
    {
      spriteRenderer.color = new Color(spriteRenderer.color.r, spriteRenderer.color.g, spriteRenderer.color.b, 1);
      var result = Sequence($"FadeOut{spriteRenderer.gameObject.name}");
      result.Insert(0, spriteRenderer.DOFade(0, MoveAnimationDurationSeconds));
      return result;
    }

    public static void ExecuteAfter(float seconds, Action action)
    {
      Sequence("ExecuteAfter").InsertCallback(seconds, () => action());
    }
  }
}