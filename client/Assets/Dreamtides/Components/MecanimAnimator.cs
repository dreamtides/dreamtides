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

    string? _currentPrimaryAnimation;
    StudioAnimation? _currentExitAnimation;

    public IEnumerator PlayAnimation(PlayMecanimAnimationCommand command)
    {
      if (_currentPrimaryAnimation != null && _currentExitAnimation != null)
      {
        _animator.Play(_currentExitAnimation.Name);
        yield return new WaitForEndOfFrame();
        var exitAnimationLength = _animator.GetCurrentAnimatorStateInfo(0).length;
        yield return new WaitForSeconds(exitAnimationLength);
      }

      if (command.EnterAnimation != null)
      {
        _animator.Play(command.EnterAnimation.Name);
        yield return new WaitForEndOfFrame();
        var enterAnimationLength = _animator.GetCurrentAnimatorStateInfo(0).length;
        yield return new WaitForSeconds(enterAnimationLength);
      }

      _animator.Play(command.Animation.Name);
      _currentPrimaryAnimation = command.Animation.Name;
      _currentExitAnimation = command.ExitAnimation;
    }
  }
}
