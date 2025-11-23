#nullable enable

using System.Runtime.CompilerServices;
using Dreamtides.Components;
using Dreamtides.Services;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class CenteredLineObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _horizontalSpacing = 0.75f;

    [SerializeField]
    internal float _cardWidth = 1f;

    [SerializeField]
    internal float _minScale = 0.85f;

    [SerializeField]
    internal float _maxScale = 1f;

    [SerializeField]
    internal int _minScaleThresholdPortrait = 4;

    [SerializeField]
    internal int _minScaleThresholdLandscape = 8;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count <= 0)
      {
        return transform.position;
      }

      var spacing = DetermineSpacing(count);
      var pivot = (count - 1) * 0.5f;
      var localX = (index - pivot) * spacing;
      return transform.position + transform.right * localX;
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count) => EvaluateScale(count);

    float DetermineSpacing(int count)
    {
      if (count <= 1)
      {
        return 0f;
      }

      var gameViewport = Registry.GameViewport;
      var scale = EvaluateScale(count);
      var requested = Mathf.Max(_horizontalSpacing * scale, 0f);

      if (SpacingFitsViewport(requested, count, scale, gameViewport))
      {
        return requested;
      }

      var minSpacing = 0f;
      var maxSpacing = requested;
      for (var i = 0; i < 12; ++i)
      {
        var mid = (minSpacing + maxSpacing) * 0.5f;
        if (SpacingFitsViewport(mid, count, scale, gameViewport))
        {
          minSpacing = mid;
        }
        else
        {
          maxSpacing = mid;
        }
      }

      return minSpacing;
    }

    float EvaluateScale(int count)
    {
      if (count <= 1)
      {
        return _maxScale;
      }

      var threshold = IsLandscape() ? _minScaleThresholdLandscape : _minScaleThresholdPortrait;
      if (count >= threshold)
      {
        return _minScale;
      }

      var t = Mathf.InverseLerp(1f, threshold, count);
      return Mathf.Lerp(_maxScale, _minScale, t);
    }

    bool SpacingFitsViewport(float spacing, int count, float scale, IGameViewport gameViewport)
    {
      if (count <= 0)
      {
        return true;
      }

      var halfWidthVector = transform.right * (_cardWidth * scale * 0.5f);
      var pivot = (count - 1) * 0.5f;
      for (var i = 0; i < count; ++i)
      {
        var offset = (i - pivot) * spacing;
        var center = transform.position + transform.right * offset;
        var leftEdge = center - halfWidthVector;
        var rightEdge = center + halfWidthVector;
        if (
          !IsPointWithinViewport(leftEdge, gameViewport)
          || !IsPointWithinViewport(rightEdge, gameViewport)
        )
        {
          return false;
        }
      }

      return true;
    }

    static bool IsPointWithinViewport(Vector3 point, IGameViewport gameViewport)
    {
      var viewport = gameViewport.WorldToViewportPoint(point);
      if (viewport.z <= 0f)
      {
        return false;
      }

      return viewport.x is >= 0f and <= 1f;
    }
  }
}
