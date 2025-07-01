#nullable enable

using System;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Dreamtides.Components;
using UnityEngine;

namespace Dreamtides.Layout
{
  public class UserHandLayout : ObjectLayout
  {
    [SerializeField] StandardObjectLayout _layout1 = null!;
    [SerializeField] StandardObjectLayout _layout2 = null!;
    [SerializeField] ScrollableUserHandLayout _scrollableHand = null!;
    [SerializeField] int _useSecondLayoutAfter;
    [SerializeField] int _useBrowserAfter;

    public override IReadOnlyList<Displayable> Objects
    {
      get
      {
        var objects = new List<Displayable>();
        objects.AddRange(_layout1.Objects);
        objects.AddRange(_layout2.Objects);
        objects.AddRange(_scrollableHand.Objects);
        return objects.AsReadOnly();
      }
    }

    public override void Add(Displayable displayable)
    {
      if (Objects.Count <= _useSecondLayoutAfter)
      {
        _layout1.Add(displayable);
      }
      else if (Objects.Count <= _useBrowserAfter)
      {
        _layout2.Add(displayable);
      }
      else
      {
        _scrollableHand.Add(displayable);
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
      _scrollableHand.ApplyLayout(sequence);
      SetSortingKeys();
    }

    public Vector3? CalculateObjectPosition(Card card)
    {
      if (_layout1.Objects.Contains(card))
      {
        return _layout1.CalculateObjectPosition(
          _layout1.Objects.ToList().IndexOf(card), _layout1.Objects.Count);
      }
      else if (_layout2.Objects.Contains(card))
      {
        return _layout2.CalculateObjectPosition(
          _layout2.Objects.ToList().IndexOf(card), _layout2.Objects.Count);
      }
      else if (_scrollableHand.Objects.Contains(card))
      {
        return _scrollableHand.CalculateObjectPosition(
          _scrollableHand.Objects.ToList().IndexOf(card), _scrollableHand.Objects.Count);
      }

      return null;
    }

    public override void ApplyTargetTransform(Displayable target, Sequence? sequence = null)
    {
      if (Objects.Count <= _useSecondLayoutAfter)
      {
        _layout1.ApplyTargetTransform(target, sequence);
      }
      else if (Objects.Count <= _useBrowserAfter)
      {
        _layout2.ApplyTargetTransform(target, sequence);
      }
      else
      {
        _scrollableHand.ApplyTargetTransform(target, sequence);
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
      _scrollableHand.RemoveIfPresent(displayable);

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

      if (totalObjects > _useBrowserAfter)
      {
        while (_layout1.Objects.Count > 0)
        {
          var obj = _layout1.Objects[0];
          _layout1.RemoveAtIndex(0);
          _scrollableHand.Add(obj);
        }

        while (_layout2.Objects.Count > 0)
        {
          var obj = _layout2.Objects[0];
          _layout2.RemoveAtIndex(0);
          _scrollableHand.Add(obj);
        }

        return;
      }

      int targetLayout1Size;

      if (totalObjects <= _useSecondLayoutAfter)
      {
        targetLayout1Size = totalObjects;
      }
      else
      {
        targetLayout1Size = totalObjects / 2;
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
    }
  }
}