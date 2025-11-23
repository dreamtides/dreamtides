#nullable enable

using System.Collections;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class CenteredLineObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardIsCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card.transform.position);
      AssertVector3Equal(new Vector3(1f, 1f, 1f), card.transform.localScale);
    }

    [UnityTest]
    public IEnumerator TwoCardsAreHorizontallyCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-0.375f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0.375f, 0f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator ThreeCardsAreEvenlyCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      Assert.That(card2.transform.position.x, Is.EqualTo(0f).Within(0.01f));
      Assert.That(card1.transform.position.x, Is.LessThan(0f));
      Assert.That(card3.transform.position.x, Is.GreaterThan(0f));
      Assert.That(
        Mathf.Abs(card1.transform.position.x),
        Is.EqualTo(card3.transform.position.x).Within(0.01f)
      );
    }

    [UnityTest]
    public IEnumerator FourCardsAreEvenlyCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[4];
      for (var i = 0; i < 4; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      var spacing = cards[1].transform.position.x - cards[0].transform.position.x;
      Assert.That(spacing, Is.GreaterThan(0f));
      Assert.That(
        cards[2].transform.position.x - cards[1].transform.position.x,
        Is.EqualTo(spacing).Within(0.01f)
      );
      Assert.That(
        cards[3].transform.position.x - cards[2].transform.position.x,
        Is.EqualTo(spacing).Within(0.01f)
      );
      Assert.That(
        (cards[0].transform.position.x + cards[3].transform.position.x) * 0.5f,
        Is.EqualTo(0f).Within(0.01f)
      );
    }

    [UnityTest]
    public IEnumerator ScaleReducesAtThreshold()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[10];
      for (var i = 0; i < 10; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      Assert.That(cards[0].transform.localScale.x, Is.EqualTo(0.85f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator ScaleInterpolatesBeforeThreshold()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.8f,
        maxScale: 1f,
        minScaleThresholdPortrait: 5,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[3];
      for (var i = 0; i < 3; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      Assert.That(cards[0].transform.localScale.x, Is.GreaterThan(0.8f));
      Assert.That(cards[0].transform.localScale.x, Is.LessThan(1f));
    }

    [UnityTest]
    public IEnumerator DifferentHorizontalSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout1 = CreateTestLayout(
        horizontalSpacing: 1.5f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout1.Add(card1);
      layout1.Add(card2);
      layout1.ApplyLayout(sequence: null);

      var spacing1 = card2.transform.position.x - card1.transform.position.x;

      var layout2 = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var card3 = CreateDisplayable();
      var card4 = CreateDisplayable();
      layout2.Add(card3);
      layout2.Add(card4);
      layout2.ApplyLayout(sequence: null);

      var spacing2 = card4.transform.position.x - card3.transform.position.x;

      Assert.That(spacing1, Is.GreaterThan(spacing2));
    }

    [UnityTest]
    public IEnumerator DifferentZPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-0.375f, 0f, 5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0.375f, 0f, 5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator CalculateObjectRotationUsesTransformEulerAngles()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );
      layout.transform.rotation = Quaternion.Euler(10f, 20f, 30f);

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(10f, 20f, 30f), card.transform.localEulerAngles);
    }

    [UnityTest]
    public IEnumerator LayoutWithTransformScale()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );
      layout.transform.localScale = new Vector3(2f, 2f, 2f);

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var spacing = card2.transform.position.x - card1.transform.position.x;
      Assert.That(spacing, Is.GreaterThan(0f));
      Assert.That(
        (card1.transform.position.x + card2.transform.position.x) * 0.5f,
        Is.EqualTo(0f).Within(0.01f)
      );
    }

    [UnityTest]
    public IEnumerator FiveCardsFormCenteredLine()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.5f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      Assert.That(cards[2].transform.position.x, Is.EqualTo(0f).Within(0.01f));
      var spacing = cards[1].transform.position.x - cards[0].transform.position.x;
      Assert.That(spacing, Is.GreaterThan(0f));
      Assert.That(
        cards[2].transform.position.x - cards[1].transform.position.x,
        Is.EqualTo(spacing).Within(0.01f)
      );
      Assert.That(
        cards[3].transform.position.x - cards[2].transform.position.x,
        Is.EqualTo(spacing).Within(0.01f)
      );
      Assert.That(
        cards[4].transform.position.x - cards[3].transform.position.x,
        Is.EqualTo(spacing).Within(0.01f)
      );
    }

    [UnityTest]
    public IEnumerator ScaleAppliedToAllCards()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.7f,
        maxScale: 1f,
        minScaleThresholdPortrait: 3,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      var expectedScale = cards[0].transform.localScale.x;
      for (var i = 1; i < 5; i++)
      {
        Assert.That(cards[i].transform.localScale.x, Is.EqualTo(expectedScale).Within(0.001f));
      }
    }

    [UnityTest]
    public IEnumerator ScaleInterpolatesBeforeThresholdReached()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.8f,
        maxScale: 1f,
        minScaleThresholdPortrait: 2,
        minScaleThresholdLandscape: 10,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      var scale = cards[0].transform.localScale.x;
      Assert.That(scale, Is.GreaterThan(0.8f));
      Assert.That(scale, Is.LessThan(1f));
    }

    [UnityTest]
    public IEnumerator ThresholdAppliesMinScale()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.9f,
        maxScale: 1f,
        minScaleThresholdPortrait: 3,
        minScaleThresholdLandscape: 10,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[15];
      for (var i = 0; i < 15; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      Assert.That(cards[0].transform.localScale.x, Is.EqualTo(0.9f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator SixCardsWithSmallSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.3f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var cards = new TestDisplayable[6];
      for (var i = 0; i < 6; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      var spacing = cards[1].transform.position.x - cards[0].transform.position.x;
      Assert.That(spacing, Is.GreaterThan(0f));
      for (var i = 1; i < 5; i++)
      {
        var currentSpacing = cards[i + 1].transform.position.x - cards[i].transform.position.x;
        Assert.That(currentSpacing, Is.EqualTo(spacing).Within(0.01f));
      }
      Assert.That(
        (cards[0].transform.position.x + cards[5].transform.position.x) * 0.5f,
        Is.EqualTo(0f).Within(0.01f)
      );
    }

    [UnityTest]
    public IEnumerator MaxScaleUsedForSingleCard()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.5f,
        maxScale: 1.2f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(card.transform.localScale.x, Is.EqualTo(1.2f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator OffsetLayoutPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 0.75f,
        cardWidth: 1f,
        minScale: 0.85f,
        maxScale: 1f,
        minScaleThresholdPortrait: 4,
        minScaleThresholdLandscape: 8,
        zPosition: 2.5f
      );
      layout.transform.localPosition = new Vector3(3f, 4f, 2.5f);

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(3f, 4f, 2.5f), card.transform.position);
    }

    CenteredLineObjectLayout CreateTestLayout(
      float horizontalSpacing,
      float cardWidth,
      float minScale,
      float maxScale,
      int minScaleThresholdPortrait,
      int minScaleThresholdLandscape,
      float zPosition
    )
    {
      var layout = CreateSceneObject<CenteredLineObjectLayout>(l =>
      {
        l._horizontalSpacing = horizontalSpacing;
        l._cardWidth = cardWidth;
        l._minScale = minScale;
        l._maxScale = maxScale;
        l._minScaleThresholdPortrait = minScaleThresholdPortrait;
        l._minScaleThresholdLandscape = minScaleThresholdLandscape;
      });
      layout.transform.localPosition = new Vector3(0f, 0f, zPosition);
      layout.transform.localRotation = Quaternion.identity;
      layout.GameContext = GameContext.Browser;
      return layout;
    }
  }
}
