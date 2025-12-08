#nullable enable

using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using DG.Tweening;
using Dreamtides.Utils;
using UnityEngine;

[assembly: InternalsVisibleTo("Dreamtides.Tests")]

namespace Dreamtides.Layout
{
  /// <summary>
  /// A layout that renders/animates its objects as transform children,
  /// using local position/rotation/scale instead of world space.
  ///
  /// This mirrors the behavior of StandardObjectLayout but applies
  /// movement with local transforms, and provides pile-like defaults
  /// for position/rotation/scale in local space.
  /// </summary>
  public sealed class RenderAsChildObjectLayout : ObjectLayout
  {
    [SerializeField]
    List<Displayable> _objects = new();

    [SerializeField]
    internal bool _debugUpdateContinuously = false;

    // Pile-like tuning (applied as LOCAL offsets relative to this transform)
    [SerializeField]
    internal float _singleElementY = 0.5f;

    [SerializeField]
    internal float _yMultiplier = 1.0f;

    /// <summary>
    /// If true, the layout will update continuously.
    /// </summary>
    public bool DebugUpdateContinuously
    {
      get => _debugUpdateContinuously;
      set => _debugUpdateContinuously = value;
    }

    /// <summary>The objects in this ObjectLayout</summary>
    public override IReadOnlyList<Displayable> Objects => _objects.AsReadOnly();

    /// <summary>Adds an object to be owned by this ObjectLayout</summary>
    public override void Add(Displayable displayable)
    {
      Errors.CheckNotNull(displayable);

      if (!_objects.Contains(displayable))
      {
        if (displayable.Parent)
        {
          displayable.Parent.RemoveIfPresent(displayable);
        }

        displayable.Parent = this;
        // Parent under this transform, preserve world space initially so we can animate to local target.
        displayable.transform.SetParent(transform, worldPositionStays: true);
        _objects.Add(displayable);
      }

      if (!displayable.ExcludeFromLayout)
      {
        displayable.GameContext = GameContext;
      }

      SortObjects();
    }

    /// <summary>Adds a range of objects to this ObjectLayout</summary>
    public override void AddRange(IEnumerable<Displayable> displayables) =>
      displayables.ToList().ForEach(Add);

    /// <summary>Tries to remove an object from this ObjectLayout</summary>
    public override void RemoveIfPresent(Displayable? displayable)
    {
      if (displayable)
      {
        displayable.Parent = null;
        // Detach from this transform if currently parented here
        if (displayable.transform.parent == transform)
        {
          displayable.transform.SetParent(null, worldPositionStays: true);
        }
        _objects.Remove(displayable);
        SortObjects();
      }
    }

    /// <summary>Removes the object at the given index.</summary>
    public void RemoveAtIndex(int index)
    {
      var displayable = _objects[index];
      displayable.Parent = null;
      if (displayable.transform.parent == transform)
      {
        displayable.transform.SetParent(null, worldPositionStays: true);
      }
      _objects.RemoveAt(index);
      SortObjects();
    }

    /// <summary>
    /// Applies the transform state at which objects should be created or
    /// destroyed for this layout. If a sequence applied, the transformation
    /// will be animated.
    /// </summary>
    public override void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      ApplyLayoutToObject(target, _objects.Count, _objects.Count + 1, sequence);
    }

    /// <summary>
    /// Applies this layout to its children using LOCAL transforms. If a sequence is
    /// provided, we animate; otherwise we set transforms immediately.
    /// </summary>
    public override void ApplyLayout(Sequence? sequence = null)
    {
      for (var i = 0; i < _objects.Count; ++i)
      {
        ApplyLayoutToObject(_objects[i], i, _objects.Count, sequence);
      }
    }

    /// <summary>
    /// Calculates the LOCAL position of the object at the given index in the layout.
    /// Defaults to a pile-like depth (Z-axis) distribution relative to this transform.
    /// </summary>
    Vector3 CalculateObjectPosition(int index, int count) =>
      new(0f, 0f, _yMultiplier * Mathf.Lerp(0f, 1f, YPosition(index, count)));

    /// <summary>
    /// Calculates the LOCAL rotation of the object at the given index in the layout.
    /// Returning Vector3.zero means the child matches this layout's rotation in world space.
    /// </summary>
    Vector3? CalculateObjectRotation(int index, int count) => Vector3.zero;

    /// <summary>
    /// Calculates the LOCAL scale of the object at the given index.
    /// Returning 1.0f here causes world scale to match the layout's scale.
    /// </summary>
    float? CalculateObjectScale(int index, int count) => 1.0f;

    protected override void OnUpdate()
    {
      if (_debugUpdateContinuously)
      {
        ApplyLayout();
      }
    }

    void ApplyLayoutToObject(
      Displayable displayable,
      int index,
      int count,
      Sequence? sequence = null,
      bool applyToChildren = true
    )
    {
      if (displayable.ExcludeFromLayout)
      {
        return;
      }

      const float duration = TweenUtils.MoveAnimationDurationSeconds;
      var localPosition = CalculateObjectPosition(index, count);
      var localRotation = CalculateObjectRotation(index, count);
      var localScale = CalculateObjectScale(index, count) ?? displayable.DefaultScale;

      // Ensure parenting under this layout
      if (displayable.transform.parent != transform)
      {
        displayable.transform.SetParent(transform, worldPositionStays: true);
      }

      if (applyToChildren && displayable is ObjectLayout layout)
      {
        // If this is a child layout, recursively animate its contained elements.
        ApplyLayoutToObject(layout, index, count, sequence: null, applyToChildren: false);
        if (sequence != null)
        {
          layout.ApplyLayout(sequence);
        }
        else
        {
          foreach (var child in layout.Objects)
          {
            ApplyLayoutToObject(child, index, count);
          }
        }
        return;
      }

      if (IsEquivalent(displayable, localPosition, localRotation, localScale))
      {
        return;
      }

      if (sequence != null)
      {
        sequence.Insert(atPosition: 0, displayable.transform.DOLocalMove(localPosition, duration));
      }
      else
      {
        displayable.transform.localPosition = localPosition;
      }

      if (localRotation is { } euler)
      {
        if (sequence != null)
        {
          sequence.Insert(atPosition: 0, displayable.transform.DOLocalRotate(euler, duration));
        }
        else
        {
          displayable.transform.localEulerAngles = euler;
        }
      }

      if (sequence != null)
      {
        sequence.Insert(
          atPosition: 0,
          displayable.transform.DOScale(Vector3.one * localScale, duration)
        );
      }
      else
      {
        displayable.transform.localScale = Vector3.one * localScale;
      }
    }

    bool IsEquivalent(
      Displayable displayable,
      Vector3 localPosition,
      Vector3? localRotation,
      float localScale
    )
    {
      if (Vector3.Distance(displayable.transform.localPosition, localPosition) > 0.01f)
      {
        return false;
      }

      if (
        localRotation != null
        && Vector3.Distance(
          EulerAngleDistance(displayable.transform.localEulerAngles, localRotation.Value),
          Vector3.zero
        ) > 0.01f
      )
      {
        return false;
      }

      if (Vector3.Distance(displayable.transform.localScale, localScale * Vector3.one) > 0.01f)
      {
        return false;
      }

      return true;
    }

    void SortObjects()
    {
      _objects.Sort((a, b) => a.SortingKey.CompareTo(b.SortingKey));
    }

    Vector3 EulerAngleDistance(Vector3 a, Vector3 b) =>
      new(Mathf.DeltaAngle(a.x, b.x), Mathf.DeltaAngle(a.y, b.y), Mathf.DeltaAngle(a.z, b.z));

    float YPosition(int index, int count) =>
      count switch
      {
        _ when index >= count => 0.65f,
        0 => _singleElementY,
        1 => _singleElementY,
        2 => new[] { 0.4f, 0.6f }[index],
        3 => new[] { 0.4f, 0.5f, 0.6f }[index],
        4 => new[] { 0.40f, 0.45f, 0.50f, 0.55f }[index],
        5 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f }[index],
        6 => new[] { 0.40f, 0.45f, 0.50f, 0.55f, 0.6f, 0.65f }[index],
        _ => index / ((float)count - 1),
      };
  }
}
