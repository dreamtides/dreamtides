#nullable enable

using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Services;
using UnityEngine;
using UnityEngine.UI;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  public class CardBrowser : AbstractCardBrowser
  {
    [SerializeField]
    internal float _scrollAmount;

    [SerializeField]
    internal Scrollbar _scrollbar = null!;

    [SerializeField]
    internal CloseBrowserButton _closeButton = null!;

    [SerializeField]
    internal float _maxStackOffsetRight = 1f;

    [SerializeField]
    internal Transform _largeCardPosition = null!;

    [SerializeField]
    internal Transform _twoCardsPosition = null!;

    public override void Show(Registry registry, Sequence? sequence)
    {
      if (!IsOpen)
      {
        _scrollbar.value = 1;
        _scrollAmount = 1;
      }

      base.Show(registry, sequence);
    }

    protected override void OnShowComplete()
    {
      SetCloseButtonVisible(true);
    }

    protected override void OnHideComplete()
    {
      SetCloseButtonVisible(false);
    }

    public override void Hide(Registry registry, Sequence? sequence)
    {
      if (_isOpen)
      {
        SetCloseButtonVisible(false);
      }

      base.Hide(registry, sequence);
    }

    public bool SetCloseButtonVisible(bool visible)
    {
      if (_closeButton.gameObject.activeSelf == visible)
      {
        return false;
      }

      _closeButton.SetActiveIfHasAction(visible);
      return true;
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count == 1)
      {
        return _largeCardPosition.position;
      }

      if (count == 2)
      {
        var divide = Registry.IsMobileDevice ? 1.9f : 1.5f;
        var edgeDirection = (_rightEdge.position - _leftEdge.position).normalized;
        var offset = edgeDirection * (_cardWidth / divide);
        if (index == 0)
        {
          return _twoCardsPosition.position - offset;
        }
        else
        {
          return _twoCardsPosition.position + offset;
        }
      }

      return InterpolateEdgePosition(
        SmoothedNormalizedPosition(index, count, Mathf.Clamp01(_scrollAmount))
      );
    }

    public override Vector3? CalculateObjectRotation(int index, int count) =>
      transform.rotation.eulerAngles;

    protected override void OnUpdateObjectLayout()
    {
      if (_isOpen)
      {
        _scrollbar.gameObject.SetActive(Objects.Count > WindowSize());
        _scrollbar.size = (float)WindowSize() / Objects.Count;
        _scrollAmount = _scrollbar.value;
        ApplyLayout();
      }
      else
      {
        _scrollbar.gameObject.SetActive(false);
      }
    }

    protected override int SortingOrder(int index, int count)
    {
      // If all objects fit in view, sort by index (higher index = lower priority)
      if (count <= WindowSize())
      {
        return count - index - 1;
      }

      // Calculate the maximum scroll offset
      var maxScrollOffset = count - WindowSize();

      // Calculate the current scroll offset based on _scrollAmount
      var currentScrollOffset = Mathf.Clamp01(_scrollAmount) * maxScrollOffset;

      // Calculate the effective index with scrolling applied
      var effectiveIndex = index - currentScrollOffset;

      // Determine if the object is visible (within the window)
      var isVisible = effectiveIndex >= 0 && effectiveIndex < WindowSize();

      if (isVisible)
      {
        // Visible objects get highest priority (count-1 down to count-WindowSize)
        // Objects closer to the front of the window get higher priority
        return count - 1 - Mathf.FloorToInt(effectiveIndex);
      }
      else if (effectiveIndex < 0)
      {
        // Objects before the window get medium-low priority
        // Further off-screen = lower priority
        return Mathf.Max(0, Mathf.FloorToInt(count - WindowSize() - 1 + effectiveIndex));
      }
      else
      {
        // Objects after the window get lowest priority
        // Further off-screen = lower priority
        var positionsAfterWindow = effectiveIndex - WindowSize();
        return Mathf.Max(0, Mathf.FloorToInt(count - WindowSize() - 1 - positionsAfterWindow));
      }
    }

    float SmoothedNormalizedPosition(int index, int count, float scrollAmount)
    {
      if (count <= WindowSize())
      {
        return ScrolledNormalizedPosition(index, count, scrollAmount);
      }

      var maxScrollOffset = count - WindowSize();
      var currentScrollPosition = scrollAmount * maxScrollOffset;
      var intScrollPosition = Mathf.FloorToInt(currentScrollPosition);
      var fraction = currentScrollPosition - intScrollPosition;

      var currentPosition = CalculateNormalizedPositionAtScroll(index, count, intScrollPosition);

      if (intScrollPosition < maxScrollOffset)
      {
        var nextPosition = CalculateNormalizedPositionAtScroll(index, count, intScrollPosition + 1);
        return Mathf.Lerp(currentPosition, nextPosition, fraction);
      }

      return currentPosition;
    }

    float CalculateNormalizedPositionAtScroll(int index, int count, int scrollOffset)
    {
      var scrollAmount = (float)scrollOffset / (count - WindowSize());
      return ScrolledNormalizedPosition(index, count, scrollAmount);
    }

    float ScrolledNormalizedPosition(int index, int count, float scrollAmount)
    {
      if (count <= WindowSize())
      {
        return NormalizedObjectPosition(index, count);
      }

      var maxScrollOffset = count - WindowSize();
      var currentScrollOffset = scrollAmount * maxScrollOffset;
      var effectiveIndex = index - currentScrollOffset;

      if (effectiveIndex < 0)
      {
        return NormalizedObjectPosition(0, count);
      }
      else if (effectiveIndex < WindowSize())
      {
        return NormalizedObjectPosition(Mathf.FloorToInt(effectiveIndex), count);
      }
      else
      {
        return NormalizedObjectPosition(Mathf.Min(index, count - 1), count);
      }
    }

    int WindowSize() => Mathf.Max(1, Mathf.FloorToInt(TotalWidth() / _cardWidth));

    float NormalizedObjectPosition(int index, int count)
    {
      if (count <= 0)
      {
        return 0.5f;
      }

      var maxObjectsInView = WindowSize();
      var totalWidth = TotalWidth();

      if (index < maxObjectsInView)
      {
        var objectsInView = Mathf.Min(maxObjectsInView, count);
        var neededWidth = (objectsInView - 1) * _cardWidth;
        var startOffset = (totalWidth - neededWidth) / 2f;
        var positionOffset = startOffset + (index * _cardWidth);
        return positionOffset / totalWidth;
      }
      else
      {
        int overflowCount = count - maxObjectsInView;
        float stackPosition = (float)(index - maxObjectsInView) / overflowCount;
        float stackOffset = stackPosition * _maxStackOffsetRight;

        var neededWidth = (maxObjectsInView - 1) * _cardWidth;
        var startOffset = (totalWidth - neededWidth) / 2f;
        var positionOffset = startOffset + ((maxObjectsInView - 1) * _cardWidth) + stackOffset;
        return Mathf.Clamp01(positionOffset / totalWidth);
      }
    }
  }
}
