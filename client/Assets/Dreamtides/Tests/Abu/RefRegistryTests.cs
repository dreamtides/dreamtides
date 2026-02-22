#nullable enable

using System;
using NUnit.Framework;

namespace Abu.Tests
{
  public class RefRegistryTests
  {
    [Test]
    public void AssignRefsMonotonically()
    {
      var registry = new RefRegistry();
      var ref1 = registry.Register(new RefCallbacks());
      var ref2 = registry.Register(new RefCallbacks());
      var ref3 = registry.Register(new RefCallbacks());

      Assert.AreEqual("e1", ref1);
      Assert.AreEqual("e2", ref2);
      Assert.AreEqual("e3", ref3);
    }

    [Test]
    public void LookupByRef()
    {
      var registry = new RefRegistry();
      var clicked = false;
      var callbacks = new RefCallbacks { OnClick = () => clicked = true };
      var refStr = registry.Register(callbacks);

      Assert.IsTrue(registry.TryGetCallbacks(refStr, out var found));
      Assert.IsNotNull(found);
      found!.OnClick?.Invoke();
      Assert.IsTrue(clicked);
    }

    [Test]
    public void LookupMissingRef()
    {
      var registry = new RefRegistry();
      Assert.IsFalse(registry.TryGetCallbacks("e99", out _));
    }

    [Test]
    public void ClearInvalidatesAllRefs()
    {
      var registry = new RefRegistry();
      registry.Register(new RefCallbacks());
      registry.Register(new RefCallbacks());

      registry.Clear();

      Assert.IsFalse(registry.TryGetCallbacks("e1", out _));
      Assert.IsFalse(registry.TryGetCallbacks("e2", out _));
    }

    [Test]
    public void RefsResetAfterClear()
    {
      var registry = new RefRegistry();
      registry.Register(new RefCallbacks());
      registry.Register(new RefCallbacks());

      registry.Clear();

      var ref1 = registry.Register(new RefCallbacks());
      var ref2 = registry.Register(new RefCallbacks());

      Assert.AreEqual("e1", ref1);
      Assert.AreEqual("e2", ref2);
    }

    [Test]
    public void RefFormatIsValid()
    {
      var registry = new RefRegistry();
      for (var i = 0; i < 100; i++)
      {
        var refStr = registry.Register(new RefCallbacks());
        Assert.IsTrue(refStr.StartsWith("e"), $"Ref '{refStr}' does not start with 'e'");
        var numberPart = refStr.Substring(1);
        Assert.IsTrue(
          int.TryParse(numberPart, out var number),
          $"Ref '{refStr}' does not have a valid integer suffix"
        );
        Assert.Greater(number, 0, $"Ref '{refStr}' number is not positive");
      }
    }

    [Test]
    public void LookupReturnsCorrectCallbacksForEachRef()
    {
      var registry = new RefRegistry();
      var clickedA = false;
      var clickedB = false;

      var refA = registry.Register(new RefCallbacks { OnClick = () => clickedA = true });
      var refB = registry.Register(new RefCallbacks { OnClick = () => clickedB = true });

      Assert.IsTrue(registry.TryGetCallbacks(refA, out var foundA));
      Assert.IsTrue(registry.TryGetCallbacks(refB, out var foundB));

      foundA!.OnClick?.Invoke();
      Assert.IsTrue(clickedA);
      Assert.IsFalse(clickedB);

      foundB!.OnClick?.Invoke();
      Assert.IsTrue(clickedB);
    }

    [Test]
    public void DragCallbackReceivesTargetRef()
    {
      var registry = new RefRegistry();
      string? receivedTarget = null;
      var callbacks = new RefCallbacks { OnDrag = target => receivedTarget = target };
      var refStr = registry.Register(callbacks);

      Assert.IsTrue(registry.TryGetCallbacks(refStr, out var found));
      found!.OnDrag?.Invoke("e5");
      Assert.AreEqual("e5", receivedTarget);
    }

    [Test]
    public void DragCallbackReceivesNullTarget()
    {
      var registry = new RefRegistry();
      string? receivedTarget = "initial";
      var callbacks = new RefCallbacks { OnDrag = target => receivedTarget = target };
      var refStr = registry.Register(callbacks);

      Assert.IsTrue(registry.TryGetCallbacks(refStr, out var found));
      found!.OnDrag?.Invoke(null);
      Assert.IsNull(receivedTarget);
    }
  }
}
