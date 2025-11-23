#nullable enable

using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamtides.Utils;
using UnityEngine;

namespace Dreamtides.Layout
{
  /// <summary>
  /// An ObjectLayout implementation which manages a list of Displayables
  /// </summary>
  public abstract class StandardObjectLayout : ObjectLayout
  {
    [SerializeField]
    List<Displayable> _objects = new();

    [SerializeField]
    bool _debugUpdateContinuously = false;

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
    /// Applies this layout to its children. If a sequence is provied, inserts a
    /// series of animations to move this layout's children to their expected
    /// position, rotation, and scale. Otherwise they are immediately set to
    /// their target values.
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
    /// Calculates the position of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object positions.
    public abstract Vector3 CalculateObjectPosition(int index, int count);

    /// <summary>
    /// Calculates the rotation of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object rotations.
    public virtual Vector3? CalculateObjectRotation(int index, int count) => null;

    /// <summary>
    /// Calculates the scale of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object scales.
    public virtual float? CalculateObjectScale(int index, int count) => null;

    /// <summary>
    /// Calculates the sorting order of the object at the given index in the layout.
    /// </summary>
    protected virtual int SortingOrder(int index, int count) => index;

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
      var position = CalculateObjectPosition(index, count);
      var rotation = CalculateObjectRotation(index, count);
      var scale = CalculateObjectScale(index, count) ?? displayable.DefaultScale;

      if (applyToChildren && displayable is ObjectLayout layout)
      {
        /// If this is a child layout, recursively animate its contained
        /// elements.
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

      if (IsEquivalent(displayable, position, rotation, scale))
      {
        return;
      }

      if (sequence != null)
      {
        sequence.Insert(atPosition: 0, displayable.transform.DOMove(position, duration));
      }
      else
      {
        displayable.transform.position = position;
      }

      if (rotation is { } vector)
      {
        if (sequence != null)
        {
          sequence.Insert(atPosition: 0, displayable.transform.DOLocalRotate(vector, duration));
        }
        else
        {
          displayable.transform.localEulerAngles = vector;
        }
      }

      if (sequence != null)
      {
        sequence.Insert(
          atPosition: 0,
          displayable.transform.DOScale(Vector3.one * scale, duration)
        );
      }
      else
      {
        displayable.transform.localScale = Vector3.one * scale;
      }
    }

    bool IsEquivalent(Displayable displayable, Vector3 position, Vector3? rotation, float scale)
    {
      if (Vector3.Distance(displayable.transform.position, position) > 0.01)
      {
        return false;
      }

      if (
        rotation != null
        && Vector3.Distance(
          EulerAngleDistance(displayable.transform.localEulerAngles, rotation.Value),
          Vector3.zero
        ) > 0.01f
      )
      {
        return false;
      }

      if (Vector3.Distance(displayable.transform.localScale, scale * Vector3.one) > 0.01)
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
