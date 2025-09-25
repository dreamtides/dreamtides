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
        string priorExitState = _currentExitAnimation.Name;
        _animator.CrossFadeInFixedTime(priorExitState, DefaultBlendDuration);
        yield return WaitForStateToBecomeActive(priorExitState, 0.5f);
        var exitInfo = GetStateInfoIfActive(priorExitState);
        if (exitInfo.HasValue)
        {
          // We already blended into this state; subtract blend so timing overlaps.
          var wait = Mathf.Max(0f, exitInfo.Value.length - DefaultBlendDuration);
          if (wait > 0f)
          {
            yield return new WaitForSeconds(wait);
          }
        }
      }

      if (command.EnterAnimation != null)
      {
        string enterState = command.EnterAnimation.Name;
        _animator.CrossFadeInFixedTime(enterState, DefaultBlendDuration);
        yield return WaitForStateToBecomeActive(enterState, 0.5f);
        var enterInfo = GetStateInfoIfActive(enterState);
        if (enterInfo.HasValue)
        {
          var wait = Mathf.Max(0f, enterInfo.Value.length - DefaultBlendDuration);
          if (wait > 0f)
          {
            yield return new WaitForSeconds(wait);
          }
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
          string exitState = command.ExitAnimation.Name;
          _animator.CrossFadeInFixedTime(exitState, DefaultBlendDuration);
          yield return WaitForStateToBecomeActive(exitState, 0.5f);
          var eInfo = GetStateInfoIfActive(exitState);
          if (eInfo.HasValue)
          {
            var wait = Mathf.Max(0f, eInfo.Value.length - DefaultBlendDuration);
            if (wait > 0f)
            {
              yield return new WaitForSeconds(wait);
            }
          }
        }

        // Blend smoothly back to idle.
        _animator.CrossFadeInFixedTime(IdleHash, DefaultBlendDuration);
        _currentPrimaryAnimation = null;
        _currentExitAnimation = null;
      }
    }

    // Wait until the specified state is either the current or the next state.
    IEnumerator WaitForStateToBecomeActive(string stateName, float timeoutSeconds)
    {
      int targetShortHash = Animator.StringToHash(stateName);
      float elapsed = 0f;
      while (elapsed < timeoutSeconds)
      {
        var current = _animator.GetCurrentAnimatorStateInfo(0);
        if (current.shortNameHash == targetShortHash || current.IsName(stateName))
        {
          yield break;
        }
        var next = _animator.GetNextAnimatorStateInfo(0);
        if (next.shortNameHash == targetShortHash || next.IsName(stateName))
        {
          yield break;
        }
        elapsed += Time.deltaTime;
        yield return null;
      }
      Debug.LogWarning($"MecanimAnimator: Timed out waiting for state '{stateName}' to activate.");
    }

    AnimatorStateInfo? GetStateInfoIfActive(string stateName)
    {
      int targetShortHash = Animator.StringToHash(stateName);
      var current = _animator.GetCurrentAnimatorStateInfo(0);
      if (current.shortNameHash == targetShortHash || current.IsName(stateName))
      {
        return current;
      }
      var next = _animator.GetNextAnimatorStateInfo(0);
      if (next.shortNameHash == targetShortHash || next.IsName(stateName))
      {
        return next;
      }
      return null;
    }
  }
}
