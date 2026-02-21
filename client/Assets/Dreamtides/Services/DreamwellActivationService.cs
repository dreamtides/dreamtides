#nullable enable

using System.Collections;
using System.Runtime.CompilerServices;
using Abu;
using DG.Tweening;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class DreamwellActivationService : Service
  {
    [SerializeField]
    internal float _displayPauseDurationSeconds;

    [SerializeField]
    internal float _durationMultiplier = 1f;

    [SerializeField]
    internal AudioClip _startSound = null!;

    [SerializeField]
    internal AudioClip _revealSound = null!;

    public IEnumerator HandleDreamwellActivationCommand(DisplayDreamwellActivationCommand command)
    {
      using var busy = new BusyToken();

      // Get the card from the layout service
      var card = Registry.CardService.GetCard(command.CardId);

      // Turn the card face down without animation first
      card.TurnFaceDown();

      // Remove card from its parent if it has one
      if (card.Parent)
      {
        card.Parent.RemoveIfPresent(card);
      }

      if (_startSound != null)
      {
        Registry.SoundService.Play(_startSound);
      }

      var moveSequence = TweenUtils.Sequence("DreamwellActivationMove");
      var moveDuration = 0.3f * _durationMultiplier;

      moveSequence.Insert(
        0,
        card.transform.DOMove(Registry.BattleLayout.DreamwellDisplay.position, moveDuration)
          .SetEase(Ease.OutCubic)
      );
      moveSequence.Insert(
        0,
        card.transform.DORotate(
          Registry.BattleLayout.DreamwellDisplay.rotation.eulerAngles,
          moveDuration
        )
      );

      var flipDuration = TweenUtils.FlipAnimationDurationSeconds / 2f * _durationMultiplier;

      // Start the flip sequence when the move sequence is 75% complete
      float flipStartTime = moveDuration * 0.75f;

      // Wait for the move sequence to reach 75% completion
      yield return new WaitForSeconds(flipStartTime);

      var flipSequence = TweenUtils.Sequence("DreamwellActivationFlip");
      Registry.SoundService.Play(_revealSound);

      // Now start the flip sequence
      var targetRotation = Registry.BattleLayout.DreamwellDisplay.rotation;
      var flippedRotation = targetRotation * Quaternion.Euler(150, 0, 0);

      flipSequence
        .Insert(0, card.transform.DORotateQuaternion(flippedRotation, flipDuration))
        .InsertCallback(
          flipDuration,
          () =>
          {
            card.TurnFaceUp();
          }
        )
        .Insert(flipDuration, card.transform.DORotateQuaternion(targetRotation, flipDuration));

      // Wait for both sequences to complete
      yield return moveSequence.WaitForCompletion();
      yield return flipSequence.WaitForCompletion();

      yield return new WaitForSeconds(_displayPauseDurationSeconds * _durationMultiplier);
    }
  }
}
