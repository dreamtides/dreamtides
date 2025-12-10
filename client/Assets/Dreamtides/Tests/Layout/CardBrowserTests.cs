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
  public class CardBrowserTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardIsCenteredAndVisible_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var card = CreateTestCard();
      browser.Add(card);
      browser.ApplyLayout(sequence: null);

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

      var browser = GetCardBrowser();
      var card = CreateTestCard();
      browser.Add(card);
      browser.ApplyLayout(sequence: null);

      AssertCardIsOnScreen(viewport, card);
    }

    [UnityTest]
    public IEnumerator TwoCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 2);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator ThreeCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator FourCardsAreVisibleAndOrdered_16x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

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

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

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

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleWithGameLayoutYRotation120()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

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

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

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

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt21x9Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution21x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt32x9Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution32x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution32x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt16x10Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x10);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x10);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt3x2Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution3x2);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution3x2);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt5x4Resolution()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution5x4);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution5x4);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardRotationMatchesBrowserTransform()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var card = CreateTestCard();
      browser.Add(card);
      browser.ApplyLayout(sequence: null);

      var browserRotation = browser.transform.rotation.eulerAngles;
      Assert.That(
        card.transform.localEulerAngles.x,
        Is.EqualTo(browserRotation.x).Within(1f),
        "Card X rotation should match browser transform"
      );
      Assert.That(
        Mathf.DeltaAngle(card.transform.localEulerAngles.y, browserRotation.y),
        Is.EqualTo(0f).Within(1f),
        "Card Y rotation should match browser transform"
      );
    }

    [UnityTest]
    public IEnumerator CardRotationMatchesBrowserTransformWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var card = CreateTestCard();
      browser.Add(card);
      browser.ApplyLayout(sequence: null);

      var browserRotation = browser.transform.rotation.eulerAngles;
      Assert.That(
        Mathf.DeltaAngle(card.transform.localEulerAngles.y, browserRotation.y),
        Is.EqualTo(0f).Within(1f),
        "Card Y rotation should match browser transform when game layout is rotated"
      );
    }

    [UnityTest]
    public IEnumerator CenterCardIsAtMiddleOfViewport()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      var middleCard = cards[1];
      var viewportPoint = viewport.WorldToViewportPoint(middleCard.GetCardCenter());
      Assert.That(
        viewportPoint.x,
        Is.EqualTo(0.5f).Within(0.15f),
        "Center card should be roughly in the middle horizontally"
      );
    }

    [UnityTest]
    public IEnumerator ThreeCardsAreSymmetricallyDistributed()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos2 = viewport.WorldToScreenPoint(cards[2].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());

      var distanceLeft = screenPos1.x - screenPos0.x;
      var distanceRight = screenPos2.x - screenPos1.x;
      Assert.That(
        distanceLeft,
        Is.EqualTo(distanceRight).Within(distanceLeft * 0.15f),
        "Cards should be symmetrically distributed around center"
      );
    }

    [UnityTest]
    public IEnumerator CardsArePositionedAlongEdgeLineWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      Assert.That(
        cards[0].transform.position.z,
        Is.Not.EqualTo(cards[2].transform.position.z).Within(0.1f),
        "Cards should NOT have identical Z when layout is rotated (fixed bug)"
      );
    }

    [UnityTest]
    public IEnumerator SingleCardIsOnScreenWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var card = CreateTestCard();
      browser.Add(card);
      browser.ApplyLayout(sequence: null);

      AssertCardIsOnScreen(viewport, card);
    }

    [UnityTest]
    public IEnumerator TwoCardsAreOnScreenWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 2);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator FourCardsAreOnScreenWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt21x9ResolutionWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution21x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreVisibleAt16x10ResolutionWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x10);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x10);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CenterCardIsAtMiddleOfViewportWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      var middleCard = cards[1];
      var viewportPoint = viewport.WorldToViewportPoint(middleCard.GetCardCenter());
      Assert.That(
        viewportPoint.x,
        Is.EqualTo(0.5f).Within(0.15f),
        "Center card should be roughly in the middle horizontally even when rotated"
      );
    }

    [UnityTest]
    public IEnumerator ThreeCardsAreSymmetricallyDistributedWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos2 = viewport.WorldToScreenPoint(cards[2].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());

      var distanceLeft = screenPos1.x - screenPos0.x;
      var distanceRight = screenPos2.x - screenPos1.x;
      Assert.That(
        distanceLeft,
        Is.EqualTo(distanceRight).Within(distanceLeft * 0.15f),
        "Cards should be symmetrically distributed around center when rotated"
      );
    }

    [UnityTest]
    public IEnumerator TwoCardsAreSymmetricallyDistributed()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 2);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());
      var centerX = viewport.ScreenWidth / 2f;

      var distanceLeft = centerX - screenPos0.x;
      var distanceRight = screenPos1.x - centerX;
      Assert.That(
        distanceLeft,
        Is.EqualTo(distanceRight).Within(distanceLeft * 0.15f),
        "Two cards should be symmetrically distributed around screen center"
      );
    }

    [UnityTest]
    public IEnumerator TwoCardsAreSymmetricallyDistributedWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 2);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());
      var centerX = viewport.ScreenWidth / 2f;

      var distanceLeft = centerX - screenPos0.x;
      var distanceRight = screenPos1.x - centerX;
      Assert.That(
        distanceLeft,
        Is.EqualTo(distanceRight).Within(distanceLeft * 0.15f),
        "Two cards should be symmetrically distributed around screen center when rotated"
      );
    }

    CardBrowser GetCardBrowser()
    {
      var browser = LandscapeLayout.Browser;
      browser.GameContext = GameContext.Browser;
      return browser;
    }
  }
}
