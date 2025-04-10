#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using UnityEngine;

namespace Dreamtides.Utils
{
  public static class TweenUtils
  {
    public const float GlobalAnimationMultiplier = 1.0f;
    public const float MoveAnimationDurationSeconds = 0.3f * GlobalAnimationMultiplier;
    public const float FlipAnimationDurationSeconds = 0.4f * GlobalAnimationMultiplier;

    public static bool TestMode { get; set; }
    static List<Sequence> _activeSequences { get; } = new List<Sequence>();

    public static Sequence Sequence(string name)
    {
      var result = DOTween.Sequence();
      result.stringId = name;
      if (TestMode)
      {
        _activeSequences.Add(result);
        result.OnComplete(() =>
        {
          _activeSequences.Remove(result);
        });
      }
      return result;
    }

    public static IEnumerator WaitForActiveSequences()
    {
      var sequences = new List<Sequence>(_activeSequences);
      foreach (var sequence in sequences)
      {
        if (sequence.IsActive())
        {
          yield return sequence.WaitForCompletion();
        }
      }
    }

    public static Sequence FadeInCanvasGroup(CanvasGroup canvasGroup)
    {
      canvasGroup.alpha = 0;
      var result = Sequence($"FadeIn{canvasGroup.gameObject.name}");
      result.Insert(0, canvasGroup.DOFade(1, MoveAnimationDurationSeconds));
      return result;
    }

    public static Sequence FadeOutCanvasGroup(CanvasGroup canvasGroup)
    {
      canvasGroup.alpha = 1;
      var result = Sequence($"FadeOut{canvasGroup.gameObject.name}");
      result.Insert(0, canvasGroup.DOFade(0, MoveAnimationDurationSeconds));
      return result;
    }

    public static Sequence FadeInSprite(SpriteRenderer spriteRenderer)
    {
      spriteRenderer.color = new Color(spriteRenderer.color.r, spriteRenderer.color.g, spriteRenderer.color.b, 0);
      var result = Sequence($"FadeIn{spriteRenderer.gameObject.name}");
      result.Insert(0, spriteRenderer.DOFade(1, MoveAnimationDurationSeconds));
      return result;
    }

    public static Sequence FadeOutSprite(SpriteRenderer spriteRenderer)
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