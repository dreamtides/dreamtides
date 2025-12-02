#nullable enable

using System.Collections.Generic;
using Dreamtides.Layout;
using Unity.Cinemachine;
using UnityEngine;

namespace Dreamtides.Components
{
  public class DreamscapeMapCamera : Displayable
  {
    [SerializeField]
    CinemachineCamera _camera = null!;

    [SerializeField]
    CinemachineTargetGroup _targetGroup = null!;

    [SerializeField]
    CinemachineGroupFraming _groupFraming = null!;

    [SerializeField]
    List<DreamscapeSite> _sites = new();

    [SerializeField]
    float _targetRadius = 0.5f;

    public CinemachineCamera Camera => _camera;

    public CinemachineTargetGroup TargetGroup => _targetGroup;

    public void FrameSites()
    {
      if (_camera == null || _targetGroup == null || _groupFraming == null)
      {
        return;
      }
      UpdateTargets();
      if (_targetGroup.Targets.Count == 0)
      {
        return;
      }
      var rotation = Quaternion.Euler(50f, Random.Range(0f, 360f), 0f);
      _camera.transform.rotation = rotation;
      _targetGroup.transform.rotation = rotation;
      _targetGroup.PositionMode = CinemachineTargetGroup.PositionModes.GroupCenter;
      _targetGroup.RotationMode = CinemachineTargetGroup.RotationModes.Manual;
      _camera.Follow = _targetGroup.transform;
      _camera.LookAt = _targetGroup.transform;
      _groupFraming.LateralAdjustment = CinemachineGroupFraming
        .LateralAdjustmentModes
        .ChangePosition;
      _groupFraming.SizeAdjustment = CinemachineGroupFraming.SizeAdjustmentModes.DollyOnly;
      _groupFraming.FramingMode = CinemachineGroupFraming.FramingModes.HorizontalAndVertical;
      _groupFraming.FramingSize = 1f;
      _groupFraming.DollyRange = new Vector2(-1000f, 1000f);
    }

    protected override void OnInitialize()
    {
      FrameSites();
    }

    void UpdateTargets()
    {
      _targetGroup.Targets.Clear();
      foreach (var site in _sites)
      {
        if (site == null)
        {
          continue;
        }
        _targetGroup.Targets.Add(
          new CinemachineTargetGroup.Target
          {
            Object = site.transform,
            Weight = 1f,
            Radius = Mathf.Max(0f, _targetRadius),
          }
        );
      }
    }
  }
}
