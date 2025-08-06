#nullable enable

using Dreamtides.Components;
using UnityEngine;

namespace Dreamtides.Layout
{
  public class EnemyHandLayout : LandscapeHandLayout
  {
    public override Vector3? CalculateObjectRotation(int index, int count)
    {
      var baseRotation = base.CalculateObjectRotation(index, count);
      if (baseRotation is { } rotation)
      {
        return new Vector3(
          x: rotation.x,
          y: rotation.y,
          z: rotation.z + 180f
        );
      }
      else
      {
        return null;
      }
    }

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var basePosition = base.CalculateObjectPosition(index, count);
      if (index < Objects.Count && Objects[index] is Card card && card.NullableCardView?.Revealed != null)
      {
        // Slightly elevate revealed cards in enemy hand in order to prevent
        // ordering issues with card backs, since card backs are not sprites.
        return new Vector3(
          x: basePosition.x,
          y: basePosition.y + 0.25f,
          z: basePosition.z - 0.25f
        );
      }
      else
      {
        return basePosition;
      }
    }
  }
}