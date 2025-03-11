#nullable enable

using UnityEngine;

namespace Dreamcaller.Layout
{
  public class ScrollingObjectLayout : StandardObjectLayout
  {
    [SerializeField] float _cardWidth = 2.5f;
    [SerializeField] Transform _leftEdge = null!;
    [SerializeField] Transform _rightEdge = null!;

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count <= 0) return transform.position;

      // Calculate the total available width between edges
      float totalWidth = _rightEdge.position.x - _leftEdge.position.x;

      // Calculate how many objects can fit in the available space
      int maxObjectsInView = Mathf.Max(1, Mathf.FloorToInt(totalWidth / _cardWidth));

      // If we have fewer objects than can fit, center them
      if (count <= maxObjectsInView)
      {
        // Calculate the width needed for all objects
        float neededWidth = (count - 1) * _cardWidth;

        // Calculate the starting X position to center the objects
        float startX = _leftEdge.position.x + (totalWidth - neededWidth) / 2;

        // Calculate the position for this specific object
        float xPos = startX + (index * _cardWidth);

        return new Vector3(xPos, transform.position.y, transform.position.z);
      }
      else
      {
        // Handle overflow case - objects that don't fit share the rightmost position
        if (index < maxObjectsInView)
        {
          // Calculate the width needed for maximum objects that can fit
          float neededWidth = (maxObjectsInView - 1) * _cardWidth;

          // Calculate the starting X position
          float startX = _leftEdge.position.x + (totalWidth - neededWidth) / 2;

          // Calculate the position for this specific object
          float xPos = startX + (index * _cardWidth);

          return new Vector3(xPos, transform.position.y, transform.position.z);
        }
        else
        {
          // All overflow objects share the position of the last visible object
          float neededWidth = (maxObjectsInView - 1) * _cardWidth;
          float startX = _leftEdge.position.x + (totalWidth - neededWidth) / 2;
          float xPos = startX + ((maxObjectsInView - 1) * _cardWidth);

          return new Vector3(xPos, transform.position.y, transform.position.z);
        }
      }
    }

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;

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