#nullable enable

using System.Collections.Generic;
using Dreamtides.Services;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Layout
{
  public sealed class SitePickObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    Registry _registry = null!;

    [SerializeField]
    float _horizontalSpacing;

    [SerializeField]
    float _verticalSpacing;

    [SerializeField]
    float _cardWidth;

    [SerializeField]
    float _cardHeight;

    [SerializeField]
    bool _forceTwoRows;

    [SerializeField]
    RectTransform? _closeSiteButton;

    [SerializeField]
    Vector2 _closeButtonCanvasOffsetPortrait;

    [SerializeField]
    Vector2 _closeButtonCanvasOffsetLandscape;

    [SerializeField]
    bool _preserveLayoutOnRemoval;

    int? _preservedInitialCount;
    readonly Dictionary<Displayable, int> _displayableToIndex = new();
    int _nextSlotIndex;

    protected override void OnBecameNonEmpty()
    {
      var count = Objects.Count;
      if (_preserveLayoutOnRemoval && _preservedInitialCount == null)
      {
        _preservedInitialCount = count;
      }
      if (!_closeSiteButton)
      {
        return;
      }

      _closeSiteButton.gameObject.SetActive(true);
      TweenUtils.FadeInCanvasGroup(ComponentUtils.Get<CanvasGroup>(_closeSiteButton));

      var canvas = _registry.Canvas;
      var isLandscape = _registry.IsLandscape;
      var topRowCount = isLandscape && !_forceTwoRows ? count : (count + 1) / 2;
      var topRightIndex = isLandscape && !_forceTwoRows ? count - 1 : topRowCount - 1;

      var target = Objects[topRightIndex];
      var targetWorld = target.transform.position;

      var screenPoint = _registry.MainCamera.WorldToScreenPoint(targetWorld);

      var rootRect = canvas.GetComponent<RectTransform>();
      RectTransformUtility.ScreenPointToLocalPointInRectangle(
        rootRect,
        screenPoint,
        null,
        out var rootLocal
      );

      var worldOnCanvas = rootRect.TransformPoint(rootLocal);
      var parent = _closeSiteButton.parent as RectTransform ?? rootRect;
      var parentLocal = parent.InverseTransformPoint(worldOnCanvas);
      var offset = _registry.IsLandscape
        ? _closeButtonCanvasOffsetLandscape
        : _closeButtonCanvasOffsetPortrait;
      _closeSiteButton.anchoredPosition = new Vector2(parentLocal.x, parentLocal.y) + offset;
    }

    protected override void OnBecameEmpty()
    {
      _preservedInitialCount = null;
      _displayableToIndex.Clear();
      _nextSlotIndex = 0;
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var isLandscape = _registry.IsLandscape;
      var effectiveCount = GetEffectiveCount(count);
      if (effectiveCount <= 0)
      {
        return transform.position;
      }
      else if (isLandscape && !_forceTwoRows)
      {
        var localX = ComputeHorizontalOffset(index, effectiveCount, _horizontalSpacing);
        return transform.position + transform.right * localX;
      }
      else
      {
        var topRowCount = (effectiveCount + 1) / 2;
        var bottomRowCount = effectiveCount - topRowCount;

        var isTopRow = index < topRowCount;
        var indexInRow = isTopRow ? index : index - topRowCount;
        var rowCount = isTopRow ? topRowCount : bottomRowCount;

        var localX = ComputeHorizontalOffset(indexInRow, rowCount, _horizontalSpacing);

        float localY =
          effectiveCount <= 1 ? 0f : (isTopRow ? _verticalSpacing / 2f : -_verticalSpacing / 2f);

        return transform.position + transform.right * localX + transform.up * localY;
      }
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    public override float? CalculateObjectScale(int index, int count) => transform.localScale.x;

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

    static float ComputeHorizontalOffset(int indexInRow, int rowCount, float spacing)
    {
      if (rowCount <= 1)
      {
        return 0f;
      }
      var totalWidth = spacing * (rowCount - 1);
      return -totalWidth / 2f + indexInRow * spacing;
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      var center = transform.position;
      var halfLayoutX = _cardWidth / 2f + _horizontalSpacing / 2f;
      var halfLayoutY = _cardHeight / 2f + _verticalSpacing / 2f;

      var right = transform.right;
      var upAxis = transform.up;

      Gizmos.DrawSphere(center, 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX + upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (-right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
      Gizmos.DrawSphere(center + (right * halfLayoutX - upAxis * halfLayoutY), 0.15f);
    }

    int GetEffectiveCount(int count)
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
  }
}
