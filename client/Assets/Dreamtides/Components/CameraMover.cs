using Unity.Cinemachine;
using UnityEngine;

public class CameraMover : MonoBehaviour
{
    [SerializeField] CinemachineCamera _spaceCameraFar;
    [SerializeField] CinemachineCamera _spaceCameraNear;
    [SerializeField] CinemachineCamera _mapCamera;
    [SerializeField] CinemachineCamera _draftCamera;
    [SerializeField] Transform _draftTrackingTarget;
    [SerializeField] CinemachineCamera _shopCamera;
    [SerializeField] Transform _shopTrackingTarget;
    [SerializeField] CinemachineCamera _eventCamera;
    [SerializeField] Transform _eventTrackingTarget;

    void Awake()
    {
        Application.targetFrameRate = 60;
    }

    public void FocusSpaceCameraFar()
    {
        ResetPrioritiesAndTrack(null);
        _spaceCameraFar.Priority = 10;
    }

    public void FocusSpaceCameraNear()
    {
        ResetPrioritiesAndTrack(null);
        _spaceCameraNear.Priority = 10;
    }

    public void FocusMapCamera()
    {
        ResetPrioritiesAndTrack(null);
        _mapCamera.Priority = 10;
    }

    public void FocusDraftCamera()
    {
        ResetPrioritiesAndTrack(_draftTrackingTarget);
        _draftCamera.Priority = 10;
    }

    public void FocusShopCamera()
    {
        ResetPrioritiesAndTrack(_shopTrackingTarget);
        _shopCamera.Priority = 10;
    }

    public void FocusEventCamera()
    {
        ResetPrioritiesAndTrack(_eventTrackingTarget);
        _eventCamera.Priority = 10;
    }

    void ResetPrioritiesAndTrack(Transform track)
    {
        _spaceCameraFar.Priority = 0;
        _spaceCameraNear.Priority = 0;
        _mapCamera.Priority = 0;
        _draftCamera.Priority = 0;
        _shopCamera.Priority = 0;
        _eventCamera.Priority = 0;

        if (track)
        {
            _spaceCameraFar.Target.TrackingTarget = track;
            _spaceCameraNear.Target.TrackingTarget = track;
            _mapCamera.Target.TrackingTarget = track;
            _draftCamera.Target.TrackingTarget = track;
            _shopCamera.Target.TrackingTarget = track;
            _eventCamera.Target.TrackingTarget = track;
        }
    }
}
