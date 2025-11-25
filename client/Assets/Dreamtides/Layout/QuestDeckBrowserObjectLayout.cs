#nullable enable

using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class QuestDeckBrowserObjectLayout : ObjectLayout
  {
    [SerializeField]
    internal Sprite _sprite = null!;

    [SerializeField]
    internal RectTransform _content = null!;

    [SerializeField]
    internal float _cardWidth;

    [SerializeField]
    internal float _cardHeight;

    [SerializeField]
    internal float _cardSpacing;

    [SerializeField]
    internal Color _tintColor;

    [SerializeField]
    internal float _worldSpaceDepth = 15f;

    [SerializeField]
    internal float _cardScale = 1f;

    [SerializeField]
    internal Vector2 _worldSpaceOffset;

    [SerializeField]
    internal Transform _worldSpaceParent = null!;

    public Transform WorldSpaceParent => _worldSpaceParent;

    [SerializeField]
    List<Displayable> _objects = new();

    List<RectTransform> _rectangles = new();

    public override IReadOnlyList<Displayable> Objects => _objects.AsReadOnly();

    public override void Add(Displayable displayable)
    {
      Errors.CheckNotNull(displayable);

      if (!_objects.Contains(displayable))
      {
        if (displayable.Parent)
        {
          displayable.Parent.RemoveIfPresent(displayable);
        }

        displayable.Parent = this;
        _objects.Add(displayable);
      }

      if (!displayable.ExcludeFromLayout)
      {
        displayable.GameContext = GameContext;
      }

      if (_worldSpaceParent != null && displayable.transform.parent != _worldSpaceParent)
      {
        displayable.transform.SetParent(_worldSpaceParent, worldPositionStays: true);
      }

      SortObjects();
    }

    public override void AddRange(IEnumerable<Displayable> displayables) =>
      displayables.ToList().ForEach(Add);

    public override void RemoveIfPresent(Displayable? displayable)
    {
      if (displayable)
      {
        displayable.Parent = null;

        if (displayable.transform.parent == _worldSpaceParent)
        {
          displayable.transform.SetParent(null, worldPositionStays: true);
        }

        _objects.Remove(displayable);
        SortObjects();
      }
    }

    public override void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      ApplyLayoutToObjectLocal(target, _objects.Count, _objects.Count + 1, sequence);
    }

    public override void ApplyLayout(Sequence? sequence = null)
    {
      for (var i = 0; i < _objects.Count; ++i)
      {
        ApplyLayoutToObjectLocal(_objects[i], i, _objects.Count, sequence);
      }
    }

    public Vector3 CalculateObjectPosition(int index, int count)
    {
      if (index < 0 || index >= _rectangles.Count)
      {
        return Vector3.zero;
      }

      Canvas.ForceUpdateCanvases();

      var rectTransform = _rectangles[index];

      GetRectangleScreenBounds(
        rectTransform,
        out var minX,
        out var maxX,
        out var minY,
        out var maxY
      );

      var screenCenter = new Vector2((minX + maxX) * 0.5f, (minY + maxY) * 0.5f);

      var worldCenter = Registry.GameViewport.ScreenToWorldPoint(
        new Vector3(screenCenter.x, screenCenter.y, _worldSpaceDepth)
      );

      var worldPosition = worldCenter + new Vector3(_worldSpaceOffset.x, _worldSpaceOffset.y, z: 0);

      if (_worldSpaceParent != null)
      {
        return _worldSpaceParent.InverseTransformPoint(worldPosition);
      }

      return worldPosition;
    }

    public Vector3? CalculateObjectRotation(int index, int count)
    {
      return Vector3.zero;
    }

    public float? CalculateObjectScale(int index, int count)
    {
      return _cardScale;
    }

    protected override void OnStart()
    {
      Canvas.ForceUpdateCanvases();

      var viewportWidth = _content.rect.width;
      var columns = Mathf.Max(
        1,
        Mathf.FloorToInt((viewportWidth + _cardSpacing) / (_cardWidth + _cardSpacing))
      );
      var rows = Mathf.CeilToInt(50f / columns);

      var totalRowWidth = (columns * _cardWidth) + ((columns - 1) * _cardSpacing);
      var horizontalOffset = (viewportWidth - totalRowWidth) * 0.5f;

      var currentX = horizontalOffset;
      var currentY = -_cardSpacing;
      var itemsCreated = 0;

      for (var row = 0; row < rows; row++)
      {
        for (var col = 0; col < columns && itemsCreated < 50; col++)
        {
          var imageObject = new GameObject($"Card_{itemsCreated}");
          imageObject.transform.SetParent(_content, worldPositionStays: false);

          var image = imageObject.AddComponent<UnityEngine.UI.Image>();
          image.sprite = _sprite;
          image.color = _tintColor;

          var rectTransform = imageObject.GetComponent<RectTransform>();
          rectTransform.anchorMin = new Vector2(x: 0, y: 1);
          rectTransform.anchorMax = new Vector2(x: 0, y: 1);
          rectTransform.pivot = new Vector2(x: 0, y: 1);
          rectTransform.sizeDelta = new Vector2(_cardWidth, _cardHeight);
          rectTransform.anchoredPosition = new Vector2(currentX, currentY);

          _rectangles.Add(rectTransform);

          currentX += _cardWidth + _cardSpacing;
          itemsCreated++;
        }

        currentX = horizontalOffset;
        currentY -= _cardHeight + _cardSpacing;
      }

      var totalHeight = (rows * _cardHeight) + ((rows - 1) * _cardSpacing) + (2 * _cardSpacing);
      _content.sizeDelta = new Vector2(_content.sizeDelta.x, totalHeight);
    }

    // protected override void OnUpdate()
    // {
    //   if (_objects.Count > 0)
    //   {
    //     ApplyLayout();
    //   }
    // }

    void SortObjects()
    {
      _objects.Sort((a, b) => a.SortingKey.CompareTo(b.SortingKey));
    }

    void ApplyLayoutToObjectLocal(
      Displayable displayable,
      int index,
      int count,
      Sequence? sequence = null
    )
    {
      if (displayable.ExcludeFromLayout)
      {
        return;
      }

      const float duration = TweenUtils.MoveAnimationDurationSeconds;
      var localPosition = CalculateObjectPosition(index, count);
      var localRotation = CalculateObjectRotation(index, count);
      var localScale = CalculateObjectScale(index, count) ?? displayable.DefaultScale;

      if (_worldSpaceParent != null && displayable.transform.parent != _worldSpaceParent)
      {
        displayable.transform.SetParent(_worldSpaceParent, worldPositionStays: true);
      }

      if (IsEquivalent(displayable, localPosition, localRotation, localScale))
      {
        return;
      }

      if (sequence != null)
      {
        sequence.Insert(atPosition: 0, displayable.transform.DOLocalMove(localPosition, duration));
      }
      else
      {
        displayable.transform.localPosition = localPosition;
      }

      if (localRotation is { } euler)
      {
        if (sequence != null)
        {
          sequence.Insert(atPosition: 0, displayable.transform.DOLocalRotate(euler, duration));
        }
        else
        {
          displayable.transform.localEulerAngles = euler;
        }
      }

      if (sequence != null)
      {
        sequence.Insert(
          atPosition: 0,
          displayable.transform.DOScale(Vector3.one * localScale, duration)
        );
      }
      else
      {
        displayable.transform.localScale = Vector3.one * localScale;
      }
    }

    bool IsEquivalent(
      Displayable displayable,
      Vector3 localPosition,
      Vector3? localRotation,
      float localScale
    )
    {
      if (Vector3.Distance(displayable.transform.localPosition, localPosition) > 0.01f)
      {
        return false;
      }

      if (
        localRotation != null
        && Vector3.Distance(
          EulerAngleDistance(displayable.transform.localEulerAngles, localRotation.Value),
          Vector3.zero
        ) > 0.01f
      )
      {
        return false;
      }

      if (Vector3.Distance(displayable.transform.localScale, localScale * Vector3.one) > 0.01f)
      {
        return false;
      }

      return true;
    }

    Vector3 EulerAngleDistance(Vector3 a, Vector3 b) =>
      new(Mathf.DeltaAngle(a.x, b.x), Mathf.DeltaAngle(a.y, b.y), Mathf.DeltaAngle(a.z, b.z));

    void GetRectangleScreenBounds(
      RectTransform rectTransform,
      out float minX,
      out float maxX,
      out float minY,
      out float maxY
    )
    {
      var corners = new Vector3[4];
      rectTransform.GetWorldCorners(corners);

      minX = float.MaxValue;
      maxX = float.MinValue;
      minY = float.MaxValue;
      maxY = float.MinValue;

      foreach (var corner in corners)
      {
        var screenPoint = corner;
        minX = Mathf.Min(minX, screenPoint.x);
        maxX = Mathf.Max(maxX, screenPoint.x);
        minY = Mathf.Min(minY, screenPoint.y);
        maxY = Mathf.Max(maxY, screenPoint.y);
      }
    }
  }
}
