#nullable enable

using System;
using System.Runtime.CompilerServices;
using Dreamtides.Services;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class AlternateLandscapePositionSceneElement : SceneElement
  {
    [SerializeField]
    internal Transform _landscapePosition = null!;

    [SerializeField]
    internal bool _debugUpdateContinuously;

    protected override void OnInitialize(GameMode mode, TestConfiguration? testConfiguration)
    {
      if (Registry.IsLandscape)
      {
        transform.position = _landscapePosition.position;
      }
    }

    public override void OnUpdate(GameMode mode, TestConfiguration? testConfiguration)
    {
      if (_debugUpdateContinuously && Registry.IsLandscape)
      {
        transform.position = _landscapePosition.position;
        transform.rotation = _landscapePosition.rotation;
      }
    }
  }
}
