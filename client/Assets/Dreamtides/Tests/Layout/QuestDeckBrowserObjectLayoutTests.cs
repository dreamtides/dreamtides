#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.Rendering;
using UnityEngine.TestTools;
using UnityEngine.UI;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class QuestDeckBrowserObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator TwoCardsLandscapeDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var card1 = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      var card2 = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertNoOverlap(card1, card2);
    }

    [UnityTest]
    public IEnumerator ThreeCardsLandscapeDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var cards = CreateCardsWithColliders(count: 3, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator SixCardsLandscapeWithMultipleRowsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 700f
      );

      var cards = CreateCardsWithColliders(count: 6, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator TenCardsLandscapeDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 150f,
        cardHeight: 210f,
        cardSpacing: 15f,
        cardScale: 0.75f,
        worldSpaceDepth: 15f,
        contentWidth: 1000f
      );

      var cards = CreateCardsWithColliders(count: 10, colliderWidth: 1.5f, colliderHeight: 2.1f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator TwoCardsPortraitDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 150f,
        cardHeight: 210f,
        cardSpacing: 30f,
        cardScale: 1f,
        worldSpaceDepth: 12f,
        contentWidth: 500f
      );

      var card1 = CreateDisplayableWithCollider(colliderWidth: 0.8f, colliderHeight: 1.1f);
      var card2 = CreateDisplayableWithCollider(colliderWidth: 0.8f, colliderHeight: 1.1f);
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertNoOverlap(card1, card2);
    }

    [UnityTest]
    public IEnumerator FourCardsPortraitDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 150f,
        cardHeight: 210f,
        cardSpacing: 30f,
        cardScale: 1f,
        worldSpaceDepth: 12f,
        contentWidth: 500f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 0.8f, colliderHeight: 1.1f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator IPhoneSECardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhoneSE);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 120f,
        cardHeight: 168f,
        cardSpacing: 25f,
        cardScale: 1f,
        worldSpaceDepth: 10f,
        contentWidth: 450f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 0.6f, colliderHeight: 0.84f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator IPadPro12CardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPadPro12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 30f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 1200f
      );

      var cards = CreateCardsWithColliders(count: 8, colliderWidth: 0.8f, colliderHeight: 1.1f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator SamsungNote20CardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionSamsungNote20);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 140f,
        cardHeight: 196f,
        cardSpacing: 30f,
        cardScale: 1f,
        worldSpaceDepth: 12f,
        contentWidth: 500f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 0.7f, colliderHeight: 1f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator Pixel5CardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionPixel5);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 130f,
        cardHeight: 182f,
        cardSpacing: 25f,
        cardScale: 1f,
        worldSpaceDepth: 11f,
        contentWidth: 500f
      );

      var cards = CreateCardsWithColliders(count: 6, colliderWidth: 0.65f, colliderHeight: 0.9f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator UltrawideCardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 220f,
        cardHeight: 308f,
        cardSpacing: 22f,
        cardScale: 1.1f,
        worldSpaceDepth: 18f,
        contentWidth: 1200f
      );

      var cards = CreateCardsWithColliders(count: 10, colliderWidth: 2.2f, colliderHeight: 3.08f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator SuperUltrawideCardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution32x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 1600f
      );

      var cards = CreateCardsWithColliders(count: 12, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator Resolution5x4CardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution5x4);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 180f,
        cardHeight: 252f,
        cardSpacing: 18f,
        cardScale: 0.95f,
        worldSpaceDepth: 14f,
        contentWidth: 600f
      );

      var cards = CreateCardsWithColliders(count: 5, colliderWidth: 1.8f, colliderHeight: 2.52f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator Resolution3x2CardsDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution3x2);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 190f,
        cardHeight: 266f,
        cardSpacing: 19f,
        cardScale: 0.97f,
        worldSpaceDepth: 14.5f,
        contentWidth: 650f
      );

      var cards = CreateCardsWithColliders(count: 6, colliderWidth: 1.9f, colliderHeight: 2.66f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator FirstRowCardsHaveSameYPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 700f
      );

      var cards = CreateCardsWithColliders(count: 3, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var firstY = cards[0].transform.localPosition.y;
      Assert.That(cards[1].transform.localPosition.y, Is.EqualTo(firstY).Within(0.01f));
      Assert.That(cards[2].transform.localPosition.y, Is.EqualTo(firstY).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator SecondRowCardsHaveLowerYPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 460f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var firstRowY = cards[0].transform.localPosition.y;
      var secondRowY = cards[2].transform.localPosition.y;

      Assert.That(cards[1].transform.localPosition.y, Is.EqualTo(firstRowY).Within(0.01f));
      Assert.That(cards[3].transform.localPosition.y, Is.EqualTo(secondRowY).Within(0.01f));
      Assert.That(secondRowY, Is.LessThan(firstRowY));
    }

    [UnityTest]
    public IEnumerator CardsInSameRowAreHorizontallyDistributed()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var cards = CreateCardsWithColliders(count: 3, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var x0 = cards[0].transform.localPosition.x;
      var x1 = cards[1].transform.localPosition.x;
      var x2 = cards[2].transform.localPosition.x;

      Assert.That(x0, Is.LessThan(x1));
      Assert.That(x1, Is.LessThan(x2));

      var spacing1 = x1 - x0;
      var spacing2 = x2 - x1;
      Assert.That(spacing1, Is.EqualTo(spacing2).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator ContentHeightMatchesNumberOfRows()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 460f
      );

      layout.EnsureRectangleCount(targetCount: 6);

      var expectedRows = 3;
      var expectedHeight = (expectedRows * 280f) + ((expectedRows - 1) * 20f) + (2 * 20f);
      Assert.That(layout._content.sizeDelta.y, Is.EqualTo(expectedHeight).Within(1f));
    }

    [UnityTest]
    public IEnumerator SingleColumnLayoutHasCorrectContentHeight()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 250f
      );

      layout.EnsureRectangleCount(targetCount: 4);

      var expectedRows = 4;
      var expectedHeight = (expectedRows * 280f) + ((expectedRows - 1) * 20f) + (2 * 20f);
      Assert.That(layout._content.sizeDelta.y, Is.EqualTo(expectedHeight).Within(1f));
    }

    [UnityTest]
    public IEnumerator CardsPlacedAtCorrectZDepthLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var card = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(card.transform.localPosition.z, Is.EqualTo(15f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator CardsPlacedAtCorrectZDepthPortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 12f,
        contentWidth: 500f
      );

      var card = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(card.transform.localPosition.z, Is.EqualTo(12f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator CardsPlacedAtCorrectZDepthUltrawide()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 18f,
        contentWidth: 1200f
      );

      var card = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(card.transform.localPosition.z, Is.EqualTo(18f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator PortraitLayoutCreatesMoreRows()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 150f,
        cardHeight: 210f,
        cardSpacing: 15f,
        cardScale: 1f,
        worldSpaceDepth: 12f,
        contentWidth: 350f
      );

      layout.EnsureRectangleCount(targetCount: 4);

      Assert.That(layout._content.sizeDelta.y, Is.GreaterThan(210f * 2));
    }

    [UnityTest]
    public IEnumerator WideContentWidthFitsMoreCardsPerRow()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layoutNarrow = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 460f
      );
      var layoutWide = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 1000f
      );

      layoutNarrow.EnsureRectangleCount(targetCount: 8);
      layoutWide.EnsureRectangleCount(targetCount: 8);

      Assert.That(layoutWide._content.sizeDelta.y, Is.LessThan(layoutNarrow._content.sizeDelta.y));
    }

    [UnityTest]
    public IEnumerator ZeroSpacingAllowsTouchingButNotOverlapping()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 0f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator LargeSpacingKeepsCardsSeparated()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 100f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 1200f
      );

      var cards = CreateCardsWithColliders(count: 3, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
      AssertMinimumSeparation(cards[0], cards[1], expectedMinGap: 0.5f);
    }

    [UnityTest]
    public IEnumerator ScaledCollidersDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 0.5f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var cards = CreateCardsWithColliders(count: 6, colliderWidth: 4f, colliderHeight: 5.6f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator LargeScaledCollidersDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 30f,
        cardScale: 1.5f,
        worldSpaceDepth: 15f,
        contentWidth: 1000f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 1.33f, colliderHeight: 1.867f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator CardsArePlacedAtCorrectZDepth()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      foreach (var card in cards)
      {
        Assert.That(card.transform.localPosition.z, Is.EqualTo(15f).Within(0.01f));
      }
    }

    [UnityTest]
    public IEnumerator DifferentZDepthPlacesCardsCorrectly()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 25f,
        contentWidth: 800f
      );

      var card = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(card.transform.localPosition.z, Is.EqualTo(25f).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator ManyCardsInGridDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 100f,
        cardHeight: 140f,
        cardSpacing: 10f,
        cardScale: 0.5f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var cards = CreateCardsWithColliders(count: 20, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator WorldSpaceOffsetMovesSingleCard()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);

      var layoutWithoutOffset = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f,
        worldSpaceOffset: Vector2.zero
      );

      var layoutWithOffset = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f,
        worldSpaceOffset: new Vector2(1.5f, -0.5f)
      );

      var card1 = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      var card2 = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      layoutWithoutOffset.Add(card1);
      layoutWithOffset.Add(card2);
      layoutWithoutOffset.ApplyLayout(sequence: null);
      layoutWithOffset.ApplyLayout(sequence: null);

      var positionDiff = card2.transform.localPosition - card1.transform.localPosition;
      Assert.That(positionDiff.x, Is.EqualTo(1.5f).Within(0.1f));
      Assert.That(positionDiff.y, Is.EqualTo(-0.5f).Within(0.1f));
    }

    [UnityTest]
    public IEnumerator CardsPlacedWithinContentBounds()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 460f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator CardsSortedBySortingKeyDoNotOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var card1 = CreateDisplayableWithColliderAndSortingGroup(
        colliderWidth: 2f,
        colliderHeight: 2.8f
      );
      var card2 = CreateDisplayableWithColliderAndSortingGroup(
        colliderWidth: 2f,
        colliderHeight: 2.8f
      );
      var card3 = CreateDisplayableWithColliderAndSortingGroup(
        colliderWidth: 2f,
        colliderHeight: 2.8f
      );
      card1.SortingKey = 3;
      card2.SortingKey = 1;
      card3.SortingKey = 2;
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      Assert.That(layout.Objects[0], Is.EqualTo(card2));
      Assert.That(layout.Objects[1], Is.EqualTo(card3));
      Assert.That(layout.Objects[2], Is.EqualTo(card1));
      AssertNoOverlap(card1, card2);
      AssertNoOverlap(card1, card3);
      AssertNoOverlap(card2, card3);
    }

    [UnityTest]
    public IEnumerator ExcludedCardDoesNotAffectOtherCardPositions()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var card1 = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      var excludedCard = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
      var card2 = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);

      excludedCard.ExcludeFromLayout = true;
      excludedCard.transform.localPosition = new Vector3(100f, 100f, 100f);

      layout.Add(card1);
      layout.Add(excludedCard);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(excludedCard.transform.localPosition.x, Is.EqualTo(100f).Within(0.01f));
      AssertNoOverlap(card1, card2);
    }

    [UnityTest]
    public IEnumerator AddingCardsIncrementallyMaintainsNonOverlapping()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 800f
      );

      var allCards = new List<TestDisplayable>();

      for (var i = 0; i < 5; i++)
      {
        var card = CreateDisplayableWithCollider(colliderWidth: 2f, colliderHeight: 2.8f);
        allCards.Add(card);
        layout.Add(card);
        layout.ApplyLayout(sequence: null);

        AssertNoOverlapBetweenAny(allCards.ToArray());
      }
    }

    [UnityTest]
    public IEnumerator NarrowContentWidthStillPreventsOverlap()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 250f
      );

      var cards = CreateCardsWithColliders(count: 4, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    [UnityTest]
    public IEnumerator VeryWideContentWidthMaintainsNonOverlapping()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        cardWidth: 200f,
        cardHeight: 280f,
        cardSpacing: 20f,
        cardScale: 1f,
        worldSpaceDepth: 15f,
        contentWidth: 2000f
      );

      var cards = CreateCardsWithColliders(count: 8, colliderWidth: 2f, colliderHeight: 2.8f);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertNoOverlapBetweenAny(cards);
    }

    QuestDeckBrowserObjectLayout CreateTestLayout(
      float cardWidth,
      float cardHeight,
      float cardSpacing,
      float cardScale,
      float worldSpaceDepth,
      float contentWidth,
      Vector2? worldSpaceOffset = null
    )
    {
      var canvas = CreateGameObject();
      canvas.AddComponent<Canvas>();
      canvas.AddComponent<CanvasScaler>();
      canvas.AddComponent<GraphicRaycaster>();

      var contentObject = CreateGameObject();
      contentObject.transform.SetParent(canvas.transform, worldPositionStays: false);
      var content = contentObject.AddComponent<RectTransform>();
      content.anchorMin = Vector2.zero;
      content.anchorMax = Vector2.one;
      content.sizeDelta = new Vector2(contentWidth, 600f);
      content.anchoredPosition = Vector2.zero;

      var scrollbarObject = CreateGameObject();
      scrollbarObject.transform.SetParent(canvas.transform, worldPositionStays: false);
      var scrollbarCanvasGroup = scrollbarObject.AddComponent<CanvasGroup>();

      var closeButtonObject = CreateGameObject();
      closeButtonObject.transform.SetParent(canvas.transform, worldPositionStays: false);
      var closeButton = closeButtonObject.AddComponent<CloseBrowserButton>();

      var worldSpaceParent = CreateGameObject().transform;

      var layout = CreateSceneObject<QuestDeckBrowserObjectLayout>(l =>
      {
        l._cardWidth = cardWidth;
        l._cardHeight = cardHeight;
        l._cardSpacing = cardSpacing;
        l._cardScale = cardScale;
        l._worldSpaceDepth = worldSpaceDepth;
        l._worldSpaceOffset = worldSpaceOffset ?? Vector2.zero;
        l._content = content;
        l._scrollbarCanvasGroup = scrollbarCanvasGroup;
        l._closeButton = closeButton;
        l._worldSpaceParent = worldSpaceParent;
        l._enableDebugOutline = false;
        l._scrollbarFadeDuration = 0.2f;
        l._backgroundOverlayOpacity = 1f;
        l._disableInterfaceOnOpen = false;
      });
      layout.GameContext = GameContext.Browser;
      return layout;
    }

    TestDisplayable CreateDisplayableWithCollider(float colliderWidth, float colliderHeight)
    {
      var displayable = CreateDisplayable();
      var collider = displayable.gameObject.AddComponent<BoxCollider>();
      collider.size = new Vector3(colliderWidth, colliderHeight, 0.1f);
      collider.center = Vector3.zero;
      return displayable;
    }

    TestDisplayable[] CreateCardsWithColliders(int count, float colliderWidth, float colliderHeight)
    {
      var cards = new TestDisplayable[count];
      for (var i = 0; i < count; i++)
      {
        cards[i] = CreateDisplayableWithCollider(colliderWidth, colliderHeight);
      }
      return cards;
    }

    TestDisplayable CreateDisplayableWithColliderAndSortingGroup(
      float colliderWidth,
      float colliderHeight
    )
    {
      var displayable = CreateDisplayableWithCollider(colliderWidth, colliderHeight);
      displayable._sortingGroup = displayable.gameObject.AddComponent<SortingGroup>();
      return displayable;
    }

    void AssertNoOverlap(Displayable card1, Displayable card2)
    {
      var collider1 = card1.GetComponent<BoxCollider>();
      var collider2 = card2.GetComponent<BoxCollider>();

      var bounds1 = GetScaledBounds(card1.transform, collider1);
      var bounds2 = GetScaledBounds(card2.transform, collider2);

      Assert.That(
        bounds1.Intersects(bounds2),
        Is.False,
        $"Cards overlap: {card1.name} at {card1.transform.position} bounds {bounds1} intersects {card2.name} at {card2.transform.position} bounds {bounds2}"
      );
    }

    void AssertNoOverlapBetweenAny(TestDisplayable[] cards)
    {
      for (var i = 0; i < cards.Length; i++)
      {
        for (var j = i + 1; j < cards.Length; j++)
        {
          AssertNoOverlap(cards[i], cards[j]);
        }
      }
    }

    void AssertMinimumSeparation(Displayable card1, Displayable card2, float expectedMinGap)
    {
      var collider1 = card1.GetComponent<BoxCollider>();
      var collider2 = card2.GetComponent<BoxCollider>();

      var bounds1 = GetScaledBounds(card1.transform, collider1);
      var bounds2 = GetScaledBounds(card2.transform, collider2);

      var closestPoint1 = bounds1.ClosestPoint(bounds2.center);
      var closestPoint2 = bounds2.ClosestPoint(bounds1.center);
      var distance = Vector3.Distance(closestPoint1, closestPoint2);

      Assert.That(
        distance,
        Is.GreaterThanOrEqualTo(expectedMinGap),
        $"Cards {card1.name} and {card2.name} are not sufficiently separated. Distance: {distance}, expected min: {expectedMinGap}"
      );
    }

    Bounds GetScaledBounds(Transform transform, BoxCollider collider)
    {
      var scale = transform.lossyScale;
      var scaledSize = new Vector3(
        collider.size.x * scale.x,
        collider.size.y * scale.y,
        collider.size.z * scale.z
      );
      var worldCenter = transform.TransformPoint(collider.center);
      return new Bounds(worldCenter, scaledSize);
    }
  }
}
