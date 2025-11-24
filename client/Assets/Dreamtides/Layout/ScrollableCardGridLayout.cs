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
  public class ScrollableCardGridLayout : MonoBehaviour
  {
    [SerializeField]
    internal Canvas _canvas = null!;

    [SerializeField]
    internal Camera _camera = null!;

    [SerializeField]
    internal RectTransform _parent = null!;

    [SerializeField]
    internal Color _color;

    [SerializeField]
    internal float _horizontalSpacing;

    [SerializeField]
    internal float _verticalSpacing;

    [SerializeField]
    internal float _cardWidth;

    [SerializeField]
    internal float _cardHeight;

    [SerializeField]
    internal float _decelerationRate = 0.135f;

    [SerializeField]
    internal float _scrollSensitivity = 20f;

    [SerializeField]
    internal float _elasticity = 0.1f;

    [SerializeField]
    internal float _worldSpaceDepth = 15f;

    [SerializeField]
    internal float _cardScale = 1f;

    Dictionary<Card, RectTransform> _cardToRectangle = new();
    List<Card> _cards = new();
    UnityEngine.UI.ScrollRect? _scrollRect;

    public void SetCards(List<Card> cards)
    {
      _cards = cards;
      ClearExistingRectangles();
      if (cards.Count == 0)
      {
        return;
      }

      EnsureScrollViewSetup();
      CreateRectanglesForCards();
      UpdateCardPositions();
    }

    IEnumerator Start()
    {
      yield return new WaitForSeconds(0.5f);
      var cards = FindObjectsByType<Card>(FindObjectsSortMode.None);
      foreach (var card in cards)
      {
        card.transform.SetParent(null);
      }
      SetCards(cards.ToList());
    }

    void Update()
    {
      if (_cards.Count > 0)
      {
        UpdateCardPositions();
      }
    }

    void ClearExistingRectangles()
    {
      _cardToRectangle.Clear();
      foreach (Transform child in _parent)
      {
        Destroy(child.gameObject);
      }
    }

    void CreateRectanglesForCards()
    {
      Canvas.ForceUpdateCanvases();

      var currentX = 0f;
      var currentY = 0f;
      var containerWidth = _parent.rect.width;

      for (var i = 0; i < _cards.Count; i++)
      {
        if (currentX + _cardWidth > containerWidth && currentX > 0)
        {
          currentX = 0f;
          currentY -= _cardHeight + _verticalSpacing;
        }

        var rectangleObject = new GameObject($"CardRectangle_{i}");
        rectangleObject.transform.SetParent(_parent, worldPositionStays: false);

        var image = rectangleObject.AddComponent<UnityEngine.UI.Image>();
        image.color = _color;

        var rectTransform = rectangleObject.GetComponent<RectTransform>();
        rectTransform.anchorMin = new Vector2(x: 0, y: 1);
        rectTransform.anchorMax = new Vector2(x: 0, y: 1);
        rectTransform.pivot = new Vector2(x: 0, y: 1);
        rectTransform.sizeDelta = new Vector2(_cardWidth, _cardHeight);
        rectTransform.anchoredPosition = new Vector2(currentX, currentY);

        _cardToRectangle[_cards[i]] = rectTransform;

        currentX += _cardWidth + _horizontalSpacing;
      }

      var totalHeight = Mathf.Abs(currentY - _cardHeight);
      _parent.sizeDelta = new Vector2(_parent.sizeDelta.x, totalHeight);
    }

    void UpdateCardPositions()
    {
      Canvas.ForceUpdateCanvases();

      var canvasCamera =
        _canvas.renderMode == RenderMode.ScreenSpaceCamera ? _canvas.worldCamera : null;

      foreach (var kvp in _cardToRectangle)
      {
        var card = kvp.Key;
        var rectTransform = kvp.Value;

        if (card == null || rectTransform == null)
        {
          continue;
        }

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

        card.transform.position = worldCenter;
        card.transform.localScale = Vector3.one * _cardScale;
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

    void EnsureScrollViewSetup()
    {
      var existingScrollRect = _parent.GetComponentInParent<UnityEngine.UI.ScrollRect>();
      if (existingScrollRect != null)
      {
        _scrollRect = existingScrollRect;
        return;
      }

      var scrollViewGo = new GameObject("ScrollView");
      var scrollViewRect = scrollViewGo.AddComponent<RectTransform>();
      scrollViewGo.transform.SetParent(_parent.parent, worldPositionStays: false);

      scrollViewRect.anchorMin = _parent.anchorMin;
      scrollViewRect.anchorMax = _parent.anchorMax;
      scrollViewRect.pivot = _parent.pivot;
      scrollViewRect.anchoredPosition = _parent.anchoredPosition;
      scrollViewRect.sizeDelta = _parent.sizeDelta;

      var scrollRect = scrollViewGo.AddComponent<UnityEngine.UI.ScrollRect>();
      scrollRect.horizontal = false;
      scrollRect.vertical = true;
      scrollRect.scrollSensitivity = _scrollSensitivity;
      scrollRect.decelerationRate = _decelerationRate;
      scrollRect.elasticity = _elasticity;
      scrollRect.movementType = UnityEngine.UI.ScrollRect.MovementType.Elastic;

      var viewportGo = new GameObject("Viewport");
      var viewportRect = viewportGo.AddComponent<RectTransform>();
      viewportGo.transform.SetParent(scrollViewRect, worldPositionStays: false);
      viewportRect.anchorMin = Vector2.zero;
      viewportRect.anchorMax = Vector2.one;
      viewportRect.sizeDelta = Vector2.zero;
      viewportRect.pivot = new Vector2(x: 0, y: 1);

      viewportGo.AddComponent<UnityEngine.UI.RectMask2D>();
      var viewportImage = viewportGo.AddComponent<UnityEngine.UI.Image>();
      viewportImage.color = new Color(r: 0, g: 0, b: 0, a: 0);

      _parent.SetParent(viewportRect, worldPositionStays: false);
      _parent.anchorMin = new Vector2(x: 0, y: 1);
      _parent.anchorMax = new Vector2(x: 1, y: 1);
      _parent.pivot = new Vector2(x: 0, y: 1);
      _parent.anchoredPosition = Vector2.zero;
      _parent.sizeDelta = new Vector2(x: 0, y: 0);

      scrollRect.content = _parent;
      scrollRect.viewport = viewportRect;

      var scrollbarGo = new GameObject("Scrollbar Vertical");
      var scrollbarRect = scrollbarGo.AddComponent<RectTransform>();
      scrollbarGo.transform.SetParent(scrollViewRect, worldPositionStays: false);

      scrollbarRect.anchorMin = new Vector2(x: 1, y: 0);
      scrollbarRect.anchorMax = new Vector2(x: 1, y: 1);
      scrollbarRect.pivot = new Vector2(x: 1, y: 1);
      scrollbarRect.sizeDelta = new Vector2(x: 20, y: 0);

      var scrollbarBg = scrollbarGo.AddComponent<UnityEngine.UI.Image>();
      scrollbarBg.color = new Color(r: 0, g: 0, b: 0, a: 0.1f);

      var scrollbar = scrollbarGo.AddComponent<UnityEngine.UI.Scrollbar>();
      scrollbar.direction = UnityEngine.UI.Scrollbar.Direction.BottomToTop;

      var handleAreaGo = new GameObject("Sliding Area");
      var handleAreaRect = handleAreaGo.AddComponent<RectTransform>();
      handleAreaGo.transform.SetParent(scrollbarRect, worldPositionStays: false);
      handleAreaRect.anchorMin = Vector2.zero;
      handleAreaRect.anchorMax = Vector2.one;
      handleAreaRect.sizeDelta = Vector2.zero;

      var handleGo = new GameObject("Handle");
      var handleRect = handleGo.AddComponent<RectTransform>();
      handleGo.transform.SetParent(handleAreaRect, worldPositionStays: false);
      handleRect.sizeDelta = new Vector2(x: 0, y: 0);

      var handleImage = handleGo.AddComponent<UnityEngine.UI.Image>();
      handleImage.color = new Color(r: 0.5f, g: 0.5f, b: 0.5f, a: 1f);

      scrollbar.targetGraphic = handleImage;
      scrollbar.handleRect = handleRect;

      scrollRect.verticalScrollbar = scrollbar;

      _scrollRect = scrollRect;
    }
  }
}
