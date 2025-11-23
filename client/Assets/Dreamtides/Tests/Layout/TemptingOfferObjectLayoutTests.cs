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
  public class TemptingOfferObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardIsCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0f, 0f, 2.5f), card.transform.position);
    }

    [UnityTest]
    public IEnumerator TwoCardsFormSingleRowWithHorizontalSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 3f,
        verticalSpacing: 2f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1.5f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(1.5f, 0f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator ThreeCardsFormTwoRows()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 4f,
        verticalSpacing: 3f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      var card3 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.Add(card3);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2f, 1.5f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(2f, 1.5f, 2.5f), card2.transform.position);
      AssertVector3Equal(new Vector3(0f, -1.5f, 2.5f), card3.transform.position);
    }

    [UnityTest]
    public IEnumerator FourCardsFormTwoEvenRows()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 3f,
        verticalSpacing: 2f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var cards = new TestDisplayable[4];
      for (var i = 0; i < 4; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1.5f, 1f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(1.5f, 1f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-1.5f, -1f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1.5f, -1f, 2.5f), cards[3].transform.position);
    }

    [UnityTest]
    public IEnumerator FiveCardsFormThreeRows()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var cards = new TestDisplayable[5];
      for (var i = 0; i < 5; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 1.5f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(1f, 1.5f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-1f, 0f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1f, 0f, 2.5f), cards[3].transform.position);
      AssertVector3Equal(new Vector3(0f, -1.5f, 2.5f), cards[4].transform.position);
    }

    [UnityTest]
    public IEnumerator SixCardsFormThreeEvenRows()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2.5f,
        verticalSpacing: 2f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var cards = new TestDisplayable[6];
      for (var i = 0; i < 6; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1.25f, 2f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(1.25f, 2f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-1.25f, 0f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1.25f, 0f, 2.5f), cards[3].transform.position);
      AssertVector3Equal(new Vector3(-1.25f, -2f, 2.5f), cards[4].transform.position);
      AssertVector3Equal(new Vector3(1.25f, -2f, 2.5f), cards[5].transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutWithDifferentHorizontalSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 6f,
        verticalSpacing: 2f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-3f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(3f, 0f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutWithDifferentVerticalSpacing()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 4f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var cards = new TestDisplayable[4];
      for (var i = 0; i < 4; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 2f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(1f, 2f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-1f, -2f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1f, -2f, 2.5f), cards[3].transform.position);
    }

    [UnityTest]
    public IEnumerator LayoutAtDifferentZPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 5f,
        cardHeight: 3.5f
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
    public IEnumerator LandscapeHorizontalSpacingOverrideApplies()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );
      layout._landscapeHorizontalSpacingOverride = 5f;

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2.5f, 0f, 2.5f), card1.transform.position);
      AssertVector3Equal(new Vector3(2.5f, 0f, 2.5f), card2.transform.position);
    }

    [UnityTest]
    public IEnumerator LandscapeVerticalSpacingOverrideApplies()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );
      layout._landscapeVerticalSpacingOverride = 3f;

      var cards = new TestDisplayable[4];
      for (var i = 0; i < 4; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1f, 1.5f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(1f, 1.5f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-1f, -1.5f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1f, -1.5f, 2.5f), cards[3].transform.position);
    }

    [UnityTest]
    public IEnumerator CalculateObjectRotationUsesTransformEulerAngles()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );
      layout.transform.rotation = Quaternion.Euler(10f, 20f, 30f);

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(10f, 20f, 30f), card.transform.localEulerAngles);
    }

    [UnityTest]
    public IEnumerator CalculateObjectScaleUsesTransformScaleX()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 2f,
        verticalSpacing: 1.5f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );
      layout.transform.localScale = new Vector3(0.75f, 0.75f, 0.75f);

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(0.75f, 0.75f, 0.75f), card.transform.localScale);
    }

    [UnityTest]
    public IEnumerator PreserveLayoutOnRemovalWorks()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 3f,
        verticalSpacing: 2f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );
      layout._preserveLayoutOnRemoval = true;

      var cards = new TestDisplayable[4];
      for (var i = 0; i < 4; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      layout.RemoveIfPresent(cards[1]);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-1.5f, 1f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(-1.5f, -1f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(1.5f, -1f, 2.5f), cards[3].transform.position);
    }

    [UnityTest]
    public IEnumerator OddNumberOfCardsInLastRowCentered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = CreateTestLayout(
        horizontalSpacing: 4f,
        verticalSpacing: 3f,
        zPosition: 2.5f,
        cardHeight: 3.5f
      );

      var cards = new TestDisplayable[7];
      for (var i = 0; i < 7; i++)
      {
        cards[i] = CreateDisplayable();
        layout.Add(cards[i]);
      }
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(new Vector3(-2f, 4.5f, 2.5f), cards[0].transform.position);
      AssertVector3Equal(new Vector3(2f, 4.5f, 2.5f), cards[1].transform.position);
      AssertVector3Equal(new Vector3(-2f, 1.5f, 2.5f), cards[2].transform.position);
      AssertVector3Equal(new Vector3(2f, 1.5f, 2.5f), cards[3].transform.position);
      AssertVector3Equal(new Vector3(-2f, -1.5f, 2.5f), cards[4].transform.position);
      AssertVector3Equal(new Vector3(2f, -1.5f, 2.5f), cards[5].transform.position);
      AssertVector3Equal(new Vector3(0f, -4.5f, 2.5f), cards[6].transform.position);
    }

    TemptingOfferObjectLayout CreateTestLayout(
      float horizontalSpacing,
      float verticalSpacing,
      float zPosition,
      float cardHeight
    )
    {
      var layout = CreateSceneObject<TemptingOfferObjectLayout>(l =>
      {
        l._horizontalSpacing = horizontalSpacing;
        l._verticalSpacing = verticalSpacing;
        l._cardWidth = 2.5f;
        l._cardHeight = cardHeight;
        l._forceTwoRows = false;
      });
      layout.transform.localPosition = new Vector3(0f, 0f, zPosition);
      layout.transform.localRotation = Quaternion.identity;
      layout.GameContext = GameContext.Browser;
      return layout;
    }
  }
}
