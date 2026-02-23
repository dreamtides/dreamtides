#nullable enable

using System;
using Abu;
using DG.Tweening;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Abu
{
  /// <summary>
  /// Settled provider for Dreamtides that checks both game command processing
  /// and DOTween animation state before reporting settled. Requires N consecutive
  /// settled frames and enforces a max timeout to prevent indefinite waiting.
  /// </summary>
  public class DreamtidesSettledProvider : ISettledProvider
  {
    const int DefaultSettleFrames = 3;
    const float DefaultMaxTimeoutSeconds = 3f;

    readonly ActionService _actionService;
    readonly Func<bool> _animationsComplete;
    readonly int _settleFrames;
    readonly float _maxTimeoutSeconds;

    int _settledFrameCount;
    float _actionTime;
    bool _actionInProgress;

    /// <summary>
    /// Creates a settled provider that checks IsProcessingCommands and
    /// DOTween.TotalPlayingTweens() by default, or a custom animation
    /// predicate for testability.
    /// </summary>
    public DreamtidesSettledProvider(
      ActionService actionService,
      int settleFrames = DefaultSettleFrames,
      float maxTimeoutSeconds = DefaultMaxTimeoutSeconds,
      Func<bool>? animationsComplete = null
    )
    {
      if (settleFrames < 1)
      {
        throw new ArgumentOutOfRangeException(nameof(settleFrames), "Settle frames must be >= 1.");
      }

      if (maxTimeoutSeconds < 0f)
      {
        throw new ArgumentOutOfRangeException(
          nameof(maxTimeoutSeconds),
          "Max timeout seconds must be >= 0."
        );
      }

      _actionService = actionService;
      _settleFrames = settleFrames;
      _maxTimeoutSeconds = maxTimeoutSeconds;
      _animationsComplete = animationsComplete ?? DefaultAnimationsComplete;
    }

    public bool IsSettled()
    {
      if (!_actionInProgress)
      {
        return true;
      }

      if (IsWaitingForFinalResponse())
      {
        return false;
      }

      if (IsTimedOut())
      {
        return MarkActionComplete();
      }

      return EvaluateLocalSettleState();
    }

    public void NotifyActionDispatched()
    {
      _actionTime = Time.realtimeSinceStartup;
      _actionInProgress = true;
      _settledFrameCount = 0;
    }

    bool EvaluateLocalSettleState()
    {
      if (!IsLocalSettled())
      {
        _settledFrameCount = 0;
        return false;
      }

      _settledFrameCount++;
      return _settledFrameCount >= _settleFrames && MarkActionComplete();
    }

    bool IsLocalSettled()
    {
      return !_actionService.IsProcessingCommands && _animationsComplete();
    }

    bool IsWaitingForFinalResponse()
    {
      if (!_actionService.WaitingForFinalResponse)
      {
        return false;
      }

      _settledFrameCount = 0;
      return true;
    }

    bool IsTimedOut()
    {
      return Time.realtimeSinceStartup - _actionTime >= _maxTimeoutSeconds;
    }

    bool MarkActionComplete()
    {
      _actionInProgress = false;
      _settledFrameCount = 0;
      return true;
    }

    static bool DefaultAnimationsComplete()
    {
      return DOTween.TotalPlayingTweens() <= 0;
    }
  }
}
