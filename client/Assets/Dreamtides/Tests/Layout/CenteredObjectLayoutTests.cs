#nullable enable

using System;
using System.Collections.Generic;
using System.Reflection;
using Dreamtides.Layout;
using Dreamtides.Services;
using NUnit.Framework;
using NUnit.Framework.Internal;
using UnityEngine;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class CenteredObjectLayoutTests
  {
    Registry? _registry;
    readonly List<GameObject> _createdObjects = new();

    [TearDown]
    public void TearDown()
    {
      foreach (var createdObject in _createdObjects)
      {
        if (createdObject)
        {
          UnityEngine.Object.DestroyImmediate(createdObject);
        }
      }

      if (_registry)
      {
        UnityEngine.Object.DestroyImmediate(_registry);
      }

      _createdObjects.Clear();
    }

    [Test]
    public void CalculateObjectPosition_HorizontalAlignsAroundTransform()
    {
      var layout = CreateLayout(10f, 0f, 2f, false);
      layout.transform.position = new Vector3(1f, -2f, 0f);

      var first = layout.CalculateObjectPosition(0, 3);
      var middle = layout.CalculateObjectPosition(1, 3);
      var last = layout.CalculateObjectPosition(2, 3);

      AssertVector3Equal(new Vector3(-1f, -2f, 0f), first);
      AssertVector3Equal(new Vector3(1f, -2f, 0f), middle);
      AssertVector3Equal(new Vector3(3f, -2f, 0f), last);
    }

    [Test]
    public void CalculateObjectPosition_VerticalUsesZAxis()
    {
      var layout = CreateLayout(6f, 1f, 2f, true);
      layout.transform.position = new Vector3(0f, 1f, -3f);

      var first = layout.CalculateObjectPosition(0, 2);
      var second = layout.CalculateObjectPosition(1, 2);

      AssertVector3Equal(new Vector3(0f, 1f, -5f), first);
      AssertVector3Equal(new Vector3(0f, 1f, -1f), second);
    }

    [Test]
    public void CalculateObjectRotation_UsesTransformEulerAngles()
    {
      var layout = CreateLayout(4f, 0f, 1f, false);
      var rotation = Quaternion.Euler(12f, 34f, 56f);
      layout.transform.rotation = rotation;

      var result = layout.CalculateObjectRotation(0, 0);

      Assert.That(result.HasValue, Is.True);
      AssertVector3Equal(rotation.eulerAngles, result!.Value);
    }

    [Test]
    public void CalculateObjectScale_UsesTransformScaleX()
    {
      var layout = CreateLayout(4f, 0f, 1f, false);
      layout.transform.localScale = new Vector3(1.5f, 2f, 3f);

      var scale = layout.CalculateObjectScale(5, 10);

      Assert.That(scale, Is.EqualTo(1.5f));
    }

    [Test]
    public void CalculateOffset_ClampsToAvailableWidth()
    {
      var first = CenteredObjectLayout.CalculateOffset(4f, 1f, 2f, 0, 3);
      var middle = CenteredObjectLayout.CalculateOffset(4f, 1f, 2f, 1, 3);
      var last = CenteredObjectLayout.CalculateOffset(4f, 1f, 2f, 2, 3);

      Assert.That(first, Is.EqualTo(-1f));
      Assert.That(middle, Is.EqualTo(0f));
      Assert.That(last, Is.EqualTo(1f));
    }

    [Test]
    public void CalculateOffset_ReturnsZeroForSingleItem()
    {
      var resultForZero = CenteredObjectLayout.CalculateOffset(10f, 0f, 2f, 0, 0);
      var resultForOne = CenteredObjectLayout.CalculateOffset(10f, 0f, 2f, 0, 1);

      Assert.That(resultForZero, Is.EqualTo(0f));
      Assert.That(resultForOne, Is.EqualTo(0f));
    }

    [Test]
    public void CalculateOffset_AppliesOffsetMultipliers()
    {
      var first = CenteredObjectLayout.CalculateOffset(8f, 0f, 2f, 0, 3, 0.5f, 2f);
      var middle = CenteredObjectLayout.CalculateOffset(8f, 0f, 2f, 1, 3, 0.5f, 2f);
      var last = CenteredObjectLayout.CalculateOffset(8f, 0f, 2f, 2, 3, 0.5f, 2f);

      Assert.That(first, Is.EqualTo(-1f));
      Assert.That(middle, Is.EqualTo(1.5f));
      Assert.That(last, Is.EqualTo(4f));
    }

    CenteredObjectLayout CreateLayout(
      float width,
      float initialSpacing,
      float cardSize,
      bool vertical
    )
    {
      _registry = new GameObject().AddComponent<Registry>();
      var gameObject = new GameObject();
      _createdObjects.Add(gameObject);
      var layout = gameObject.AddComponent<CenteredObjectLayout>();
      layout.Initialize(_registry, GameMode.Quest, new TestConfiguration(Guid.NewGuid()));
      SetField(layout, "_width", width);
      SetField(layout, "_initialSpacing", initialSpacing);
      SetField(layout, "_cardSize", cardSize);
      SetField(layout, "_vertical", vertical);
      return layout;
    }

    static void SetField<T>(CenteredObjectLayout layout, string fieldName, T value)
    {
      var field = typeof(CenteredObjectLayout).GetField(
        fieldName,
        BindingFlags.Instance | BindingFlags.NonPublic
      );
      Assert.That(field, Is.Not.Null);
      field!.SetValue(layout, value);
    }

    static void AssertVector3Equal(Vector3 expected, Vector3 actual, float tolerance = 0.0001f)
    {
      Assert.That(actual.x, Is.EqualTo(expected.x).Within(tolerance));
      Assert.That(actual.y, Is.EqualTo(expected.y).Within(tolerance));
      Assert.That(actual.z, Is.EqualTo(expected.z).Within(tolerance));
    }
  }
}
