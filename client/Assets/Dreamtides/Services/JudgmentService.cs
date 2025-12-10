#nullable enable

using System.Collections;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Components;
using Dreamtides.Schema;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Services
{
  public class JudgmentService : Service
  {
    [SerializeField]
    internal float _durationMultiplier = 1f;

    [SerializeField]
    internal float _shakeStrength = 0.5f;

    [SerializeField]
    internal int _shakeVibrato = 10;

    [SerializeField]
    internal AudioClip _startSound = null!;

    [SerializeField]
    internal AudioClip _windUpSound = null!;

    [SerializeField]
    internal AudioClip _pointsSound = null!;

    [SerializeField]
    internal TimedEffect _hitEffectPrefab = null!;

    [SerializeField]
    internal Projectile _scorePointsProjectilePrefab = null!;

    public IEnumerator HandleDisplayJudgmentCommand(DisplayJudgmentCommand displayJudgment)
    {
      var sequence = TweenUtils.Sequence("DisplayJudgment");
      var actorIsUser = displayJudgment.Player == DisplayPlayer.User;
      if (displayJudgment.NewScore == null)
      {
        // I used to do an animation when no points were scored, but it's
        // pretty annoying.
        yield break;
      }
      var actorStatusDisplay = actorIsUser
        ? Registry.BattleLayout.UserStatusDisplay
        : Registry.BattleLayout.EnemyStatusDisplay;
      var actorSparkTotal = actorStatusDisplay.TotalSpark.transform;
      var opponentSparkTotal = actorIsUser
        ? Registry.BattleLayout.EnemyStatusDisplay.TotalSpark.transform
        : Registry.BattleLayout.UserStatusDisplay.TotalSpark.transform;
      Vector3 actorOriginalPos = actorSparkTotal.position;
      Vector3 opponentOriginalPos = opponentSparkTotal.position;

      Registry.SoundService.Play(_startSound);
      var actorUpPosition = new Vector3(
        actorOriginalPos.x + 1f,
        actorOriginalPos.y + 3,
        actorOriginalPos.z
      );
      var opponentUpPosition = new Vector3(
        opponentOriginalPos.x + 1f,
        opponentOriginalPos.y + 3,
        opponentOriginalPos.z
      );
      sequence.Append(actorSparkTotal.DOMove(actorUpPosition, _durationMultiplier * 0.1f));
      sequence.Join(opponentSparkTotal.DOMove(opponentUpPosition, _durationMultiplier * 0.1f));

      // Calculate movement direction based on orientation and player
      float moveDirection = actorIsUser ? -1f : 1f;
      Vector3 moveAwayPosition = actorUpPosition;
      Vector3 moveBackPosition = actorUpPosition;

      moveAwayPosition.z += moveDirection;
      moveBackPosition.z -= moveDirection; // Opposite direction for moving back

      // Move away from opponent
      sequence.Append(actorSparkTotal.DOMove(moveAwayPosition, _durationMultiplier * 0.1f));
      sequence.AppendCallback(() => Registry.SoundService.Play(_windUpSound));

      // Rapidly move back towards opponent
      sequence.Append(actorSparkTotal.DOMove(moveBackPosition, _durationMultiplier * 0.05f));
      sequence.AppendCallback(() =>
      {
        var hit = Registry.AssetPoolService.Create(_hitEffectPrefab, actorSparkTotal.position);
        var rotation = Quaternion.LookRotation(
          transform.position - Registry.MainCamera.transform.position
        );
        hit.transform.rotation = rotation;
        hit.transform.localScale = 5f * Vector3.one;
        Registry.SoundService.Play(_pointsSound);
        var projectile = Registry.AssetPoolService.Create(
          _scorePointsProjectilePrefab,
          actorSparkTotal.position
        );
        StartCoroutine(
          projectile.Fire(
            Registry,
            actorStatusDisplay.transform,
            new Milliseconds { MillisecondsValue = 500 },
            onHit: () =>
            {
              actorStatusDisplay.SetScore(displayJudgment.NewScore!.Value, true);
            }
          )
        );
      });

      // Add shake effect after collision
      var shakeActor = opponentSparkTotal;
      sequence.Append(
        shakeActor.DOShakePosition(_durationMultiplier * 0.1f, _shakeStrength, _shakeVibrato)
      );

      // Animate both objects returning to their original positions
      sequence.Append(actorSparkTotal.DOMove(actorOriginalPos, _durationMultiplier * 0.1f));
      sequence.Join(opponentSparkTotal.DOMove(opponentOriginalPos, _durationMultiplier * 0.1f));

      yield return sequence.WaitForCompletion();
      yield return new WaitForSeconds(1f);
    }
  }
}
