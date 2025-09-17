#nullable enable

using System.Collections;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Animations
{
  public class ShuffleVoidIntoDeckAnimation
  {
    public IEnumerator Handle(ShuffleVoidIntoDeckCommand command, CardAnimationService service)
    {
      var (source, destination) = command.Player switch
      {
        DisplayPlayer.User => (service.Registry.Layout.UserVoid, service.Registry.Layout.UserDeck),
        DisplayPlayer.Enemy => (service.Registry.Layout.EnemyVoid, service.Registry.Layout.EnemyDeck),
        _ => (null, null)
      };

      if (source == null || destination == null)
      {
        yield break;
      }

      var sourceCards = source.Objects.OfType<Card>().ToList();
      if (sourceCards.Count == 0)
      {
        yield break;
      }

      const float totalDuration = 3f;
      const float earlyPhaseDuration = 1f; // First second: move only up to 10 cards
      float shuffleRotationDuration = 0.4f; // Wiggle at end (can be trimmed if overcrowded)
      float movePhaseDuration = Mathf.Max(0f, totalDuration - shuffleRotationDuration);

      int earlyBatchCount = Mathf.Min(10, sourceCards.Count);
      int laterBatchCount = sourceCards.Count - earlyBatchCount;

      // If there are too many cards, ensure later cards still have some minimum time slice.
      const float minLegDuration = 0.03f; // per leg minimal duration target
      const float earlyPauseFraction = 0.35f; // portion of early slice used for pause (tapered)

      // Compute base per-card durations for phases.
      float earlyPerCardTotal = earlyBatchCount > 0 ? earlyPhaseDuration / earlyBatchCount : 0f;
      float laterPhaseDuration = movePhaseDuration - earlyPhaseDuration;
      if (laterPhaseDuration < 0f) laterPhaseDuration = 0f;
      float laterPerCardTotal = laterBatchCount > 0 ? laterPhaseDuration / laterBatchCount : 0f;

      // Adjust if later cards get too little time; borrow from shuffle wiggle if necessary.
      if (laterBatchCount > 0 && laterPerCardTotal / 2f < minLegDuration)
      {
        // Required later duration to hit minLegDuration with no pause.
        float requiredLaterDuration = laterBatchCount * (minLegDuration * 2f);
        float deficit = requiredLaterDuration - laterPhaseDuration;
        if (deficit > 0f)
        {
          // First try trimming shuffle time.
          float availableFromShuffle = Mathf.Max(0f, shuffleRotationDuration - 0.15f); // keep at least 0.15s
          float take = Mathf.Min(deficit, availableFromShuffle);
          shuffleRotationDuration -= take;
          laterPhaseDuration += take;
          laterPerCardTotal = laterPhaseDuration / laterBatchCount;
        }
      }

      for (int i = 0; i < sourceCards.Count; ++i)
      {
        var card = sourceCards[i];
        source.RemoveIfPresent(card);

        bool isEarly = i < earlyBatchCount;
        float perCardTotal = isEarly ? earlyPerCardTotal : laterPerCardTotal;

        // Early cards get a pause that tapers off over the first 10. Later cards have no pause.
        float pauseTaper = 0f;
        if (isEarly && earlyBatchCount > 1)
        {
          pauseTaper = 1f - (i / (float)(earlyBatchCount - 1)); // 1 for first early, 0 for last early
        }
        else if (isEarly && earlyBatchCount == 1)
        {
          pauseTaper = 1f;
        }
        float pauseDuration = isEarly ? perCardTotal * earlyPauseFraction * pauseTaper : 0f;
        float legsAvailable = perCardTotal - pauseDuration;
        if (legsAvailable < minLegDuration * 2f)
        {
          pauseDuration = Mathf.Max(0f, perCardTotal - (minLegDuration * 2f));
          legsAvailable = perCardTotal - pauseDuration;
        }
        float legDuration = legsAvailable / 2f;
        legDuration = Mathf.Max(minLegDuration, legDuration);

        service.Registry.SoundService.PlayDrawCardSound();

        // 1) Move to drawn position.
        yield return CardAnimationUtils.MoveCardToPosition(card,
          service.Registry.Layout.DrawnCardsPosition.transform.position,
          service.Registry.Layout.DrawnCardsPosition.transform.rotation,
          legDuration);

        card.GameContext = GameContext.DrawnCards;

        if (pauseDuration > 0.001f)
        {
          yield return new UnityEngine.WaitForSeconds(pauseDuration);
        }

        // 2) Move into deck root.
        yield return CardAnimationUtils.MoveCardToPosition(card,
          destination.transform.position,
          destination.transform.rotation,
          legDuration);

        destination.Add(card);
        card.transform.position = destination.transform.position;
        card.transform.rotation = destination.transform.rotation;
      }

      destination.ApplyLayout();

      // 3) Shuffle rotation effect (wiggle cards around Y axis within remaining time budget)
      yield return CardAnimationUtils.ShuffleDeckRotation(destination, shuffleRotationDuration);
    }
  }
}