#nullable enable

using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class ObjectLayoutPair : ObjectLayout
  {
    [SerializeField] StandardObjectLayout _layout1 = null!;
    [SerializeField] StandardObjectLayout _layout2 = null!;
    [SerializeField] int _useSecondLayoutAfter;

    public override IReadOnlyList<Displayable> Objects
    {
      get
      {
        var objects = new List<Displayable>();
        objects.AddRange(_layout1.Objects);
        objects.AddRange(_layout2.Objects);
        return objects.AsReadOnly();
      }
    }

    public override void Add(Displayable displayable)
    {
      if (Objects.Count < _useSecondLayoutAfter)
      {
        _layout1.Add(displayable);
      }
      else
      {
        _layout2.Add(displayable);
      }

      RebalanceLayouts();
    }

    public override void AddRange(IEnumerable<Displayable> displayables)
    {
      foreach (var displayable in displayables)
      {
        Add(displayable);
      }
    }

    public override void ApplyLayout(Sequence? sequence = null)
    {
      _layout1.ApplyLayout(sequence);
      _layout2.ApplyLayout(sequence);
    }

    public override void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      if (Objects.Count <= _useSecondLayoutAfter)
      {
        _layout1.ApplyTargetTransform(target, sequence);
      }
      else
      {
        _layout2.ApplyTargetTransform(target, sequence);
      }
    }

    public override void RemoveIfPresent(Displayable? displayable)
    {
      if (displayable == null)
      {
        return;
      }

      _layout1.RemoveIfPresent(displayable);
      _layout2.RemoveIfPresent(displayable);

      RebalanceLayouts();
    }

    void RebalanceLayouts()
    {
      var totalObjects = Objects.Count;
      var targetLayout1Size = totalObjects <= _useSecondLayoutAfter
        ? totalObjects  // If we're below threshold, everything goes in layout1
        : totalObjects / 2;  // Otherwise aim for an even split

      // Balance by moving objects between layouts
      while (_layout1.Objects.Count > targetLayout1Size)
      {
        // Move last object from layout1 to layout2
        var lastIndex = _layout1.Objects.Count - 1;
        var obj = _layout1.Objects[lastIndex];
        _layout1.RemoveAtIndex(lastIndex);
        _layout2.Add(obj);
      }

      while (_layout1.Objects.Count < targetLayout1Size && _layout2.Objects.Count > 0)
      {
        // Move first object from layout2 to layout1
        var obj = _layout2.Objects[0];
        _layout2.RemoveAtIndex(0);
        _layout1.Add(obj);
      }
    }
  }
}