#nullable enable

using DG.Tweening;
using Dreamcaller.Services;
using UnityEngine;
using UnityEngine.UI;

namespace Dreamcaller.Layout
{
  public class ScrollingObjectLayout : StandardObjectLayout
  {
    [SerializeField] float _cardWidth = 2.5f;
    [SerializeField] Transform _leftEdge = null!;
    [SerializeField] Transform _rightEdge = null!;
    [SerializeField] float _scrollAmount;
    [SerializeField] Scrollbar _scrollbar = null!;
    [SerializeField] bool _isOpen;
    [SerializeField] bool _zAxis;

    public bool IsOpen => _isOpen;

    public void Show(Registry registry, Sequence? sequence)
    {
      if (!_isOpen)
      {
        registry.Layout.BackgroundOverlay.Show(BackgroundOverlay.DisplayOver.Battlefield, 0.75f, sequence);
        if (sequence != null)
        {
          sequence.AppendCallback(() => _isOpen = true);
        }
        else
        {
          _isOpen = true;
        }
      }
    }

    public void Hide(Registry registry, Sequence? sequence)
    {
      if (_isOpen)
      {
        registry.Layout.BackgroundOverlay.Hide(sequence);
        if (sequence != null)
        {
          sequence.AppendCallback(() => _isOpen = false);
        }
        else
        {
          _isOpen = false;
        }
      }
    }

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      // In landscape mode we *decrease* Z positions as we move towards screen
      // right, which in retrospect was stupid. So now we have to invert
      // everything to scroll on the Z axis.
      return new Vector3(
        _zAxis ? transform.position.x : SmoothedOffset(index, count, Mathf.Clamp01(_scrollAmount)),
        transform.position.y - YOffset(index, count, Mathf.Clamp01(_zAxis ? 1 - _scrollAmount : _scrollAmount)),
        _zAxis ? SmoothedOffset(index, count, Mathf.Clamp01(1 - _scrollAmount)) - TotalWidth() : transform.position.z);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    protected override void OnUpdate()
    {
      if (_isOpen)
      {
        _scrollbar.gameObject.SetActive(Objects.Count > WindowSize());
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
      var currentScrollOffset = Mathf.Clamp01(_zAxis ? 1 - _scrollAmount : _scrollAmount) * maxScrollOffset;

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

    float YOffset(float index, float count, float scrollAmount)
    {
      // If all objects fit in view, return transform.position.y (no offset)
      if (count <= WindowSize())
      {
        return 0;
      }

      // Calculate the maximum scroll offset
      var maxScrollOffset = count - WindowSize();

      // Calculate the current scroll offset based on _scrollAmount
      var currentScrollOffset = scrollAmount * maxScrollOffset;

      // Calculate the effective index with scrolling applied
      var effectiveIndex = index - currentScrollOffset;

      // Determine if the object is visible (within the window)
      var isVisible = effectiveIndex >= 0 && effectiveIndex < WindowSize();

      // Visible objects get higher Y positions (closer to 0 offset)
      // Non-visible objects get lower Y positions (closer to 1 offset)
      float yOffset;

      if (isVisible)
      {
        // Distribute visible objects evenly in the upper range (0 to 0.4)
        yOffset = 0.4f * (effectiveIndex / WindowSize());
      }
      else if (effectiveIndex < 0)
      {
        // Objects before the view window
        // Position is based on how far off-screen they are, clamped to max 1.0
        yOffset = 0.4f + Mathf.Min(0.6f, -effectiveIndex / WindowSize());
      }
      else
      {
        // Objects after the view window
        // Position is based on how far off-screen they are, clamped to max 1.0
        yOffset = 0.4f + Mathf.Min(0.6f, (effectiveIndex - WindowSize()) / WindowSize());
      }

      // Apply the offset to transform.position.y
      return yOffset;
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
      else if (effectiveIndex < count)
      {
        return ObjectOffset(Mathf.FloorToInt(effectiveIndex), count);
      }
      // This shouldn't happen, but handle it just in case
      else
      {
        return ObjectOffset(count - 1, count);
      }
    }

    /// <summary>
    /// Returns the total width of the layout.
    /// </summary>
    float TotalWidth() => Mathf.Abs(RightEdge() - LeftEdge());

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
        // All overflow objects share the position of the last visible object
        var neededWidth = (maxObjectsInView - 1) * _cardWidth;
        var startX = LeftEdge() + (TotalWidth() - neededWidth) / 2;
        return startX + ((maxObjectsInView - 1) * _cardWidth);
      }
    }

    float LeftEdge() => _zAxis ? _leftEdge.position.z : _leftEdge.position.x;

    float RightEdge() => _zAxis ? _rightEdge.position.z : _rightEdge.position.x;

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      if (_leftEdge)
      {
        Gizmos.DrawSphere(_leftEdge.position, radius: 1);
      }
      if (_rightEdge)
      {
        Gizmos.DrawSphere(_rightEdge.position, radius: 1);
      }
    }
  }
}