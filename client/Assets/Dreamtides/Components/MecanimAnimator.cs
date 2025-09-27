#nullable enable

using System.Collections;
using Dreamtides.Schema;
using UnityEngine;

namespace Dreamtides.Components
{
  public class MecanimAnimator : MonoBehaviour
  {
    [SerializeField]
    Animator _animator = null!;

    const float DefaultBlendDuration = 0.3f;

    static readonly int IdleHash = Animator.StringToHash("IDL_Base");

    string? _currentPrimaryAnimation;
    StudioAnimation? _currentExitAnimation;

    public IEnumerator PlayAnimation(PlayMecanimAnimationCommand command)
    {
      foreach (var param in command.Parameters)
      {
        if (param.TriggerParam != null)
        {
          _animator.SetTrigger(param.TriggerParam.Name);
        }

        if (param.BoolParam != null)
        {
          _animator.SetBool(param.BoolParam.Name, param.BoolParam.Value);
        }

        if (param.IntParam != null)
        {
          _animator.SetInteger(param.IntParam.Name, (int)param.IntParam.Value);
        }

        if (param.FloatParam != null)
        {
          _animator.SetFloat(param.FloatParam.Name, (float)param.FloatParam.Value);
        }
      }
      _animator.SetTrigger("WaveSmall");
      yield break;
    }
  }
}
