#nullable enable

using UnityEngine;
using System.Collections;
using System.Collections.Generic;
using Dreamtides.Schema;
using Dreamtides.Services;

namespace Dreamtides.Components
{
  public class PlayerStatusAnimations : MonoBehaviour
  {
    [SerializeField] StudioType _studioType;
    [SerializeField] Registry _registry = null!;
    [SerializeField] float _lowerBoundSeconds;
    [SerializeField] float _upperBoundSeconds;
    [SerializeField] float _loopSeconds;

    StudioService? _studioService;
    Animator? _animator;
    Coroutine? _animationCycle;

    struct AnimationConfig
    {
      public string? EnterAnimation;
      public string MainAnimation;
      public string? ExitAnimation;
      public bool IsLooping;
    }

    readonly List<AnimationConfig> _animations = new()
    {
      new AnimationConfig { EnterAnimation = "IDL_ArmsFolded_Casual_Enter", MainAnimation = "IDL_ArmsFolded_Casual_Loop", ExitAnimation = "IDL_ArmsFolded_Casual_Exit", IsLooping = true },
      new AnimationConfig { MainAnimation = "IDL_Bored_SlumpBack" },
      new AnimationConfig { MainAnimation = "IDL_Bored_SwingArms" },
      new AnimationConfig { EnterAnimation = "IDL_HandsOnHips_Base_Enter", MainAnimation = "IDL_HandsOnHips_Base_Loop", ExitAnimation = "IDL_HandsOnHips_Base_Exit", IsLooping = true },
      new AnimationConfig { MainAnimation = "IDL_HeadNod_Small" },
      new AnimationConfig { MainAnimation = "IDL_HeadShake_Disappointed" },
      new AnimationConfig { MainAnimation = "IDL_HeadShake_Small" },
      new AnimationConfig { MainAnimation = "IDL_Inspect_Hands" },
      new AnimationConfig { EnterAnimation = "IDL_Lean_B_Base_Enter", MainAnimation = "IDL_Lean_B_Base_Loop", ExitAnimation = "IDL_Lean_B_Base_Exit", IsLooping = true },
      new AnimationConfig { EnterAnimation = "IDL_Lean_L_Base_Enter", MainAnimation = "IDL_Lean_L_Base_Loop", ExitAnimation = "IDL_Lean_L_Base_Exit", IsLooping = true },
      new AnimationConfig { MainAnimation = "IDL_Look_R" },
      new AnimationConfig { MainAnimation = "IDL_Look_L_Scared" },
      new AnimationConfig { MainAnimation = "IDL_Plead_F" },
      new AnimationConfig { MainAnimation = "IDL_PointHand_Thumb_L" },
      new AnimationConfig { MainAnimation = "IDL_PointHand_Index_F" },
      new AnimationConfig { EnterAnimation = "IDL_Posture_Aggressive_Enter", MainAnimation = "IDL_Posture_Aggressive_Loop", ExitAnimation = "IDL_Posture_Aggressive_Exit", IsLooping = true },
      new AnimationConfig { EnterAnimation = "IDL_Idles_Posture_Slumped_Enter", MainAnimation = "IDL_Idles_Posture_Slumped_Loop", ExitAnimation = "IDL_Idles_Posture_Slumped_Exit", IsLooping = true },
      new AnimationConfig { MainAnimation = "IDL_Stretch_Arms" },
      new AnimationConfig { MainAnimation = "IDL_Stretch_Shoulders" },
      new AnimationConfig { MainAnimation = "IDL_WeightShift_L" },
      new AnimationConfig { MainAnimation = "IDL_WeightShift_R" },
      new AnimationConfig { MainAnimation = "IDL_Yawn_Masc" }
    };

    void Awake()
    {
      _studioService = _registry.StudioService;
    }

    void OnEnable()
    {
      if (_animationCycle == null)
      {
        _animationCycle = StartCoroutine(AnimationCycle());
      }
    }

    void OnDisable()
    {
      if (_animationCycle != null)
      {
        StopCoroutine(_animationCycle);
        _animationCycle = null;
      }
    }

    IEnumerator AnimationCycle()
    {
      while (true)
      {
        var waitTime = Random.Range(_lowerBoundSeconds, _upperBoundSeconds);
        yield return new WaitForSeconds(waitTime);

        var randomIndex = Random.Range(0, _animations.Count);
        var animConfig = _animations[randomIndex];

        var command = new PlayStudioAnimationCommand
        {
          StudioType = _studioType,
          Animation = new StudioAnimation { Name = animConfig.MainAnimation }
        };

        if (animConfig.EnterAnimation != null)
        {
          command.EnterAnimation = new StudioAnimation { Name = animConfig.EnterAnimation };
        }

        if (animConfig.ExitAnimation != null)
        {
          command.ExitAnimation = new StudioAnimation { Name = animConfig.ExitAnimation };
        }

        _studioService?.PlayStudioAnimation(command);

        if (animConfig.IsLooping)
        {
          yield return new WaitForSeconds(_loopSeconds);
        }
        else
        {
          yield return new WaitForEndOfFrame();
          var animationDuration = GetCurrentAnimationDuration();
          yield return new WaitForSeconds(animationDuration);
        }

        var returnToBaseCommand = new PlayStudioAnimationCommand
        {
          StudioType = _studioType,
          Animation = new StudioAnimation { Name = "IDL_Base" }
        };
        _studioService?.PlayStudioAnimation(returnToBaseCommand);
      }
    }

    float GetCurrentAnimationDuration()
    {
      var subject = _studioService?.GetSubject(_studioType);
      if (subject == null)
      {
        _registry.LoggingService.LogError(nameof(PlayerStatusAnimations),
          $"Subject not found for StudioType {_studioType}");
        return 2.0f;
      }

      var animator = subject.GetComponent<Animator>();
      if (animator == null)
      {
        _registry.LoggingService.LogError(nameof(PlayerStatusAnimations),
          $"Animator component not found on subject for StudioType {_studioType}");
        return 2.0f;
      }

      return animator.GetCurrentAnimatorStateInfo(0).length;
    }

    float GetAnimationDuration(string animationName)
    {
      var subject = _studioService?.GetSubject(_studioType);
      if (subject == null) return 2.0f;

      var animator = subject.GetComponent<Animator>();
      if (animator == null || animator.runtimeAnimatorController == null) return 2.0f;

      var clips = animator.runtimeAnimatorController.animationClips;
      foreach (var clip in clips)
      {
        if (clip.name == animationName)
        {
          return clip.length;
        }
      }

      return 2.0f;
    }
  }
}