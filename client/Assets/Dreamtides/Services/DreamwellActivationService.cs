#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Services
{
  public class DreamwellActivationService : Service
  {
    [SerializeField] float _displayPauseDurationSeconds;
    [SerializeField] float _durationMultiplier = 1f;
    [SerializeField] Projectile _updateEnergyProjectilePrefab = null!;
    [SerializeField] AudioClip _startSound = null!;
    [SerializeField] AudioClip _revealSound = null!;
    [SerializeField] AudioClip _fireSound = null!;
    [SerializeField] AudioClip _energyUpdateSound = null!;

    public IEnumerator HandleDreamwellActivationCommand(DisplayDreamwellActivationCommand command)
    {
      // Get the card from the layout service
      var card = Registry.LayoutService.GetCard(command.CardId);

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
        card.transform.DOMove(Registry.Layout.DreamwellDisplay.position, moveDuration)
          .SetEase(Ease.OutCubic));
      moveSequence.Insert(
        0,
        card.transform.DORotate(Registry.Layout.DreamwellDisplay.rotation.eulerAngles, moveDuration));

      var flipDuration = TweenUtils.FlipAnimationDurationSeconds / 2f * _durationMultiplier;

      // Start the flip sequence when the move sequence is 75% complete
      float flipStartTime = moveDuration * 0.75f;

      // Wait for the move sequence to reach 75% completion
      yield return new WaitForSeconds(flipStartTime);

      var flipSequence = TweenUtils.Sequence("DreamwellActivationFlip");
      Registry.SoundService.Play(_revealSound);

      var flippedAngles = new Vector3(150, 0, 0);
      // Now start the flip sequence
      flipSequence
        .Insert(0, card.transform.DOLocalRotate(flippedAngles, flipDuration))
        .InsertCallback(flipDuration, () =>
        {
          card.TurnFaceUp();
        })
        .Insert(flipDuration, card.transform.DOLocalRotate(
            Registry.Layout.DreamwellDisplay.rotation.eulerAngles, flipDuration));

      // Wait for both sequences to complete
      yield return moveSequence.WaitForCompletion();
      yield return flipSequence.WaitForCompletion();

      yield return new WaitForSeconds(_displayPauseDurationSeconds * _durationMultiplier);
      /*
            // Fire energy update projectile if energy change is specified
            if (command.NewProducedEnergy.HasValue && command.NewEnergy.HasValue)
            {
              var projectile = Registry.AssetPoolService.Create(
                _updateEnergyProjectilePrefab,
                card.transform.position
              );

              if (_fireSound != null)
              {
                Registry.SoundService.Play(_fireSound);
              }

              // Create a throw animation sequence
              var startPosition = card.transform.position;
              var statusDisplay = command.Player switch
              {
                DisplayPlayer.Enemy => Registry.Layout.EnemyStatusDisplay,
                DisplayPlayer.User => Registry.Layout.UserStatusDisplay,
                _ => throw Errors.UnknownEnumValue(command.Player)
              };
              var targetPosition = statusDisplay.transform.position;
              var throwSequence = TweenUtils.Sequence("DreamwellThrow")
                .Insert(0, card.transform.DOMove(Vector3.Lerp(startPosition, targetPosition, 0.1f), 0.1f))
                .Insert(0.1f, card.transform.DOMove(startPosition, 0.1f));

              StartCoroutine(projectile.Fire(
                Registry,
                statusDisplay.transform,
                new Milliseconds { MillisecondsValue = 500 },
                onHit: () =>
                {
                  Registry.SoundService.Play(_energyUpdateSound);
                  statusDisplay.SetEnergy(command.NewEnergy.Value, command.NewProducedEnergy.Value, true);
                },
                mute: true
              ));

              // Wait for the throw animation to complete
              yield return throwSequence.WaitForCompletion();
            }

            yield return new WaitForSeconds(0.3f * _durationMultiplier);
            var finalMoveSequence = TweenUtils.Sequence("DreamwellActivationFinalMove");
            finalMoveSequence.Insert(
                0,
                card.transform.DOMove(Registry.Layout.DreamwellActivation.transform.position, moveDuration)
                  .SetEase(Ease.OutCubic));
            finalMoveSequence.Insert(
              0,
              card.transform.DORotate(Registry.Layout.DreamwellActivation.transform.rotation.eulerAngles, moveDuration));

            yield return finalMoveSequence.WaitForCompletion();
            */
    }
  }
}