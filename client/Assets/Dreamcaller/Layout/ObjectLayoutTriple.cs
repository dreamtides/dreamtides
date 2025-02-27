#nullable enable

using System.Collections.Generic;
using DG.Tweening;
using Mono.Cecil.Cil;
using UnityEngine;

namespace Dreamcaller.Layout
{
  public class ObjectLayoutTriple : ObjectLayout
  {
    [SerializeField] StandardObjectLayout _layout1 = null!;
    [SerializeField] StandardObjectLayout _layout2 = null!;
    [SerializeField] StandardObjectLayout _layout3 = null!;
    [SerializeField] int _useSecondLayoutAfter;
    [SerializeField] int _useThirdLayoutAfter;

    public override IReadOnlyList<Displayable> Objects
    {
      get
      {
        var objects = new List<Displayable>();
        objects.AddRange(_layout1.Objects);
        objects.AddRange(_layout2.Objects);
        objects.AddRange(_layout3.Objects);
        return objects.AsReadOnly();
      }
    }

    public override void Add(Displayable displayable)
    {
      if (Objects.Count <= _useSecondLayoutAfter)
      {
        _layout1.Add(displayable);
      }
      else if (Objects.Count <= _useThirdLayoutAfter)
      {
        _layout2.Add(displayable);
      }
      else
      {
        _layout3.Add(displayable);
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
      _layout3.ApplyLayout(sequence);
      SetSortingKeys();
    }

    public override void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      if (Objects.Count <= _useSecondLayoutAfter)
      {
        _layout1.ApplyTargetTransform(target, sequence);
      }
      else if (Objects.Count <= _useThirdLayoutAfter)
      {
        _layout2.ApplyTargetTransform(target, sequence);
      }
      else
      {
        _layout3.ApplyTargetTransform(target, sequence);
      }
      SetSortingKeys();
    }

    public override void RemoveIfPresent(Displayable? displayable)
    {
      if (displayable == null)
      {
        return;
      }

      _layout1.RemoveIfPresent(displayable);
      _layout2.RemoveIfPresent(displayable);
      _layout3.RemoveIfPresent(displayable);

      RebalanceLayouts();
    }

    void SetSortingKeys()
    {
      for (var i = 0; i < Objects.Count; ++i)
      {
        var obj = Objects[i];
        if (obj.SortingGroup)
        {
          obj.SortingGroup.sortingOrder = i;
        }
      }
    }

    void RebalanceLayouts()
    {
      var totalObjects = Objects.Count;

      // Calculate target sizes based on thresholds
      int targetLayout1Size, targetLayout2Size;

      if (totalObjects <= _useSecondLayoutAfter)
      {
        targetLayout1Size = totalObjects;
        targetLayout2Size = 0;
      }
      else if (totalObjects <= _useThirdLayoutAfter)
      {
        targetLayout1Size = totalObjects / 2;
        targetLayout2Size = totalObjects - targetLayout1Size;
      }
      else
      {
        targetLayout1Size = totalObjects / 3;
        targetLayout2Size = totalObjects / 3;
      }

      // Move objects between layout1 and layout2
      while (_layout1.Objects.Count > targetLayout1Size)
      {
        // Move from end of layout1 to start of layout2
        var lastIndex = _layout1.Objects.Count - 1;
        var obj = _layout1.Objects[lastIndex];
        _layout1.RemoveAtIndex(lastIndex);
        _layout2.Add(obj);
      }
      while (_layout1.Objects.Count < targetLayout1Size && _layout2.Objects.Count > 0)
      {
        // Move from start of layout2 to end of layout1
        var obj = _layout2.Objects[0];
        _layout2.RemoveAtIndex(0);
        _layout1.Add(obj);
      }

      // Move objects between layout2 and layout3
      while (_layout2.Objects.Count > targetLayout2Size)
      {
        // Move from end of layout2 to start of layout3
        var lastIndex = _layout2.Objects.Count - 1;
        var obj = _layout2.Objects[lastIndex];
        _layout2.RemoveAtIndex(lastIndex);
        _layout3.Add(obj);
      }
      while (_layout2.Objects.Count < targetLayout2Size && _layout3.Objects.Count > 0)
      {
        // Move from start of layout3 to end of layout2
        var obj = _layout3.Objects[0];
        _layout3.RemoveAtIndex(0);
        _layout2.Add(obj);
      }
    }
  }
}