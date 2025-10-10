#nullable enable

using System;
using Dreamtides.Services;
using UnityEngine;

namespace Dreamtides.Layout
{
  public class AlternateLandscapePositionSceneElement : SceneElement
  {
    [SerializeField]
    Transform _landscapePosition = null!;

    [SerializeField]
    bool _debugUpdateContinuously;

    protected override void OnInitialize(GameMode mode, TestConfiguration? testConfiguration)
    {
      if (Registry.IsLandscape)
      {
        transform.position = _landscapePosition.position;
      }
    }

    protected override void OnUpdate(GameMode mode, TestConfiguration? testConfiguration)
    {
      if (_debugUpdateContinuously && Registry.IsLandscape)
      {
        transform.position = _landscapePosition.position;
      }
    }
  }
}
