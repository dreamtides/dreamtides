#nullable enable

using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamcaller.Utils;
using UnityEngine;

namespace Dreamcaller.Layout
{
  /// <summary>
  /// An ObjectLayout stores a list of Displayable GameObjects and is able to
  /// compute suggested position, rotation, and scale values for them.
  /// </summary>
  public abstract class ObjectLayout : Displayable
  {
    [SerializeField] List<Displayable> _objects = new();
    [SerializeField] bool _debugUpdateContinuously = false;

    /// <summary>The objects in this ObjectLayout</summary>
    public IReadOnlyList<Displayable> Objects => _objects.AsReadOnly();

    /// <summary>Adds an object to be owned by this ObjectLayout</summary>
    public void Add(Displayable displayable)
    {
      Errors.CheckNotNull(displayable);

      if (!_objects.Contains(displayable))
      {
        if (displayable.Parent)
        {
          displayable.Parent.RemoveIfPresent(displayable);
        }

        displayable.Parent = this;
        _objects.Add(displayable);

        displayable.GameContext = GameContext;
      }
    }

    /// <summary>Adds a range of objects to this ObjectLayout</summary>
    public void AddRange(IEnumerable<Displayable> displayables) =>
      displayables.ToList().ForEach(Add);

    /// <summary>Tries to remove an object from this ObjectLayout</summary>
    public void RemoveIfPresent(Displayable? displayable)
    {
      if (displayable)
      {
        displayable.Parent = null;
        _objects.Remove(displayable);
      }
    }

    /// <summary>
    /// Applies the transform state at which objects should be created or
    /// destroyed for this layout. If a sequence applied, the transformation
    /// will be animated.
    /// </summary>
    public virtual void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      var index = _objects.Count == 0 ? 0 : _objects.Count - 1;
      ApplyLayout(target, index, sequence);
    }

    /// <summary>
    /// Inserts a series of animations into a Sequence to move this layout's
    /// children to their expected position, rotation, and scale.
    /// </summary>
    public void InsertAnimationSequence(Sequence sequence)
    {
      for (var i = 0; i < _objects.Count; ++i)
      {
        ApplyLayout(_objects[i], i, sequence);
      }
    }

    /// <summary>
    /// Calculates the position of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object positions.
    protected abstract Vector3 CalculateObjectPosition(int index, int count);

    /// <summary>
    /// Calculates the rotation of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object rotations.
    protected virtual Vector3? CalculateObjectRotation(int index, int count) => null;

    /// <summary>
    /// Calculates the scale of the object at the given index in the layout.
    /// </summary>
    ///
    /// Note that this may be invoked with index=0, count=0 to compute initial
    /// object scales.
    protected virtual float? CalculateObjectScale(int index, int count) => null;

    void Update()
    {
      if (_debugUpdateContinuously)
      {
        for (var i = 0; i < _objects.Count; ++i)
        {
          ApplyLayout(_objects[i], i);
        }
      }
    }

    void ApplyLayout(Displayable displayable, int i, Sequence? sequence = null, bool applyToChildren = true)
    {
      const float duration = TweenUtils.MoveAnimationDurationSeconds;
      var position = CalculateObjectPosition(i, _objects.Count);
      var rotation = CalculateObjectRotation(i, _objects.Count);
      var scale = CalculateObjectScale(i, _objects.Count) ?? displayable.DefaultScale;

      if (applyToChildren && displayable is ObjectLayout layout)
      {
        /// If this is a child layout, recursively animate its contained
        /// elements.
        ApplyLayout(layout, i, sequence: null, applyToChildren: false);
        if (sequence != null)
        {
          layout.InsertAnimationSequence(sequence);
        }
        else
        {
          foreach (var child in layout.Objects)
          {
            ApplyLayout(child, i);
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
          sequence.Insert(atPosition: 0,
            displayable.transform.DOLocalRotate(vector, duration));
        }
        else
        {
          displayable.transform.localEulerAngles = vector;
        }
      }

      if (sequence != null)
      {
        sequence.Insert(atPosition: 0, displayable.transform.DOScale(Vector3.one * scale, duration));
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

      if (rotation != null && Vector3.Distance(EulerAngleDistance(displayable.transform.localEulerAngles, rotation.Value), Vector3.zero) > 0.01f)
      {
        return false;
      }

      if (Vector3.Distance(displayable.transform.localScale, scale * Vector3.one) > 0.01)
      {
        return false;
      }

      return true;
    }

    Vector3 EulerAngleDistance(Vector3 a, Vector3 b) => new(
      Mathf.DeltaAngle(a.x, b.x),
      Mathf.DeltaAngle(a.y, b.y),
      Mathf.DeltaAngle(a.z, b.z));
  }
}
