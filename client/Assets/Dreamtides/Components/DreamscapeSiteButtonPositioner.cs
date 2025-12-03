#nullable enable

using System;
using System.Collections.Generic;
using UnityEngine;

namespace Dreamtides.Components
{
  public sealed class DreamscapeSiteButtonPositioner
  {
    const float DefaultButtonSize = 20f;
    const float SeparationPadding = 0.5f;
    const int MaxSearchIterations = 512;
    const float QuantizeFactor = 1000f;

    readonly IGameViewport _viewport;
    readonly RectTransform _safeArea;

    public DreamscapeSiteButtonPositioner(IGameViewport viewport, RectTransform safeArea)
    {
      _viewport = viewport ?? throw new ArgumentNullException(nameof(viewport));
      _safeArea = safeArea ?? throw new ArgumentNullException(nameof(safeArea));
    }

    public IReadOnlyList<Vector2> PositionButtons(
      IReadOnlyList<Vector3> worldPositions,
      IReadOnlyList<RectTransform> buttons
    )
    {
      if (worldPositions == null)
      {
        throw new ArgumentNullException(nameof(worldPositions));
      }
      if (buttons == null)
      {
        throw new ArgumentNullException(nameof(buttons));
      }
      var count = worldPositions.Count;
      if (buttons.Count != count)
      {
        throw new ArgumentException("Position and button counts must match.", nameof(buttons));
      }
      var safeRect = _safeArea.rect;
      var desiredPositions = new List<Vector2>(count);
      var halfSizes = new List<Vector2>(count);
      for (var i = 0; i < count; i++)
      {
        var button = buttons[i];
        if (button == null)
        {
          throw new ArgumentNullException(nameof(buttons));
        }
        var halfSize = GetHalfSize(button);
        halfSizes.Add(halfSize);
        var desired = GetDesiredPosition(worldPositions[i], halfSize);
        desiredPositions.Add(ClampToRect(desired, safeRect, halfSize));
      }

      var resolved = new Vector2[count];
      var placedCenters = new List<Vector2>(count);
      var placedHalfSizes = new List<Vector2>(count);
      var order = GetPlacementOrder(desiredPositions);

      for (var i = 0; i < order.Count; i++)
      {
        var index = order[i];
        var position = FindPosition(
          desiredPositions[index],
          halfSizes[index],
          safeRect,
          placedCenters,
          placedHalfSizes
        );
        resolved[index] = position;
        placedCenters.Add(position);
        placedHalfSizes.Add(halfSizes[index]);
      }

      for (var i = 0; i < count; i++)
      {
        buttons[i].anchoredPosition = resolved[i];
      }

      return resolved;
    }

    Vector2 GetDesiredPosition(Vector3 worldPosition, Vector2 halfSize)
    {
      var screenPoint = _viewport.WorldToScreenPoint(worldPosition);
      if (
        RectTransformUtility.ScreenPointToLocalPointInRectangle(
          _safeArea,
          screenPoint,
          null,
          out var local
        )
      )
      {
        return new Vector2(local.x, local.y + halfSize.y);
      }

      var safeRect = _safeArea.rect;
      var fallback = new Vector2(
        (safeRect.xMin + safeRect.xMax) * 0.5f,
        (safeRect.yMin + safeRect.yMax) * 0.5f + halfSize.y
      );
      return fallback;
    }

    static Vector2 GetHalfSize(RectTransform button)
    {
      var bounds = RectTransformUtility.CalculateRelativeRectTransformBounds(button.parent, button);
      var extents = bounds.extents;
      var halfSize = new Vector2(extents.x, extents.y);
      if (halfSize.x > 0f && halfSize.y > 0f)
      {
        return halfSize;
      }
      var fallback = DefaultButtonSize * 0.5f;
      return new Vector2(fallback, fallback);
    }

    static List<int> GetPlacementOrder(IReadOnlyList<Vector2> desired)
    {
      var indices = new List<int>(desired.Count);
      for (var i = 0; i < desired.Count; i++)
      {
        indices.Add(i);
      }
      indices.Sort(
        (a, b) =>
        {
          var yComparison = -desired[a].y.CompareTo(desired[b].y);
          if (yComparison != 0)
          {
            return yComparison;
          }
          var xComparison = desired[a].x.CompareTo(desired[b].x);
          if (xComparison != 0)
          {
            return xComparison;
          }
          return a.CompareTo(b);
        }
      );
      return indices;
    }

    Vector2 FindPosition(
      Vector2 desired,
      Vector2 halfSize,
      Rect safeRect,
      List<Vector2> placedCenters,
      List<Vector2> placedHalfSizes
    )
    {
      var bestCandidate = ClampToRect(desired, safeRect, halfSize);
      var queue = new List<Vector2> { bestCandidate };
      var visited = new HashSet<QuantizedVector2>(new QuantizedVector2Comparer());
      while (queue.Count > 0 && visited.Count < MaxSearchIterations)
      {
        var index = GetLowestCostIndex(queue, desired);
        var candidate = queue[index];
        queue.RemoveAt(index);

        var key = new QuantizedVector2(candidate);
        if (!visited.Add(key))
        {
          continue;
        }

        if (!OverlapsAny(candidate, halfSize, placedCenters, placedHalfSizes))
        {
          return candidate;
        }

        var overlaps = GetOverlappingIndices(candidate, halfSize, placedCenters, placedHalfSizes);
        for (var i = 0; i < overlaps.Count; i++)
        {
          var neighborIndex = overlaps[i];
          var neighborCenter = placedCenters[neighborIndex];
          var neighborHalf = placedHalfSizes[neighborIndex];
          foreach (
            var next in GenerateNeighborPositions(
              candidate,
              desired,
              halfSize,
              neighborCenter,
              neighborHalf
            )
          )
          {
            var clamped = ClampToRect(next, safeRect, halfSize);
            queue.Add(clamped);
          }
        }
      }

      var gridCandidate = SearchGridForPosition(
        desired,
        halfSize,
        safeRect,
        placedCenters,
        placedHalfSizes
      );
      if (gridCandidate.HasValue)
      {
        return gridCandidate.Value;
      }

      return bestCandidate;
    }

    static int GetLowestCostIndex(List<Vector2> queue, Vector2 desired)
    {
      var bestIndex = 0;
      var bestCost = Cost(queue[0], desired);
      for (var i = 1; i < queue.Count; i++)
      {
        var cost = Cost(queue[i], desired);
        if (cost < bestCost)
        {
          bestCost = cost;
          bestIndex = i;
        }
        else if (Mathf.Approximately(cost, bestCost))
        {
          if (
            queue[i].y > queue[bestIndex].y
            || (
              Mathf.Approximately(queue[i].y, queue[bestIndex].y) && queue[i].x < queue[bestIndex].x
            )
          )
          {
            bestIndex = i;
          }
        }
      }
      return bestIndex;
    }

    static float Cost(Vector2 candidate, Vector2 desired)
    {
      return (candidate - desired).sqrMagnitude;
    }

    static IEnumerable<Vector2> GenerateNeighborPositions(
      Vector2 candidate,
      Vector2 desired,
      Vector2 halfSize,
      Vector2 otherCenter,
      Vector2 otherHalf
    )
    {
      var separationX = otherHalf.x + halfSize.x + SeparationPadding;
      var separationY = otherHalf.y + halfSize.y + SeparationPadding;
      yield return new Vector2(candidate.x, otherCenter.y + separationY);
      yield return new Vector2(candidate.x, otherCenter.y - separationY);
      yield return new Vector2(otherCenter.x + separationX, candidate.y);
      yield return new Vector2(otherCenter.x - separationX, candidate.y);
      yield return new Vector2(desired.x, otherCenter.y + separationY);
      yield return new Vector2(desired.x, otherCenter.y - separationY);
      yield return new Vector2(otherCenter.x + separationX, desired.y);
      yield return new Vector2(otherCenter.x - separationX, desired.y);
      yield return new Vector2(otherCenter.x + separationX, otherCenter.y + separationY);
      yield return new Vector2(otherCenter.x - separationX, otherCenter.y + separationY);
      yield return new Vector2(otherCenter.x + separationX, otherCenter.y - separationY);
      yield return new Vector2(otherCenter.x - separationX, otherCenter.y - separationY);
    }

    static List<int> GetOverlappingIndices(
      Vector2 candidate,
      Vector2 halfSize,
      List<Vector2> placedCenters,
      List<Vector2> placedHalfSizes
    )
    {
      var overlaps = new List<int>();
      for (var i = 0; i < placedCenters.Count; i++)
      {
        if (Overlaps(candidate, halfSize, placedCenters[i], placedHalfSizes[i]))
        {
          overlaps.Add(i);
        }
      }
      return overlaps;
    }

    static bool OverlapsAny(
      Vector2 candidate,
      Vector2 halfSize,
      List<Vector2> placedCenters,
      List<Vector2> placedHalfSizes
    )
    {
      for (var i = 0; i < placedCenters.Count; i++)
      {
        if (Overlaps(candidate, halfSize, placedCenters[i], placedHalfSizes[i]))
        {
          return true;
        }
      }
      return false;
    }

    static bool Overlaps(Vector2 aCenter, Vector2 aHalf, Vector2 bCenter, Vector2 bHalf)
    {
      return Mathf.Abs(aCenter.x - bCenter.x) < aHalf.x + bHalf.x
        && Mathf.Abs(aCenter.y - bCenter.y) < aHalf.y + bHalf.y;
    }

    static Vector2 ClampToRect(Vector2 position, Rect rect, Vector2 halfSize)
    {
      var minX = rect.xMin + halfSize.x;
      var maxX = rect.xMax - halfSize.x;
      var minY = rect.yMin + halfSize.y;
      var maxY = rect.yMax - halfSize.y;
      if (maxX < minX)
      {
        var midX = (rect.xMin + rect.xMax) * 0.5f;
        minX = midX;
        maxX = midX;
      }
      if (maxY < minY)
      {
        var midY = (rect.yMin + rect.yMax) * 0.5f;
        minY = midY;
        maxY = midY;
      }
      var x = Mathf.Clamp(position.x, minX, maxX);
      var y = Mathf.Clamp(position.y, minY, maxY);
      return new Vector2(x, y);
    }

    Vector2? SearchGridForPosition(
      Vector2 desired,
      Vector2 halfSize,
      Rect safeRect,
      List<Vector2> placedCenters,
      List<Vector2> placedHalfSizes
    )
    {
      var startX = safeRect.xMin + halfSize.x;
      var endX = safeRect.xMax - halfSize.x;
      var startY = safeRect.yMin + halfSize.y;
      var endY = safeRect.yMax - halfSize.y;
      if (endX < startX || endY < startY)
      {
        return null;
      }

      var stepX = Mathf.Max(halfSize.x * 2f, 1f);
      var stepY = Mathf.Max(halfSize.y * 2f, 1f);
      Vector2? best = null;
      var bestCost = float.MaxValue;
      for (var x = startX; x <= endX + 0.001f; x += stepX)
      {
        for (var y = startY; y <= endY + 0.001f; y += stepY)
        {
          var candidate = new Vector2(Mathf.Min(x, endX), Mathf.Min(y, endY));
          if (OverlapsAny(candidate, halfSize, placedCenters, placedHalfSizes))
          {
            continue;
          }
          var cost = Cost(candidate, desired);
          if (best == null || cost < bestCost)
          {
            best = candidate;
            bestCost = cost;
          }
          else if (
            Mathf.Approximately(cost, bestCost)
            && best.HasValue
            && (
              candidate.y > best.Value.y
              || (Mathf.Approximately(candidate.y, best.Value.y) && candidate.x < best.Value.x)
            )
          )
          {
            best = candidate;
            bestCost = cost;
          }
        }
      }

      return best;
    }

    readonly struct QuantizedVector2
    {
      public readonly int X;
      public readonly int Y;

      public QuantizedVector2(Vector2 value)
      {
        X = Mathf.RoundToInt(value.x * QuantizeFactor);
        Y = Mathf.RoundToInt(value.y * QuantizeFactor);
      }
    }

    sealed class QuantizedVector2Comparer : IEqualityComparer<QuantizedVector2>
    {
      public bool Equals(QuantizedVector2 a, QuantizedVector2 b)
      {
        return a.X == b.X && a.Y == b.Y;
      }

      public int GetHashCode(QuantizedVector2 value)
      {
        var hash = 17;
        hash = hash * 31 + value.X.GetHashCode();
        hash = hash * 31 + value.Y.GetHashCode();
        return hash;
      }
    }
  }
}
