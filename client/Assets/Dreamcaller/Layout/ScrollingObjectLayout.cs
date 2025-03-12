#nullable enable

using UnityEngine;

namespace Dreamcaller.Layout
{
  public class ScrollingObjectLayout : StandardObjectLayout
  {
    [SerializeField] float _cardWidth = 2.5f;
    [SerializeField] Transform _leftEdge = null!;
    [SerializeField] Transform _rightEdge = null!;
    [SerializeField] float _scrollAmount;

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      return new Vector3(ScrolledXOffset(index, count), transform.position.y, transform.position.z);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

    /// <summary>
    /// Returns the x offset for an object at a given index in a list of
    /// objects, shifting to smaller x coordinates as the _scrollAmount
    /// increases.
    /// </summary>
    float ScrolledXOffset(int index, int count)
    {
      if (count <= WindowSize())
      {
        // If all objects fit in view, no scrolling needed
        return ObjectXOffset(index, count);
      }

      // Calculate the maximum scroll offset (number of objects that can be scrolled)
      int maxScrollOffset = count - WindowSize();

      // Calculate the current scroll offset based on _scrollAmount
      float currentScrollOffset = _scrollAmount * maxScrollOffset;

      // Calculate the effective index with scrolling applied
      float effectiveIndex = index - currentScrollOffset;

      // Handle objects that are scrolled off to the left (before view)
      if (effectiveIndex < 0)
      {
        return ObjectXOffset(0, count);
      }
      // Handle objects that are in view
      else if (effectiveIndex < count)
      {
        return ObjectXOffset(Mathf.FloorToInt(effectiveIndex), count);
      }
      // This shouldn't happen, but handle it just in case
      else
      {
        return ObjectXOffset(count - 1, count);
      }
    }

    /// <summary>
    /// Returns the total width of the layout.
    /// </summary>
    float TotalWidth() => _rightEdge.position.x - _leftEdge.position.x;

    /// <summary>
    /// Returns the maximum number of objects that can be displayed.
    /// </summary>
    int WindowSize() => Mathf.Max(1, Mathf.FloorToInt(TotalWidth() / _cardWidth));

    /// <summary>
    /// Calculate the x offset for an object at a given index in a list of
    /// objects. If 'count' is larger than the number of objects that will fit,
    /// places all remaining objects at the position of the last visible object.
    /// </summary>
    float ObjectXOffset(int index, int count)
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
        var startX = _leftEdge.position.x + (TotalWidth() - neededWidth) / 2;
        // Calculate the position for this specific object
        return startX + (index * _cardWidth);
      }
      else
      {
        // All overflow objects share the position of the last visible object
        var neededWidth = (maxObjectsInView - 1) * _cardWidth;
        var startX = _leftEdge.position.x + (TotalWidth() - neededWidth) / 2;
        return startX + ((maxObjectsInView - 1) * _cardWidth);
      }
    }

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