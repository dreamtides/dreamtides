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
  public class LandscapeHandLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardIsCenteredAndVisible_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var card = CreateTestCard();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertCardIsOnScreen(viewport, card);
      var viewportCenter = viewport.WorldToViewportPoint(card.GetCardCenter());
      Assert.That(
        viewportCenter.x,
        Is.EqualTo(0.5f).Within(0.15f),
        "Single card should be roughly centered horizontally"
      );
    }

    [UnityTest]
    public IEnumerator SingleCardIsCenteredAndVisible_21x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution21x9);

      var layout = GetLandscapeHandLayout();
      var card = CreateTestCard();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertCardIsOnScreen(viewport, card);
    }

    [UnityTest]
    public IEnumerator ThreeCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator FiveCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator SevenCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 7);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator NineCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 9);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleWithGameLayoutYRotation0()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 0f
      );

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleWithGameLayoutYRotation90()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 90f
      );

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleWithGameLayoutYRotation120()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 120f
      );

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleWithGameLayoutYRotation180()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 180f
      );

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleWithGameLayoutYRotation270()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 270f
      );

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt21x9Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution21x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt32x9Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution32x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution32x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt16x10Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x10);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x10);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt3x2Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution3x2);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution3x2);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt5x4Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution5x4);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution5x4);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardRotationsIncludeCameraXAngle()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var card = CreateTestCard();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card.transform.localEulerAngles.x,
        Is.EqualTo(75f).Within(1f),
        "Card X rotation should match CameraXAngle"
      );
    }

    [UnityTest]
    public IEnumerator CardRotationIncludesBattleYRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForLandscapeLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 120f
      );

      var layout = GetLandscapeHandLayout();
      var card = CreateTestCard();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      var expectedYRotation = LandscapeLayout.BattleYRotation();
      Assert.That(
        Mathf.DeltaAngle(card.transform.localEulerAngles.y, expectedYRotation),
        Is.EqualTo(0f).Within(5f),
        $"Card Y rotation should match BattleYRotation ({expectedYRotation})"
      );
    }

    [UnityTest]
    public IEnumerator CardsHaveVaryingZRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var zRotations = new float[cards.Length];
      for (var i = 0; i < cards.Length; i++)
      {
        zRotations[i] = cards[i].transform.localEulerAngles.z;
      }

      Assert.That(
        zRotations[0],
        Is.Not.EqualTo(zRotations[2]).Within(0.5f),
        "Cards should have varying Z rotation based on curve position"
      );
      Assert.That(
        zRotations[2],
        Is.Not.EqualTo(zRotations[4]).Within(0.5f),
        "Cards should have varying Z rotation based on curve position"
      );
    }

    [UnityTest]
    public IEnumerator CenterCardIsAtMiddleOfViewport()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var middleCard = cards[2];
      var viewportPoint = viewport.WorldToViewportPoint(middleCard.GetCardCenter());
      Assert.That(
        viewportPoint.x,
        Is.EqualTo(0.5f).Within(0.15f),
        "Center card should be roughly in the middle horizontally"
      );
    }

    [UnityTest]
    public IEnumerator CardsAreSymmetricallyDistributed()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos4 = viewport.WorldToScreenPoint(cards[4].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());
      var screenPos3 = viewport.WorldToScreenPoint(cards[3].GetCardCenter());
      var screenPos2 = viewport.WorldToScreenPoint(cards[2].GetCardCenter());

      var distanceLeft = screenPos2.x - screenPos0.x;
      var distanceRight = screenPos4.x - screenPos2.x;
      Assert.That(
        distanceLeft,
        Is.EqualTo(distanceRight).Within(distanceLeft * 0.1f),
        "Cards should be symmetrically distributed around center"
      );

      var innerDistanceLeft = screenPos2.x - screenPos1.x;
      var innerDistanceRight = screenPos3.x - screenPos2.x;
      Assert.That(
        innerDistanceLeft,
        Is.EqualTo(innerDistanceRight).Within(innerDistanceLeft * 0.1f),
        "Inner cards should be symmetrically distributed around center"
      );
    }

    [UnityTest]
    public IEnumerator MaximumCardsAreStillVisible()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var layout = GetLandscapeHandLayout();
      var cards = CreateTestCards(count: 15);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    LandscapeHandLayout GetLandscapeHandLayout()
    {
      var layout = (LandscapeHandLayout)LandscapeLayout.UserHand._layout1;
      layout.GameContext = GameContext.Hand;
      return layout;
    }
  }
}
