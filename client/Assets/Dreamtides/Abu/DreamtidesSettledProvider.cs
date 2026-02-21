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

      // Max timeout: report settled to prevent indefinite hanging
      if (Time.realtimeSinceStartup - _actionTime >= _maxTimeoutSeconds)
      {
        _actionInProgress = false;
        return true;
      }

      var conditionsMet =
        !_actionService.IsProcessingCommands
        && !_actionService.LastResponseIncremental
        && _animationsComplete();

      if (conditionsMet)
      {
        _settledFrameCount++;
        if (_settledFrameCount >= _settleFrames)
        {
          _actionInProgress = false;
          return true;
        }
      }
      else
      {
        _settledFrameCount = 0;
      }

      return false;
    }

    public void NotifyActionDispatched()
    {
      _settledFrameCount = 0;
      _actionTime = Time.realtimeSinceStartup;
      _actionInProgress = true;
    }

    static bool DefaultAnimationsComplete()
    {
      return DOTween.TotalPlayingTweens() <= 0;
    }
  }
}
