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
  /// movement with local transforms, automatically parenting children
  /// to this layout's transform.
  /// </summary>
  public abstract class RenderAsChildObjectLayout : ObjectLayout
  {
    [SerializeField]
    List<Displayable> _objects = new();

    [SerializeField]
    internal bool _debugUpdateContinuously = false;

    bool _shouldFireBecameNonEmptyAfterNextLayoutAnimation;

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

      var wasEmpty = _objects.Count == 0;

      if (!_objects.Contains(displayable))
      {
        if (displayable.Parent)
        {
          displayable.Parent.RemoveIfPresent(displayable);
        }

        displayable.Parent = this;
        displayable.transform.SetParent(transform, worldPositionStays: true);
        _objects.Add(displayable);
      }

      if (!displayable.ExcludeFromLayout)
      {
        displayable.GameContext = GameContext;
      }

      SortObjects();

      if (wasEmpty && Objects.Count > 0)
      {
        _shouldFireBecameNonEmptyAfterNextLayoutAnimation = true;
      }
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
        if (displayable.transform.parent == transform)
        {
          displayable.transform.SetParent(null, worldPositionStays: true);
        }
        _objects.Remove(displayable);
        SortObjects();
        if (_objects.Count == 0)
        {
          OnBecameEmpty();
        }
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
      if (_objects.Count == 0)
      {
        OnBecameEmpty();
      }
    }

    /// <summary>
    /// Applies the transform state at which objects should be created or
    /// destroyed for this layout. If a sequence applied, the transformation
    /// will be animated.
    /// </summary>
    public override void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      OnBeforeApplyLayout();
      ApplyLayoutToObject(target, _objects.Count, _objects.Count + 1, sequence);
    }

    /// <summary>
    /// Applies this layout to its children using LOCAL transforms. If a sequence is
    /// provided, we animate; otherwise we set transforms immediately.
    /// </summary>
    public override void ApplyLayout(Sequence? sequence = null)
    {
      OnBeforeApplyLayout();
      for (var i = 0; i < _objects.Count; ++i)
      {
        var layoutIndex = GetLayoutIndexOverride(_objects[i], i, _objects.Count);
        ApplyLayoutToObject(_objects[i], layoutIndex, _objects.Count, sequence);
      }

      if (_shouldFireBecameNonEmptyAfterNextLayoutAnimation)
      {
        if (sequence != null)
        {
          sequence.AppendCallback(() =>
          {
            _shouldFireBecameNonEmptyAfterNextLayoutAnimation = false;
            OnBecameNonEmpty();
          });
        }
        else
        {
          _shouldFireBecameNonEmptyAfterNextLayoutAnimation = false;
          OnBecameNonEmpty();
        }
      }

      OnAppliedLayout();
    }

    /// <summary>
    /// Invoked before applying the layout to the objects.
    /// </summary>
    protected virtual void OnBeforeApplyLayout() { }

    /// <summary>
    /// Invoked after applying the layout to the objects.
    /// </summary>
    protected virtual void OnAppliedLayout() { }

    /// <summary>
    /// Invoked after the 'add objects' animation completes when the count of
    /// objects in this layout changes from being zero to being nonzero.
    /// </summary>
    protected virtual void OnBecameNonEmpty() { }

    /// <summary>
    /// Invoked after removing objects when the count of objects in this layout
    /// changes from being nonzero to being zero.
    /// </summary>
    protected virtual void OnBecameEmpty() { }

    /// <summary>
    /// Calculates the LOCAL position of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object positions.
    public abstract Vector3 CalculateObjectLocalPosition(int index, int count);

    /// <summary>
    /// Calculates the LOCAL rotation of the object at the given index in the layout.
    /// Returning Vector3.zero means the child matches this layout's rotation in world space.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object rotations.
    public virtual Vector3? CalculateObjectLocalRotation(int index, int count) => null;

    /// <summary>
    /// Calculates the LOCAL scale of the object at the given index.
    /// Returning 1.0f here causes world scale to match the layout's scale.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object scales.
    public virtual float? CalculateObjectLocalScale(int index, int count) => null;

    /// <summary>
    /// Override the layout index for the given object.
    /// </summary>
    protected virtual int GetLayoutIndexOverride(Displayable displayable, int index, int count) =>
      index;

    protected sealed override void OnUpdate()
    {
      if (_debugUpdateContinuously)
      {
        ApplyLayout();
      }

      OnUpdateObjectLayout();
    }

    protected virtual void OnUpdateObjectLayout() { }

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
      var localPosition = CalculateObjectLocalPosition(index, count);
      var localRotation = CalculateObjectLocalRotation(index, count);
      var localScale = CalculateObjectLocalScale(index, count) ?? displayable.DefaultScale;

      if (displayable.transform.parent != transform)
      {
        displayable.transform.SetParent(transform, worldPositionStays: true);
      }

      if (applyToChildren && displayable is ObjectLayout layout)
      {
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
  }
}
