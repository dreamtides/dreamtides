#nullable enable

using System.Collections;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using UnityEngine;

namespace Dreamtides.Animations
{
  public static class CardAnimationUtils
  {
    public static IEnumerator MoveCardToPosition(
      Card card,
      Vector3 position,
      Quaternion rotation,
      float duration
    )
    {
      var seq = DOTween.Sequence();
      seq.Insert(0, card.transform.DOMove(position, duration).SetEase(Ease.OutCubic));
      seq.Insert(0, card.transform.DORotateQuaternion(rotation, duration).SetEase(Ease.OutCubic));
      yield return seq.WaitForCompletion();
    }

    public static IEnumerator ShuffleDeckRotation(ObjectLayout deckLayout, float totalDuration)
    {
      var cards = deckLayout.Objects.OfType<Card>().ToList();
      if (cards.Count == 0 || totalDuration <= 0f)
      {
        yield break;
      }

      float half = totalDuration / 2f;
      var seq = DOTween.Sequence();
      foreach (var card in cards)
      {
        var startEuler = card.transform.localEulerAngles;
        float angle = UnityEngine.Random.Range(-15f, 15f);
        var midEuler = startEuler + new Vector3(0f, angle, 0f);
        seq.Insert(0, card.transform.DOLocalRotate(midEuler, half * 0.9f).SetEase(Ease.OutCubic));
        seq.Insert(
          half,
          card.transform.DOLocalRotate(startEuler, half * 0.9f).SetEase(Ease.InCubic)
        );
      }
      yield return seq.WaitForCompletion();
    }
  }
}
