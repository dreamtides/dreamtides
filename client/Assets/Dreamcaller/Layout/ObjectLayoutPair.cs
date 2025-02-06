#nullable enable

using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class ObjectLayoutPair : ObjectLayout
  {
    [SerializeField] ObjectLayout _layout1 = null!;
    [SerializeField] ObjectLayout _layout2 = null!;
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
      var totalObjects = Objects.Count;

      // If we haven't hit the threshold yet, add to layout1
      if (totalObjects < _useSecondLayoutAfter)
      {
        _layout1.Add(displayable);
        return;
      }

      // Add to the layout with fewer objects, preferring layout1 if equal
      if (_layout1.Objects.Count <= _layout2.Objects.Count)
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
      // Apply transform using layout1 by default
      _layout1.ApplyTargetTransform(target, sequence);
    }

    public override void RemoveIfPresent(Displayable? displayable)
    {
      if (displayable == null) return;

      _layout1.RemoveIfPresent(displayable);
      _layout2.RemoveIfPresent(displayable);

      RebalanceLayouts();
    }


    void RebalanceLayouts()
    {
      var totalObjects = Objects.Count;

      // If we're under the threshold, move everything to layout1
      if (totalObjects <= _useSecondLayoutAfter)
      {
        foreach (var obj in _layout2.Objects.ToList())
        {
          _layout2.RemoveIfPresent(obj);
          _layout1.Add(obj);
        }
        return;
      }

      // Calculate target sizes for balanced distribution
      var targetLayout1Size = (totalObjects + 1) / 2; // Rounds up for odd numbers
      var targetLayout2Size = totalObjects / 2; // Rounds down for odd numbers

      // Move objects between layouts to achieve balance
      while (_layout1.Objects.Count > targetLayout1Size)
      {
        var obj = _layout1.Objects[0];
        _layout1.RemoveIfPresent(obj);
        _layout2.Add(obj);
      }

      while (_layout2.Objects.Count > targetLayout2Size)
      {
        var obj = _layout2.Objects[0];
        _layout2.RemoveIfPresent(obj);
        _layout1.Add(obj);
      }
    }
  }
}