#nullable enable

using System.Collections;
using DG.Tweening;
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

    public IEnumerator HandleDisplayJudgmentCommand(DisplayJudgmentCommand displayJudgment)
    {
      var sequence = TweenUtils.Sequence("DisplayJudgment");
      var actorSparkTotal = displayJudgment.Player switch
      {
        PlayerName.User => Registry.Layout.UserStatusDisplay.TotalSpark.transform,
        PlayerName.Enemy => Registry.Layout.EnemyStatusDisplay.TotalSpark.transform,
        _ => throw Errors.UnknownEnumValue(displayJudgment.Player)
      };
      var opponentSparkTotal = displayJudgment.Player switch
      {
        PlayerName.User => Registry.Layout.EnemyStatusDisplay.TotalSpark.transform,
        PlayerName.Enemy => Registry.Layout.UserStatusDisplay.TotalSpark.transform,
        _ => throw Errors.UnknownEnumValue(displayJudgment.Player)
      };
      Vector3 actorOriginalPos = actorSparkTotal.position;
      Vector3 opponentOriginalPos = opponentSparkTotal.position;

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
        actorUpPosition.z - 1f
      ), _durationMultiplier * 0.1f));

      // Rapidly move back towards opponent
      sequence.Append(actorSparkTotal.DOMove(new Vector3(
        actorUpPosition.x,
        actorUpPosition.y,
        actorUpPosition.z + 1f
      ), _durationMultiplier * 0.05f));

      // Add shake effect after collision
      var shakeActor = displayJudgment.NewScore == null ? actorSparkTotal : opponentSparkTotal;
      sequence.Append(shakeActor.DOShakePosition(_durationMultiplier * 0.1f, _shakeStrength, _shakeVibrato));

      // Animate both objects returning to their original positions
      sequence.Append(actorSparkTotal.DOMove(actorOriginalPos, _durationMultiplier * 0.1f));
      sequence.Join(opponentSparkTotal.DOMove(opponentOriginalPos, _durationMultiplier * 0.1f));

      yield return sequence.WaitForCompletion();
    }
  }
}
