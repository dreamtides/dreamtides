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
    public IEnumerator CardsAreOnScreenWhenGameLayoutRotated()
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
    }

    [UnityTest]
    public IEnumerator CardsAreOnScreenWithNoRotation()
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
    }

    [UnityTest]
    public IEnumerator CardsAreOnScreenWithYRotation90()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 90f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
    }

    [UnityTest]
    public IEnumerator CardsAreOnScreenWithYRotation180()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 180f, 0f);
      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
    }

    [UnityTest]
    public IEnumerator FiveCardsAreOnScreenWhenRotated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      LandscapeLayout.transform.position = new Vector3(25.1893806f, 10f, -44.6367035f);
      LandscapeLayout.transform.rotation = Quaternion.Euler(0f, 120.351913f, 0f);

      viewport = CreateViewportForLandscapeLayout(GameViewResolution.Resolution16x9);

      var browser = GetCardBrowser();
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        browser.Add(card);
      }
      browser.ApplyLayout(sequence: null);

      AssertCardsAreOnScreen(viewport, cards);
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

    CardBrowser GetCardBrowser()
    {
      var browser = LandscapeLayout.Browser;
      browser.GameContext = GameContext.Browser;
      return browser;
    }
  }
}
