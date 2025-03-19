#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamcaller.Components;
using Dreamcaller.Schema;
using Dreamcaller.Utils;

using UnityEngine;

namespace Dreamcaller.Services
{
  public class JudgmentService : Service
  {
    [SerializeField] float _durationMultiplier = 1f;
    [SerializeField] float _shakeStrength = 0.5f;
    [SerializeField] int _shakeVibrato = 10;
    [SerializeField] AudioClip _startSound = null!;
    [SerializeField] AudioClip _windUpSound = null!;
    [SerializeField] AudioClip _noPointsSound = null!;
    [SerializeField] AudioClip _pointsSound = null!;
    [SerializeField] TimedEffect _hitEffectPrefab = null!;
    [SerializeField] Projectile _scorePointsProjectilePrefab = null!;

    public IEnumerator HandleDisplayJudgmentCommand(DisplayJudgmentCommand displayJudgment)
    {
      var isLandscape = Registry.IsLandscape;
      var sequence = TweenUtils.Sequence("DisplayJudgment");
      var actorIsUser = displayJudgment.Player == PlayerName.User;
      var actorScoredPoints = displayJudgment.NewScore != null;
      var actorStatusDisplay = actorIsUser ?
          Registry.Layout.UserStatusDisplay :
          Registry.Layout.EnemyStatusDisplay;
      var actorSparkTotal = actorStatusDisplay.TotalSpark.transform;
      var opponentSparkTotal = actorIsUser ?
          Registry.Layout.EnemyStatusDisplay.TotalSpark.transform :
          Registry.Layout.UserStatusDisplay.TotalSpark.transform;
      Vector3 actorOriginalPos = actorSparkTotal.position;
      Vector3 opponentOriginalPos = opponentSparkTotal.position;

      Registry.SoundService.Play(_startSound);
      var actorUpPosition = new Vector3(
        actorOriginalPos.x + (isLandscape ? 0 : 1f),
        actorOriginalPos.y + 3,
        actorOriginalPos.z - (isLandscape ? 2f : 0)
      );
      var opponentUpPosition = new Vector3(
        opponentOriginalPos.x + (isLandscape ? 0 : 1f),
        opponentOriginalPos.y + 3,
        opponentOriginalPos.z - (isLandscape ? 2f : 0)
      );
      sequence.Append(actorSparkTotal.DOMove(actorUpPosition, _durationMultiplier * 0.1f));
      sequence.Join(opponentSparkTotal.DOMove(opponentUpPosition, _durationMultiplier * 0.1f));

      // Calculate movement direction based on orientation and player
      float moveDirection = actorIsUser ? -1f : 1f;
      Vector3 moveAwayPosition = actorUpPosition;
      Vector3 moveBackPosition = actorUpPosition;

      if (isLandscape)
      {
        // In landscape, move along X axis
        moveAwayPosition.x += moveDirection;
        moveBackPosition.x -= moveDirection; // Opposite direction for moving back
      }
      else
      {
        // In portrait, move along Z axis
        moveAwayPosition.z += moveDirection;
        moveBackPosition.z -= moveDirection; // Opposite direction for moving back
      }

      // Move away from opponent
      sequence.Append(actorSparkTotal.DOMove(moveAwayPosition, _durationMultiplier * 0.1f));
      sequence.AppendCallback(() => Registry.SoundService.Play(_windUpSound));

      // Rapidly move back towards opponent
      sequence.Append(actorSparkTotal.DOMove(moveBackPosition, _durationMultiplier * 0.05f));
      sequence.AppendCallback(() =>
      {
        var hit = Registry.AssetPoolService.Create(_hitEffectPrefab, actorSparkTotal.position);
        var rotation = Quaternion.LookRotation(transform.position - Registry.Layout.MainCamera.transform.position);
        hit.transform.rotation = rotation;
        hit.transform.localScale = 5f * Vector3.one;
        Registry.SoundService.Play(actorScoredPoints ? _pointsSound : _noPointsSound);
        if (actorScoredPoints)
        {
          var projectile = Registry.AssetPoolService.Create(_scorePointsProjectilePrefab, actorSparkTotal.position);
          StartCoroutine(projectile.Fire(Registry, actorStatusDisplay.transform, new Milliseconds
          {
            MillisecondsValue = 500,
          }, onHit: () =>
          {
            actorStatusDisplay.Score.SetNumber(displayJudgment.NewScore!.Value, true);
          }));
        }
      });

      // Add shake effect after collision
      var shakeActor = actorScoredPoints ? opponentSparkTotal : actorSparkTotal;
      sequence.Append(shakeActor.DOShakePosition(_durationMultiplier * 0.1f, _shakeStrength, _shakeVibrato));

      // Animate both objects returning to their original positions
      sequence.Append(actorSparkTotal.DOMove(actorOriginalPos, _durationMultiplier * 0.1f));
      sequence.Join(opponentSparkTotal.DOMove(opponentOriginalPos, _durationMultiplier * 0.1f));

      if (!actorScoredPoints)
      {
        sequence.AppendCallback(() =>
        {
          Registry.SoundService.Play(_startSound);
        });
      }

      yield return sequence.WaitForCompletion();
    }
  }
}
