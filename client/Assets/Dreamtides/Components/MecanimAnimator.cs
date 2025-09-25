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
      if (_currentPrimaryAnimation != null && _currentExitAnimation != null)
      {
        _animator.CrossFadeInFixedTime(_currentExitAnimation.Name, DefaultBlendDuration);
        // Use clip length directly because state info may still reference the previous state for a frame or two.
        var exitAnimationLength = GetClipLength(_currentExitAnimation.Name);
        // Wait the full clip length (the cross-fade itself overlaps with prior state).
        yield return new WaitForSeconds(Mathf.Max(0f, exitAnimationLength));
      }

      if (command.EnterAnimation != null)
      {
        _animator.CrossFadeInFixedTime(command.EnterAnimation.Name, DefaultBlendDuration);
        var enterAnimationLength = GetClipLength(command.EnterAnimation.Name);
        // We want to start the looping animation right as (or slightly before) the enter finishes.
        // Subtract the blend duration so the tail of enter blends into loop start.
        var wait = Mathf.Max(0f, enterAnimationLength - DefaultBlendDuration);
        if (wait > 0f)
        {
          yield return new WaitForSeconds(wait);
        }
      }

      _animator.CrossFadeInFixedTime(command.Animation.Name, DefaultBlendDuration);

      _currentPrimaryAnimation = command.Animation.Name;
      _currentExitAnimation = command.ExitAnimation;

      // If we have an ExitAfterLoops directive, count completed loops and then play exit + idle.
      if (command.ExitAfterLoops.HasValue && command.ExitAfterLoops.Value > 0)
      {
        var targetLoops = command.ExitAfterLoops.Value;
        var stateInfo = _animator.GetCurrentAnimatorStateInfo(0);
        long completedLoops = 0;
        int lastLoopIndex = Mathf.FloorToInt(stateInfo.normalizedTime);
        while (completedLoops < targetLoops)
        {
          yield return null;
          stateInfo = _animator.GetCurrentAnimatorStateInfo(0);
          int loopIndex = Mathf.FloorToInt(stateInfo.normalizedTime);
          if (loopIndex > lastLoopIndex)
          {
            completedLoops += loopIndex - lastLoopIndex;
            lastLoopIndex = loopIndex;
          }

          if (!stateInfo.loop && stateInfo.normalizedTime >= 1f)
          {
            break;
          }
        }

        if (command.ExitAnimation != null)
        {
          _animator.CrossFadeInFixedTime(command.ExitAnimation.Name, DefaultBlendDuration);
          var exitLength = GetClipLength(command.ExitAnimation.Name);
          yield return new WaitForSeconds(Mathf.Max(0f, exitLength));
        }

        // Blend smoothly back to idle.
        _animator.CrossFadeInFixedTime(IdleHash, DefaultBlendDuration);
        _currentPrimaryAnimation = null;
        _currentExitAnimation = null;
      }
    }

    float GetClipLength(string clipName)
    {
      if (_animator.runtimeAnimatorController == null)
      {
        return 0f;
      }
      foreach (var clip in _animator.runtimeAnimatorController.animationClips)
      {
        if (clip != null && clip.name == clipName)
        {
          return clip.length;
        }
      }
      Debug.LogWarning(
        $"MecanimAnimator: Could not find clip '{clipName}' in controller; defaulting length 0."
      );
      return 0f;
    }
  }
}
