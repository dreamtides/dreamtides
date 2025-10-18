#nullable enable

using UnityEngine;

namespace Dreamtides.Layout
{
  public sealed class DreamsignDisplayLayout : StandardObjectLayout
  {
    [SerializeField]
    float _horizontalSpacing;

    [SerializeField]
    float _verticalSpacing;

    [SerializeField]
    float _cardWidth;

    [SerializeField]
    float _cardHeight;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      if (count <= 0)
      {
        return transform.position;
      }

      var scale = transform.localScale.x;
      var hs = _horizontalSpacing * scale;
      var vs = _verticalSpacing * scale;

      var column = index / 2;
      var row = index % 2;

      var localX = -hs * column;
      var localY = count <= 1 ? 0f : (row == 0 ? vs / 2f : -vs / 2f);

      return transform.position + transform.right * localX + transform.up * localY;
    }

    public override Vector3? CalculateObjectRotation(int index, int count) => Vector3.zero;

    public override float? CalculateObjectScale(int index, int count) => 1.0f;

    protected override void OnAppliedLayout()
    {
      Debug.Log($"OnAppliedLayout {name} {Objects.Count}");
    }

    protected override void OnBeforeApplyLayout()
    {
      for (var i = 0; i < Objects.Count; ++i)
      {
        var displayable = Objects[i];
        if (displayable && displayable.transform.parent != transform)
        {
          displayable.transform.SetParent(transform, worldPositionStays: true);
        }
      }
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      var center = transform.position;
      var s = transform.localScale.x;
      var halfLayoutX = _cardWidth * s / 2f + _horizontalSpacing * s / 2f;
      var halfLayoutY = _cardHeight * s / 2f + _verticalSpacing * s / 2f;

      var right = transform.right;
      var upAxis = transform.up;

      Gizmos.DrawSphere(center, 0.15f * s);
      Gizmos.DrawSphere(center + (-right * halfLayoutX + upAxis * halfLayoutY), 0.15f * s);
      Gizmos.DrawSphere(center + (right * halfLayoutX + upAxis * halfLayoutY), 0.15f * s);
      Gizmos.DrawSphere(center + (-right * halfLayoutX - upAxis * halfLayoutY), 0.15f * s);
      Gizmos.DrawSphere(center + (right * halfLayoutX - upAxis * halfLayoutY), 0.15f * s);
    }
  }
}
