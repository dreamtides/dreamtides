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
    /// <summary>
    /// The objects in this ObjectLayout
    /// </summary>
    public abstract IReadOnlyList<Displayable> Objects { get; }

    /// <summary>Adds an object to be owned by this ObjectLayout</summary>
    public abstract void Add(Displayable displayable);

    /// <summary>Adds a range of objects to this ObjectLayout</summary>
    public abstract void AddRange(IEnumerable<Displayable> displayables);

    /// <summary>Tries to remove an object from this ObjectLayout</summary>
    public abstract void RemoveIfPresent(Displayable? displayable);

    /// <summary>
    /// Applies the transform state at which objects should be created or
    /// destroyed for this layout. If a sequence applied, the transformation
    /// will be animated.
    /// </summary>
    public abstract void ApplyTargetTransform(Displayable target, Sequence? sequence = null);

    /// <summary>
    /// Applie this layout to its children. If a sequence is provied, inserts a
    /// series of animations to move this layout's children to their expected
    /// position, rotation, and scale. Otherwise they are immediately set to
    /// their target values.
    /// </summary>
    public abstract void ApplyLayout(Sequence? sequence = null);
  }
}
