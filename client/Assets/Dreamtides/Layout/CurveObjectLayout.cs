#nullable enable

using System;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Layout
{
  public class CurveObjectLayout : StandardObjectLayout
  {
    [SerializeField] int _zRotationAddition;
    [SerializeField] float _zRotationMultiplier;
    [SerializeField] Transform _controlPoint1 = null!;
    [SerializeField] Transform _controlPoint2 = null!;
    [SerializeField] Transform _controlPoint3 = null!;
    [SerializeField] Transform _controlPoint4 = null!;
    [SerializeField] float _gizmoRadius = 1.0f;
    [SerializeField] float _objectScale;
    [SerializeField] float _yRotation;
    [SerializeField] bool _portraitLayout;

    public override Vector3 CalculateObjectPosition(int index, int count)
    {
      var curvePosition = CalculateCurvePosition(index, count);
      var bezier = CalculateBezierPosition(curvePosition);
      return bezier;
    }

    public override Vector3? CalculateObjectRotation(int index, int count)
    {
      var curvePosition = CalculateCurvePosition(index, count);
      return new Vector3(
          x: Constants.CameraXAngle,
          y: _yRotation,
          z: _zRotationAddition + _zRotationMultiplier * CalculateZRotation(curvePosition));
    }

    public override float? CalculateObjectScale(int index, int count) => _objectScale == 0.0 ? null : _objectScale;

    float CalculateCurvePosition(int cardIndex, int cardCount)
    {
      if (cardCount == 0)
      {
        return 0.5f;
      }

      if (cardIndex < 0 || cardIndex >= cardCount)
      {
        throw new ArgumentException("Index out of bounds");
      }

      switch (cardCount)
      {
        case 1:
          return 0.5f;
        case 2:
          return new float[] { 0.3333f, 0.6666f }[cardIndex];
        case 3:
          return new float[] { 0.25f, 0.5f, 0.75f }[cardIndex];
        case 4:
          return new float[] { 0f, 0.3333f, 0.6666f, 1f }[cardIndex];
        // case 4:
        //   return new float[] { 0.2f, 0.4f, 0.6f, 0.8f }[cardIndex];
        case 5:
          return new float[] { 0f, 0.25f, 0.5f, 0.75f, 1f }[cardIndex];
        case 6:
          return new float[] { 0f, 0.2f, 0.4f, 0.6f, 0.8f, 1f }[cardIndex];
        //case 5 when _portraitLayout:
        // There's a lot of weird aesthetics going on here. The cards don't
        // feel right unless you can see the same sliver of each one, but with
        // rotation and stuff this doesn't seem to work with even spacing.
        // return new float[] { 0.15f, 0.35f, 0.50f, 0.65f, 0.85f }[cardIndex];
        // case 6 when _portraitLayout:
        // For these two, we kick the right control point over a bunch and
        // then pull the leftmost cards a little. Because you see so much more
        // of the "top" card, this gives the illusion of even distribution
        // even though the spacing itself is uneven. For this to work, the
        // cards really need to be flush with the screen edges -- if you can
        // see space around them you will realize they are not centered.
        // return new float[] { 0.06f, 0.28f, 0.42f, 0.56f, 0.7f, 0.86f }[cardIndex];
        // case 7 when _portraitLayout:
        // return new float[] { 0.06f, 0.26f, 0.40f, 0.52f, 0.64f, 0.76f, 0.88f }[cardIndex];
        default:
          return 0.1f + (0.8f * cardIndex / (cardCount - 1));
      }
    }

    float CalculateZRotation(float t) => -10.0f * t + 5.0f;

    Vector3 CalculateBezierPosition(float t) =>
      Mathf.Pow(1 - t, 3) * ControlPointPosition(1) +
      3 * Mathf.Pow(1 - t, 2) * t * ControlPointPosition(2) +
      3 * (1 - t) * Mathf.Pow(t, 2) * ControlPointPosition(3) +
      Mathf.Pow(t, 3) * ControlPointPosition(4);

    Vector3 ControlPointPosition(int index) =>
      index switch
      {
        1 => _controlPoint1.position,
        2 => _controlPoint2.position,
        3 => _controlPoint3.position,
        4 => _controlPoint4.position + Objects.Count switch
        {
          6 when _portraitLayout => new Vector3(1, 0, 0),
          >= 7 when _portraitLayout => new Vector3(2.5f, 0, 0),
          _ => Vector3.zero
        },
        _ => throw new ArgumentException("Invalid control point index"),
      };

    void OnDrawGizmosSelected()
    {
      for (var t = 0.0f; t <= 1; t += 0.05f)
      {
        var position = CalculateBezierPosition(t);
        Gizmos.DrawSphere(position, radius: _gizmoRadius);
      }

      Gizmos.color = Color.green;
      Gizmos.DrawSphere(_controlPoint1.position, radius: _gizmoRadius);
      Gizmos.DrawSphere(_controlPoint2.position, radius: _gizmoRadius);
      Gizmos.DrawSphere(_controlPoint3.position, radius: _gizmoRadius);
      Gizmos.DrawSphere(_controlPoint4.position, radius: _gizmoRadius);
    }
  }
}