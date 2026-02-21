#nullable enable

using UnityEngine;

namespace Abu
{
    /// <summary>
    /// Default settled provider that waits a fixed number of frames after an action
    /// before reporting settled. Does not reference DOTween.
    /// </summary>
    public class DefaultSettledProvider : ISettledProvider
    {
        const int DefaultSettleFrames = 3;
        const float DefaultMaxTimeoutSeconds = 3f;

        readonly int _settleFrames;
        readonly float _maxTimeoutSeconds;

        int _framesRemaining;
        float _actionTime;
        bool _actionInProgress;

        public DefaultSettledProvider(
            int settleFrames = DefaultSettleFrames,
            float maxTimeoutSeconds = DefaultMaxTimeoutSeconds
        )
        {
            _settleFrames = settleFrames;
            _maxTimeoutSeconds = maxTimeoutSeconds;
        }

        /// <summary>
        /// Returns true when enough frames have elapsed or the timeout has been reached.
        /// </summary>
        public bool IsSettled()
        {
            if (!_actionInProgress)
            {
                return true;
            }

            if (Time.realtimeSinceStartup - _actionTime >= _maxTimeoutSeconds)
            {
                _actionInProgress = false;
                return true;
            }

            if (BusyToken.IsAnyActive)
            {
                _framesRemaining = _settleFrames;
                return false;
            }

            _framesRemaining--;
            if (_framesRemaining <= 0)
            {
                _actionInProgress = false;
                return true;
            }

            return false;
        }

        /// <summary>
        /// Called when an input action is dispatched to reset the frame counter.
        /// </summary>
        public void NotifyActionDispatched()
        {
            _framesRemaining = _settleFrames;
            _actionTime = Time.realtimeSinceStartup;
            _actionInProgress = true;
        }
    }
}
