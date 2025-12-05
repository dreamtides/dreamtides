#nullable enable

using System.Collections.Generic;
using System.Runtime.CompilerServices;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class SitePickObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal float _horizontalSpacing;

    [SerializeField]
    internal float _verticalSpacing;

    [SerializeField]
    internal float _cardWidth;

    [SerializeField]
    internal float _cardHeight;

    [SerializeField]
    internal bool _forceTwoRows;

    [SerializeField]
    internal float _landscapeScaleOverride;

    [SerializeField]
    internal float _landscapeHorizontalSpacingOverride;

    [SerializeField]
    internal RectTransform? _closeSiteButton;

    [SerializeField]
    internal Vector2 _portraitCloseButtonOffset;

    [SerializeField]
    internal Vector2 _landscapeCloseButtonOffset;

    [SerializeField]
    internal bool _preserveLayoutOnRemoval;

    int? _preservedInitialCount;
    readonly Dictionary<Displayable, int> _displayableToIndex = new();
    int _nextSlotIndex;

    protected float VerticalSpacing => _verticalSpacing;
    protected float CardWidth => _cardWidth;
    protected float CardHeight => _cardHeight;

    public float HorizontalSpacing() =>
      IsLandscape() && _landscapeHorizontalSpacingOverride > 0f
        ? _landscapeHorizontalSpacingOverride
        : _horizontalSpacing;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var isLandscape = IsLandscape();
      var horizontalSpacing = HorizontalSpacing();
      var effectiveCount = GetEffectiveCount(count);
      if (effectiveCount <= 0)
      {
        return transform.position;
      }
      else if (isLandscape && !_forceTwoRows)
      {
        var localX = ComputeHorizontalOffset(index, effectiveCount, horizontalSpacing);
        return transform.position + transform.right * localX;
      }
      else
      {
        var topRowCount = (effectiveCount + 1) / 2;
        var bottomRowCount = effectiveCount - topRowCount;

        var isTopRow = index < topRowCount;
        var indexInRow = isTopRow ? index : index - topRowCount;
        var rowCount = isTopRow ? topRowCount : bottomRowCount;

        var localX = ComputeHorizontalOffset(indexInRow, rowCount, horizontalSpacing);

        float localY =
          effectiveCount <= 1 ? 0f : (isTopRow ? _verticalSpacing / 2f : -_verticalSpacing / 2f);

        return transform.position + transform.right * localX + transform.up * localY;
      }
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count)
    {
      if (IsLandscape() && _landscapeScaleOverride > 0f)
      {
        return _landscapeScaleOverride;
      }
      return transform.localScale.x;
    }

    protected override void OnBecameNonEmpty()
    {
      var count = Objects.Count;
      if (_preserveLayoutOnRemoval && _preservedInitialCount == null)
      {
        _preservedInitialCount = count;
      }

      PositionCloseButtonInTopRightCorner();
    }

    protected override void OnBecameEmpty()
    {
      _preservedInitialCount = null;
      _displayableToIndex.Clear();
      _nextSlotIndex = 0;
    }

    protected override void OnUpdateObjectLayout()
    {
      if (DebugUpdateContinuously)
      {
        PositionCloseButtonInTopRightCorner();
      }
    }

    protected override int GetLayoutIndexOverride(Displayable displayable, int index, int count)
    {
      if (!_preserveLayoutOnRemoval)
      {
        return index;
      }

      if (!_displayableToIndex.TryGetValue(displayable, out var slot))
      {
        _displayableToIndex[displayable] = _nextSlotIndex;
        slot = _nextSlotIndex;
        _nextSlotIndex += 1;
      }

      return slot;
    }

    protected int GetEffectiveCount(int count)
    {
      if (
        _preserveLayoutOnRemoval
        && _preservedInitialCount != null
        && count < _preservedInitialCount.Value
      )
      {
        return _preservedInitialCount.Value;
      }
      return count;
    }

    static float ComputeHorizontalOffset(int indexInRow, int rowCount, float spacing)
    {
      if (rowCount <= 1)
      {
        return 0f;
      }
      var totalWidth = spacing * (rowCount - 1);
      return -totalWidth / 2f + indexInRow * spacing;
    }

    void PositionCloseButtonInTopRightCorner()
    {
      if (!_closeSiteButton || Objects.Count == 0)
      {
        return;
      }

      var wasActive = _closeSiteButton.gameObject.activeSelf;
      _closeSiteButton.gameObject.SetActive(true);
      if (!wasActive)
      {
        TweenUtils.FadeInCanvasGroup(ComponentUtils.Get<CanvasGroup>(_closeSiteButton));
      }

      var anchor = Objects[Objects.Count - 1].transform.position;
      if (TryGetTopRightColliderCenter(out var colliderCenter))
      {
        anchor = colliderCenter;
      }

      var worldTarget = GetCloseButtonWorldPosition(anchor);
      var rootRect = Registry.GameViewport.CanvasRootRect;
      if (
        !RectTransformUtility.ScreenPointToLocalPointInRectangle(
          rootRect,
          Registry.GameViewport.WorldToScreenPoint(worldTarget),
          null,
          out var rootLocal
        )
      )
      {
        return;
      }

      var halfSize = GetButtonHalfSizeInRootSpace(rootRect, _closeSiteButton);
      var clampedRootLocal = ClampToCanvasBounds(rootLocal, rootRect.rect, halfSize);
      var worldOnCanvas = rootRect.TransformPoint(
        new Vector3(clampedRootLocal.x, clampedRootLocal.y, 0f)
      );
      var parent = _closeSiteButton.parent;
      if (parent)
      {
        var parentLocal = parent.InverseTransformPoint(worldOnCanvas);
        _closeSiteButton.anchoredPosition = new Vector2(parentLocal.x, parentLocal.y);
      }
    }

    Vector3 GetCloseButtonWorldPosition(Vector3 anchor)
    {
      var offset = IsLandscape() ? _landscapeCloseButtonOffset : _portraitCloseButtonOffset;
      var worldOffset = transform.right * offset.x + transform.up * offset.y;
      return anchor + worldOffset;
    }

    static Vector2 ClampToCanvasBounds(Vector2 rootLocal, Rect rect, Vector2 halfSize)
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
      var x = Mathf.Clamp(rootLocal.x, minX, maxX);
      var y = Mathf.Clamp(rootLocal.y, minY, maxY);
      return new Vector2(x, y);
    }

    static Vector2 GetButtonHalfSizeInRootSpace(RectTransform root, RectTransform button)
    {
      var bounds = RectTransformUtility.CalculateRelativeRectTransformBounds(root, button);
      return new Vector2(bounds.extents.x, bounds.extents.y);
    }

    bool TryGetTopRightColliderCenter(out Vector3 center)
    {
      const float verticalTolerance = 0.1f;
      var found = false;
      var bestViewport = Vector2.zero;
      var bestCenter = Vector3.zero;
      foreach (var displayable in Objects)
      {
        var colliders = displayable.GetComponentsInChildren<BoxCollider>(true);
        foreach (var collider in colliders)
        {
          if (!collider || !collider.enabled)
          {
            continue;
          }
          var worldCenter = collider.bounds.center;
          var viewport = Registry.GameViewport.WorldToViewportPoint(worldCenter);
          if (viewport.z <= 0f)
          {
            continue;
          }
          if (
            !found
            || viewport.y > bestViewport.y + verticalTolerance
            || (
              Mathf.Abs(viewport.y - bestViewport.y) <= verticalTolerance
              && viewport.x > bestViewport.x
            )
          )
          {
            bestViewport = new Vector2(viewport.x, viewport.y);
            bestCenter = worldCenter;
            found = true;
          }
        }
      }
      center = bestCenter;
      return found;
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      var center = transform.position;
      var halfLayoutX = _cardWidth / 2f + HorizontalSpacing() / 2f;
      var halfLayoutY = _cardHeight / 2f + _verticalSpacing / 2f;

      var right = transform.right;
      var upAxis = transform.up;

      Gizmos.DrawSphere(center, 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
    }
  }
}
