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
    [SerializeField] internal float _scrollAmount;
    [SerializeField] internal Scrollbar _scrollbar = null!;
    [SerializeField] internal Button _closeButton = null!;
    [SerializeField] internal float _maxStackOffsetRight = 1f;
    [SerializeField] internal Transform _singleCardPosition = null!;

    public override void Show(Registry registry, Sequence? sequence)
    {
      _scrollbar.value = 1;
      _scrollAmount = 1;

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

      _closeButton.gameObject.SetActive(visible);
      return true;
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count == 1)
      {
        return _singleCardPosition.position;
      }

      return new Vector3(
        SmoothedOffset(index, count, Mathf.Clamp01(_scrollAmount)),
        transform.position.y,
        _leftEdge.position.z);
    }

    public override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    protected override void OnUpdate()
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

    /// <summary>
    /// Returns the x offset for an object with smoothing between positions.
    /// </summary>
    float SmoothedOffset(int index, int count, float scrollAmount)
    {
      if (count <= WindowSize())
      {
        // If all objects fit in view, no smoothing needed
        return ScrolledOffset(index, count, scrollAmount);
      }

      // Calculate the maximum scroll offset
      var maxScrollOffset = count - WindowSize();

      // Get the integer and fractional parts of the current scroll position
      var currentScrollPosition = scrollAmount * maxScrollOffset;
      var intScrollPosition = Mathf.FloorToInt(currentScrollPosition);
      var fraction = currentScrollPosition - intScrollPosition;

      // Calculate position at the current integer scroll position
      var currentPosition = CalculatePositionAtScroll(index, count, intScrollPosition);

      // Only calculate next position if we're not at the end of the scroll range
      if (intScrollPosition < maxScrollOffset)
      {
        // Calculate position at the next integer scroll position
        var nextPosition = CalculatePositionAtScroll(index, count, intScrollPosition + 1);

        // Return smoothed position by linear interpolation
        return Mathf.Lerp(currentPosition, nextPosition, fraction);
      }

      return currentPosition;
    }

    /// <summary>
    /// Helper method to calculate the position at a specific scroll offset.
    /// </summary>
    float CalculatePositionAtScroll(int index, int count, int scrollOffset)
    {
      // Set _scrollAmount to the normalized position for this scroll offset
      var scrollAmount = (float)scrollOffset / (count - WindowSize());
      return ScrolledOffset(index, count, scrollAmount);
    }

    /// <summary>
    /// Returns the x offset for an object at a given index in a list of
    /// objects, shifting to smaller x coordinates as the _scrollAmount
    /// increases.
    /// </summary>
    float ScrolledOffset(int index, int count, float scrollAmount)
    {
      if (count <= WindowSize())
      {
        // If all objects fit in view, no scrolling needed
        return ObjectOffset(index, count);
      }

      // Calculate the maximum scroll offset (number of objects that can be scrolled)
      var maxScrollOffset = count - WindowSize();

      // Calculate the current scroll offset based on _scrollAmount
      var currentScrollOffset = scrollAmount * maxScrollOffset;

      // Calculate the effective index with scrolling applied
      var effectiveIndex = index - currentScrollOffset;

      // Handle objects that are scrolled off to the left (before view)
      if (effectiveIndex < 0)
      {
        return ObjectOffset(0, count);
      }
      // Handle objects that are in view
      else if (effectiveIndex < WindowSize())
      {
        return ObjectOffset(Mathf.FloorToInt(effectiveIndex), count);
      }
      // Handle objects after the view
      else
      {
        return ObjectOffset(Mathf.Min(index, count - 1), count);
      }
    }

    /// <summary>
    /// Returns the maximum number of objects that can be displayed.
    /// </summary>
    int WindowSize() => Mathf.Max(1, Mathf.FloorToInt(TotalWidth() / _cardWidth));

    /// <summary>
    /// Calculate the x offset for an object at a given index in a list of
    /// objects. If 'count' is larger than the number of objects that will fit,
    /// places all remaining objects at the position of the last visible object.
    /// </summary>
    float ObjectOffset(int index, int count)
    {
      if (count <= 0)
      {
        return 0;
      }

      var maxObjectsInView = WindowSize();

      if (index < maxObjectsInView)
      {
        var objectsInView = Mathf.Min(maxObjectsInView, count);
        // Calculate the width needed for maximum objects that can fit
        var neededWidth = (objectsInView - 1) * _cardWidth;
        // Calculate the starting X position
        var startX = LeftEdge() + (TotalWidth() - neededWidth) / 2;
        // Calculate the position for this specific object
        return startX + (index * _cardWidth);
      }
      else
      {
        // For objects after the visible window, add a small stacking offset
        // Calculate how many objects are in the stack
        int overflowCount = count - maxObjectsInView;
        // Calculate relative position in the overflow stack (0 to 1)
        float stackPosition = (float)(index - maxObjectsInView) / overflowCount;
        // Maximum stacking offset
        // Apply offset based on position in stack
        float stackOffset = stackPosition * _maxStackOffsetRight;

        // Position at the right edge with the calculated stack offset
        var neededWidth = (maxObjectsInView - 1) * _cardWidth;
        var startX = LeftEdge() + (TotalWidth() - neededWidth) / 2;
        return startX + ((maxObjectsInView - 1) * _cardWidth) + stackOffset;
      }
    }
  }
}