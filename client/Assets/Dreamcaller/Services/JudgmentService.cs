#nullable enable

using System.Collections;
using DG.Tweening;
using Dreamcaller.Components;
using Dreamcaller.Schema;
using Dreamcaller.Utils;
using Graphy.Runtime.UI;
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

    public IEnumerator HandleDisplayJudgmentCommand(DisplayJudgmentCommand displayJudgment)
    {
      var sequence = TweenUtils.Sequence("DisplayJudgment");
      var actorIsUser = displayJudgment.Player == PlayerName.User;
      var actorScoredPoints = displayJudgment.NewScore != null;
      var actorSparkTotal = actorIsUser ?
          Registry.Layout.UserStatusDisplay.TotalSpark.transform :
          Registry.Layout.EnemyStatusDisplay.TotalSpark.transform;
      var opponentSparkTotal = actorIsUser ?
          Registry.Layout.EnemyStatusDisplay.TotalSpark.transform :
          Registry.Layout.UserStatusDisplay.TotalSpark.transform;
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

      // Move away from opponent
      sequence.Append(actorSparkTotal.DOMove(new Vector3(
        actorUpPosition.x,
        actorUpPosition.y,
        actorUpPosition.z + (actorIsUser ? -1f : 1f)
      ), _durationMultiplier * 0.1f));
      sequence.AppendCallback(() => Registry.SoundService.Play(_windUpSound));

      // Rapidly move back towards opponent
      sequence.Append(actorSparkTotal.DOMove(new Vector3(
        actorUpPosition.x,
        actorUpPosition.y,
        actorUpPosition.z + (actorIsUser ? 1f : -1f)
      ), _durationMultiplier * 0.05f));
      sequence.AppendCallback(() =>
      {
        var hit = Registry.AssetPoolService.Create(_hitEffectPrefab, actorSparkTotal.position);
        var rotation = Quaternion.LookRotation(transform.position - Registry.Layout.MainCamera.transform.position);
        hit.transform.rotation = rotation;
        hit.transform.localScale = 5f * Vector3.one;
        Registry.SoundService.Play(actorScoredPoints ? _pointsSound : _noPointsSound);
      });

      // Add shake effect after collision
      var shakeActor = actorScoredPoints ? opponentSparkTotal : actorSparkTotal;
      sequence.Append(shakeActor.DOShakePosition(_durationMultiplier * 0.1f, _shakeStrength, _shakeVibrato));

      // Animate both objects returning to their original positions
      sequence.Append(actorSparkTotal.DOMove(actorOriginalPos, _durationMultiplier * 0.1f));
      sequence.Join(opponentSparkTotal.DOMove(opponentOriginalPos, _durationMultiplier * 0.1f));
      sequence.AppendCallback(() =>
      {
        Registry.SoundService.Play(_startSound);
      });

      yield return sequence.WaitForCompletion();
    }
  }
}
