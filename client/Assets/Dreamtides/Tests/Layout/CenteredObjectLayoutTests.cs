#nullable enable

using System.Collections;
using Dreamtides.Layout;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class CenteredObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator CalculateObjectPositionHorizontalAlignsAroundTransform()
    {
      yield return Initialize();
      var layout = CreateLayout(10f, 0f, 2f, false);
      layout.transform.position = new Vector3(1f, -2f, 0f);

      var first = layout.CalculateObjectPosition(0, 3);
      var middle = layout.CalculateObjectPosition(1, 3);
      var last = layout.CalculateObjectPosition(2, 3);

      AssertVector3Equal(new Vector3(-1f, -2f, 0f), first);
      AssertVector3Equal(new Vector3(1f, -2f, 0f), middle);
      AssertVector3Equal(new Vector3(3f, -2f, 0f), last);
    }

    [UnityTest]
    public IEnumerator CalculateObjectPositionVerticalUsesZAxis()
    {
      yield return Initialize();
      var layout = CreateLayout(6f, 1f, 2f, true);
      layout.transform.position = new Vector3(0f, 1f, -3f);

      var first = layout.CalculateObjectPosition(0, 2);
      var second = layout.CalculateObjectPosition(1, 2);

      AssertVector3Equal(new Vector3(0f, 1f, -5f), first);
      AssertVector3Equal(new Vector3(0f, 1f, -1f), second);
    }

    [UnityTest]
    public IEnumerator CalculateObjectRotationUsesTransformEulerAngles()
    {
      yield return Initialize();
      var layout = CreateLayout(4f, 0f, 1f, false);
      var rotation = Quaternion.Euler(12f, 34f, 56f);
      layout.transform.rotation = rotation;

      var result = layout.CalculateObjectRotation(0, 0);

      Assert.That(result.HasValue, Is.True);
      AssertVector3Equal(rotation.eulerAngles, result!.Value);
    }

    [UnityTest]
    public IEnumerator CalculateObjectScaleUsesTransformScaleX()
    {
      yield return Initialize();
      var layout = CreateLayout(4f, 0f, 1f, false);
      layout.transform.localScale = new Vector3(1.5f, 2f, 3f);

      var scale = layout.CalculateObjectScale(5, 10);

      Assert.That(scale, Is.EqualTo(1.5f));
    }

    [UnityTest]
    public IEnumerator CalculateOffsetClampsToAvailableWidth()
    {
      yield return Initialize();
      var first = CenteredObjectLayout.CalculateOffset(4f, 1f, 2f, 0, 3);
      var middle = CenteredObjectLayout.CalculateOffset(4f, 1f, 2f, 1, 3);
      var last = CenteredObjectLayout.CalculateOffset(4f, 1f, 2f, 2, 3);

      Assert.That(first, Is.EqualTo(-1f));
      Assert.That(middle, Is.EqualTo(0f));
      Assert.That(last, Is.EqualTo(1f));
    }

    [UnityTest]
    public IEnumerator CalculateOffsetReturnsZeroForSingleItem()
    {
      yield return Initialize();
      var resultForZero = CenteredObjectLayout.CalculateOffset(10f, 0f, 2f, 0, 0);
      var resultForOne = CenteredObjectLayout.CalculateOffset(10f, 0f, 2f, 0, 1);

      Assert.That(resultForZero, Is.EqualTo(0f));
      Assert.That(resultForOne, Is.EqualTo(0f));
    }

    [UnityTest]
    public IEnumerator CalculateOffsetAppliesOffsetMultipliers()
    {
      yield return Initialize();
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
      return CreateSceneObject<CenteredObjectLayout>(layout =>
      {
        layout._width = width;
        layout._initialSpacing = initialSpacing;
        layout._cardSize = cardSize;
        layout._vertical = vertical;
      });
    }
  }
}
