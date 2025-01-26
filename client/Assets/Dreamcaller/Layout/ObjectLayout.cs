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
    private readonly List<Displayable> _objects = new();

    /// <summary>The objects in this ObjectLayout</summary>
    public IReadOnlyList<Displayable> Objects => _objects;

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
    /// Gets the position at which objects should be created or destroyed for
    /// this layout.
    /// </summary>
    public virtual Vector3 GetTargetPosition() => transform.position;

    /// <summary>
    /// Inserts a series of animations into a Sequence to move this layout's
    /// children to their expected position, rotation, and scale.
    /// </summary>
    public void InsertAnimationSequence(Sequence sequence)
    {
      const float duration = TweenUtils.MoveAnimationDurationSeconds;

      for (var i = 0; i < _objects.Count; ++i)
      {
        var displayable = _objects[i];
        var position = CalculateObjectPosition(i, _objects.Count);
        var rotation = CalculateObjectRotation(i, _objects.Count);
        var scale = CalculateObjectScale(i, _objects.Count) ?? displayable.DefaultScale;

        if (displayable is ObjectLayout layout)
        {
          /// If this is a child layout, recursively animate its contained elements.
          layout.transform.position = position;
          if (rotation is { } r)
          {
            displayable.transform.localEulerAngles = r;
          }
          layout.transform.localScale = Vector3.one * scale;
          layout.InsertAnimationSequence(sequence);
          continue;
        }

        if (IsEquivalent(displayable, position, rotation, scale))
        {
          continue;
        }

        sequence.Insert(atPosition: 0, displayable.transform.DOMove(position, duration));

        if (rotation is { } vector)
        {
          sequence.Insert(atPosition: 0,
            displayable.transform.DOLocalRotate(vector, duration));
        }

        sequence.Insert(atPosition: 0,
          displayable.transform.DOScale(Vector3.one * scale, duration));
      }
    }

    protected abstract Vector3 CalculateObjectPosition(int index, int count);

    protected virtual Vector3? CalculateObjectRotation(int index, int count) => null;

    protected virtual float? CalculateObjectScale(int index, int count) => null;

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
