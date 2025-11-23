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
  public class SitePickObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardInLandscapeIsCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card.transform.position);
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscapeAreHorizontallySpaced()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(1f, 0f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator ThreeCardsInLandscapeAreEvenlyDistributed()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card2.transform.position);
      AssertVector3Equal(new Vector3(2f, 0f, 2.5f), card3.transform.position);
    }

    [UnityTest]
    public IEnumerator FiveCardsInLandscapeUseFullSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 3f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-6f, 0f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(-3f, 0f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(3f, 0f, 2.5f), cards[3].transform.position);
      AssertVector3Equal(new Vector3(6f, 0f, 2.5f), cards[4].transform.position);
    }

    [UnityTest]
    public IEnumerator SingleCardInPortraitIsCentered()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card.transform.position);
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitFormTwoRows()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0f, 0.75f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0f, -0.75f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator ThreeCardsInPortraitHaveTwoTopOneBottom()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 0.75f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(1f, 0.75f, 2.5f), card2.transform.position);
      AssertVector3Equal(new Vector3(0f, -0.75f, 2.5f), card3.transform.position);
    }

    [UnityTest]
    public IEnumerator FourCardsInPortraitFormTwoEvenRows()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var cards = new TestDisplayable[4];
      for (var i = 0; i < 4; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 0.75f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(1f, 0.75f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-1f, -0.75f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1f, -0.75f, 2.5f), cards[3].transform.position);
    }

    [UnityTest]
    public IEnumerator FiveCardsInPortraitHaveThreeTopTwoBottom()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2f, 0.75f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(0f, 0.75f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(2f, 0.75f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(-1f, -0.75f, 2.5f), cards[3].transform.position);
      AssertVector3Equal(new Vector3(1f, -0.75f, 2.5f), cards[4].transform.position);
    }

    [UnityTest]
    public IEnumerator ForceTwoRowsWorksInLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: true
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 0.75f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(1f, 0.75f, 2.5f), card2.transform.position);
      AssertVector3Equal(new Vector3(0f, -0.75f, 2.5f), card3.transform.position);
    }

    [UnityTest]
    public IEnumerator CalculateObjectRotationUsesTransformEulerAngles()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );
      layout.transform.rotation = Quaternion.Euler(15f, 30f, 45f);

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(15f, 30f, 45f), card.transform.localEulerAngles);
    }

    [UnityTest]
    public IEnumerator CalculateObjectScaleUsesTransformScaleX()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );
      layout.transform.localScale = new Vector3(0.5f, 0.5f, 0.5f);

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0.5f, 0.5f, 0.5f), card.transform.localScale);
    }

    [UnityTest]
    public IEnumerator PreserveLayoutOnRemovalKeepsInitialPositions()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );
      layout._preserveLayoutOnRemoval = true;

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      layout.RemoveIfPresent(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(2f, 0f, 2.5f), card3.transform.position);
    }

    [UnityTest]
    public IEnumerator PreserveLayoutOnRemovalMaintainsSpacingAfterMultipleRemovals()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );
      layout._preserveLayoutOnRemoval = true;

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      layout.RemoveIfPresent(cards[1]);
      layout.RemoveIfPresent(cards[3]);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-4f, 0f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(4f, 0f, 2.5f), cards[4].transform.position);
    }

    [UnityTest]
    public IEnumerator PreserveLayoutResetsWhenAllCardsRemoved()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );
      layout._preserveLayoutOnRemoval = true;

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      layout.RemoveIfPresent(card1);
      layout.RemoveIfPresent(card2);
      layout.ApplyLayout(sequence: null);

      var newCard1 = CreateDisplayable();
      var newCard2 = CreateDisplayable();
      layout.Add(newCard1);
      layout.Add(newCard2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 0f, 2.5f), newCard1.transform.position);
      AssertVector3Equal(new Vector3(1f, 0f, 2.5f), newCard2.transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutWithDifferentHorizontalSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 4f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-4f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card2.transform.position);
      AssertVector3Equal(new Vector3(4f, 0f, 2.5f), card3.transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutWithDifferentVerticalSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 3f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0f, 1.5f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0f, -1.5f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutAtDifferentZPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 0f, 5f), card1.transform.position);
      AssertVector3Equal(new Vector3(1f, 0f, 5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutWithScaledTransform()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );
      layout.transform.localScale = new Vector3(0.25f, 0.25f, 0.25f);

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card2.transform.position);
      AssertVector3Equal(new Vector3(2f, 0f, 2.5f), card3.transform.position);
      AssertVector3Equal(new Vector3(0.25f, 0.25f, 0.25f), card1.transform.localScale);
    }

    [UnityTest]
    public IEnumerator PortraitLayoutWithMultipleRowsAndDifferentSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 3f,
        verticalSpacing: 2f,
        zPosition: 2.5f,
        cardHeight: 3.5f,
        forceTwoRows: false
      );

      var cards = new TestDisplayable[6];
      for (var i = 0; i < 6; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-3f, 1f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(0f, 1f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(3f, 1f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(-3f, -1f, 2.5f), cards[3].transform.position);
      AssertVector3Equal(new Vector3(0f, -1f, 2.5f), cards[4].transform.position);
      AssertVector3Equal(new Vector3(3f, -1f, 2.5f), cards[5].transform.position);
    }

    SitePickObjectLayout CreateTestLayout(
      float horizontalSpacing,
      float verticalSpacing,
      float zPosition,
      float cardHeight,
      bool forceTwoRows
    )
    {
      var layout = CreateSceneObject<SitePickObjectLayout>(l =>
      {
        l._horizontalSpacing = horizontalSpacing;
        l._verticalSpacing = verticalSpacing;
        l._cardWidth = 2.5f;
        l._cardHeight = cardHeight;
        l._forceTwoRows = forceTwoRows;
      });
      layout.transform.localPosition = new Vector3(0f, 0f, zPosition);
      layout.transform.localRotation = Quaternion.identity;
      layout.GameContext = GameContext.Browser;
      return layout;
    }
  }
}
