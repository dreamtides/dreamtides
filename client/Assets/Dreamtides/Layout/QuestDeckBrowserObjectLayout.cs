#nullable enable

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using Dreamtides.Components;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class QuestDeckBrowserObjectLayout : StandardObjectLayout
  {
    [SerializeField]
    internal Sprite _sprite = null!;

    [SerializeField]
    internal Canvas _canvas = null!;

    [SerializeField]
    internal Camera _camera = null!;

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

    List<RectTransform> _rectangles = new();

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (index < 0 || index >= _rectangles.Count)
      {
        return Vector3.zero;
      }

      Canvas.ForceUpdateCanvases();

      var rectTransform = _rectangles[index];
      var canvasCamera =
        _canvas.renderMode == RenderMode.ScreenSpaceCamera ? _canvas.worldCamera : null;

      GetRectangleScreenBounds(
        rectTransform,
        canvasCamera,
        out var minX,
        out var maxX,
        out var minY,
        out var maxY
      );

      var screenCenter = new Vector2((minX + maxX) * 0.5f, (minY + maxY) * 0.5f);

      var worldCenter = _camera.ScreenToWorldPoint(
        new Vector3(screenCenter.x, screenCenter.y, _worldSpaceDepth)
      );

      return worldCenter + new Vector3(_worldSpaceOffset.x, _worldSpaceOffset.y, z: 0);
    }

    public override float? CalculateObjectScale(int index, int count)
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

    protected override IEnumerator? OnStartAsync()
    {
      yield return new WaitForSeconds(0.5f);
      var cards = FindObjectsByType<Card>(FindObjectsSortMode.None);
      foreach (var card in cards)
      {
        card.transform.SetParent(null);
      }
      AddRange(cards);
    }

    protected override void OnUpdateObjectLayout()
    {
      if (Objects.Count > 0)
      {
        ApplyLayout();
      }
    }

    void GetRectangleScreenBounds(
      RectTransform rectTransform,
      Camera? canvasCamera,
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
        Vector2 screenPoint;

        if (_canvas.renderMode == RenderMode.ScreenSpaceOverlay)
        {
          screenPoint = corner;
        }
        else if (canvasCamera != null)
        {
          screenPoint = RectTransformUtility.WorldToScreenPoint(canvasCamera, corner);
        }
        else
        {
          screenPoint = RectTransformUtility.WorldToScreenPoint(_camera, corner);
        }

        minX = Mathf.Min(minX, screenPoint.x);
        maxX = Mathf.Max(maxX, screenPoint.x);
        minY = Mathf.Min(minY, screenPoint.y);
        maxY = Mathf.Max(maxY, screenPoint.y);
      }
    }
  }
}
