#nullable enable

using System.Collections;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class StartBattleObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleCardIsPositionedAtCenter()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(layout.transform.position, card.transform.position);
    }

    [UnityTest]
    public IEnumerator TwoCardsAreHorizontallySpaced()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(card1.transform.position.x, Is.LessThan(card2.transform.position.x));
      Assert.That(card1.transform.position.x, Is.LessThan(0f));
      Assert.That(card2.transform.position.x, Is.GreaterThan(0f));
    }

    [UnityTest]
    public IEnumerator TwoCardsAreSymmetricAroundCenter()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var center = layout.transform.position;
      var leftOffset = center.x - card1.transform.position.x;
      var rightOffset = card2.transform.position.x - center.x;

      Assert.That(leftOffset, Is.EqualTo(rightOffset).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator TwoCardsHaveCorrectScaleLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScaleLandscape;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card1.transform.localScale
      );
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card2.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator TwoCardsHaveCorrectScalePortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScalePortrait;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card1.transform.localScale
      );
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card2.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator TwoCardsHaveCorrectRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(layout.transform.eulerAngles, card1.transform.eulerAngles);
      AssertVector3Equal(layout.transform.eulerAngles, card2.transform.eulerAngles);
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscape16x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscape16x10AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x10);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscape21x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscape32x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution32x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscape3x2AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution3x2);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInLandscape5x4AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution5x4);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitIPhone12AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitIPhoneSEAreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhoneSE);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitIPadPro12AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPadPro12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitIPodTouch6AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPodTouch6);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitSamsungNote20AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionSamsungNote20);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitSamsungZFold2ArePositionedSymmetrically()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionSamsungZFold2);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var center = layout.transform.position;
      var leftOffset = center.x - card1.transform.position.x;
      var rightOffset = card2.transform.position.x - center.x;
      Assert.That(leftOffset, Is.EqualTo(rightOffset).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator TwoCardsInPortraitPixel5AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionPixel5);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateTestCard();
      var card2 = CreateTestCard();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      AssertCardBoxColliderIsOnScreen(viewport, card1, "Left card");
      AssertCardBoxColliderIsOnScreen(viewport, card2, "Right card");
    }

    [UnityTest]
    public IEnumerator SingleCardHasCorrectScaleLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScaleLandscape;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator SingleCardHasCorrectScalePortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      var expectedScale = layout._cardScalePortrait;
      AssertVector3Equal(
        new Vector3(expectedScale, expectedScale, expectedScale),
        card.transform.localScale
      );
    }

    [UnityTest]
    public IEnumerator VsTextIsHiddenAfterLayoutBecomesEmpty()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.gameObject.activeSelf, Is.True);
      }

      layout.RemoveIfPresent(card);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.gameObject.activeSelf, Is.False);
      }
    }

    [UnityTest]
    public IEnumerator VsTextIsShownWhenLayoutIsNonEmpty()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.gameObject.activeSelf, Is.True);
      }
    }

    [UnityTest]
    public IEnumerator VsTextIsHiddenAfterRemovingMultipleCards()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.gameObject.activeSelf, Is.True);
      }

      layout.RemoveIfPresent(card1);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.gameObject.activeSelf, Is.True);
      }

      layout.RemoveIfPresent(card2);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.gameObject.activeSelf, Is.False);
      }
    }

    [UnityTest]
    public IEnumerator VsTextIsPositionedAtLayoutCenter()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      if (layout._vsText != null)
      {
        AssertVector3Equal(layout.transform.position, layout._vsText.transform.position);
      }
    }

    [UnityTest]
    public IEnumerator VsTextHasCorrectFontSizeLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.fontSize, Is.EqualTo(layout._vsTextFontSizeLandscape));
      }
    }

    [UnityTest]
    public IEnumerator VsTextHasCorrectFontSizePortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      if (layout._vsText != null)
      {
        Assert.That(layout._vsText.fontSize, Is.EqualTo(layout._vsTextFontSizePortrait));
      }
    }

    [UnityTest]
    public IEnumerator HideButtonHidesTheButton()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      layout.ShowButton();
      layout.HideButton();

      var buttonField = typeof(StartBattleObjectLayout).GetField(
        "_buttonInstance",
        System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance
      );
      var buttonInstance = buttonField?.GetValue(layout) as MonoBehaviour;
      if (buttonInstance != null)
      {
        Assert.That(buttonInstance.gameObject.activeSelf, Is.False);
      }
    }

    [UnityTest]
    public IEnumerator ShowButtonShowsTheButton()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      layout.ShowButton();

      var buttonField = typeof(StartBattleObjectLayout).GetField(
        "_buttonInstance",
        System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance
      );
      var buttonInstance = buttonField?.GetValue(layout) as MonoBehaviour;
      if (buttonInstance != null)
      {
        Assert.That(buttonInstance.gameObject.activeSelf, Is.True);
      }
    }

    [UnityTest]
    public IEnumerator LeftCardIsToLeftOfCenter()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card1.transform.position.x,
        Is.LessThan(layout.transform.position.x),
        "Left card should be to the left of center"
      );
    }

    [UnityTest]
    public IEnumerator RightCardIsToRightOfCenter()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card2.transform.position.x,
        Is.GreaterThan(layout.transform.position.x),
        "Right card should be to the right of center"
      );
    }

    [UnityTest]
    public IEnumerator CardsShareSameYPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card1.transform.position.y,
        Is.EqualTo(card2.transform.position.y).Within(0.01f),
        "Both cards should be at the same Y position"
      );
    }

    [UnityTest]
    public IEnumerator CardsShareSameZPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card1.transform.position.z,
        Is.EqualTo(card2.transform.position.z).Within(0.01f),
        "Both cards should be at the same Z position"
      );
    }

    [UnityTest]
    public IEnumerator CardsArePositionedAtLayoutYPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card1.transform.position.y,
        Is.EqualTo(layout.transform.position.y).Within(0.01f),
        "Card Y should match layout Y"
      );
      Assert.That(
        card2.transform.position.y,
        Is.EqualTo(layout.transform.position.y).Within(0.01f),
        "Card Y should match layout Y"
      );
    }

    [UnityTest]
    public IEnumerator CardsArePositionedAtLayoutZPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        card1.transform.position.z,
        Is.EqualTo(layout.transform.position.z).Within(0.01f),
        "Card Z should match layout Z"
      );
      Assert.That(
        card2.transform.position.z,
        Is.EqualTo(layout.transform.position.z).Within(0.01f),
        "Card Z should match layout Z"
      );
    }

    [UnityTest]
    public IEnumerator CalculateObjectPositionReturnsLayoutPositionForEmptyLayout()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var position = layout.CalculateObjectPosition(index: 0, count: 0);

      AssertVector3Equal(layout.transform.position, position);
    }

    [UnityTest]
    public IEnumerator CalculateObjectRotationReturnsLayoutRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var rotation = layout.CalculateObjectRotation(index: 0, count: 2);

      Assert.That(rotation, Is.Not.Null);
      AssertVector3Equal(layout.transform.rotation.eulerAngles, rotation!.Value);
    }

    [UnityTest]
    public IEnumerator CalculateObjectScaleReturnsCardScaleLandscape()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var scale = layout.CalculateObjectScale(index: 0, count: 2);

      Assert.That(scale, Is.Not.Null);
      Assert.That(scale!.Value, Is.EqualTo(layout._cardScaleLandscape));
    }

    [UnityTest]
    public IEnumerator CalculateObjectScaleReturnsCardScalePortrait()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var scale = layout.CalculateObjectScale(index: 0, count: 2);

      Assert.That(scale, Is.Not.Null);
      Assert.That(scale!.Value, Is.EqualTo(layout._cardScalePortrait));
    }

    [UnityTest]
    public IEnumerator LandscapeUsesLandscapeInwardOffset()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var expectedLeftX =
        layout.transform.position.x
        - layout._horizontalSpacing / 2f
        + layout._cardInwardOffsetLandscape;
      var expectedRightX =
        layout.transform.position.x
        + layout._horizontalSpacing / 2f
        - layout._cardInwardOffsetLandscape;

      Assert.That(card1.transform.position.x, Is.EqualTo(expectedLeftX).Within(0.01f));
      Assert.That(card2.transform.position.x, Is.EqualTo(expectedRightX).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator PortraitUsesPortraitInwardOffset()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      var card2 = CreateDisplayable();
      layout.Add(card1);
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      var expectedLeftX =
        layout.transform.position.x
        - layout._horizontalSpacing / 2f
        + layout._cardInwardOffsetPortrait;
      var expectedRightX =
        layout.transform.position.x
        + layout._horizontalSpacing / 2f
        - layout._cardInwardOffsetPortrait;

      Assert.That(card1.transform.position.x, Is.EqualTo(expectedLeftX).Within(0.01f));
      Assert.That(card2.transform.position.x, Is.EqualTo(expectedRightX).Within(0.01f));
    }

    [UnityTest]
    public IEnumerator RemovingCardMakesLayoutEmpty()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card = CreateDisplayable();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      Assert.That(layout.Objects.Count, Is.EqualTo(1));

      layout.RemoveIfPresent(card);

      Assert.That(layout.Objects.Count, Is.EqualTo(0));
    }

    [UnityTest]
    public IEnumerator AddingSecondCardUpdatesLayout()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var card1 = CreateDisplayable();
      layout.Add(card1);
      layout.ApplyLayout(sequence: null);

      AssertVector3Equal(layout.transform.position, card1.transform.position);

      var card2 = CreateDisplayable();
      layout.Add(card2);
      layout.ApplyLayout(sequence: null);

      Assert.That(card1.transform.position.x, Is.LessThan(layout.transform.position.x));
      Assert.That(card2.transform.position.x, Is.GreaterThan(layout.transform.position.x));
    }

    [UnityTest]
    public IEnumerator OneDreamsignLandscape16x9IsFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign = CreateDreamsign(isUserSide: true);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign, "User dreamsign 1");
    }

    [UnityTest]
    public IEnumerator TwoDreamsignsLandscape16x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: true);
      var dreamsign2 = CreateDreamsign(isUserSide: true);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "User dreamsign 2");
    }

    [UnityTest]
    public IEnumerator ThreeDreamsignsLandscape16x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: true);
      var dreamsign2 = CreateDreamsign(isUserSide: true);
      var dreamsign3 = CreateDreamsign(isUserSide: true);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.Add(dreamsign3);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "User dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign3, "User dreamsign 3");
    }

    [UnityTest]
    public IEnumerator FourDreamsignsLandscape16x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: true);
      var dreamsign2 = CreateDreamsign(isUserSide: true);
      var dreamsign3 = CreateDreamsign(isUserSide: true);
      var dreamsign4 = CreateDreamsign(isUserSide: true);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.Add(dreamsign3);
      layout.Add(dreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "User dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign3, "User dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign4, "User dreamsign 4");
    }

    [UnityTest]
    public IEnumerator FourDreamsignsPortraitIPhone12AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: true);
      var dreamsign2 = CreateDreamsign(isUserSide: true);
      var dreamsign3 = CreateDreamsign(isUserSide: true);
      var dreamsign4 = CreateDreamsign(isUserSide: true);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.Add(dreamsign3);
      layout.Add(dreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "User dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign3, "User dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign4, "User dreamsign 4");
    }

    [UnityTest]
    public IEnumerator FourDreamsignsPortraitIPhoneSEAreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhoneSE);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: true);
      var dreamsign2 = CreateDreamsign(isUserSide: true);
      var dreamsign3 = CreateDreamsign(isUserSide: true);
      var dreamsign4 = CreateDreamsign(isUserSide: true);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.Add(dreamsign3);
      layout.Add(dreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "User dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign3, "User dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign4, "User dreamsign 4");
    }

    [UnityTest]
    public IEnumerator FourEnemyDreamsignsLandscape16x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: false);
      var dreamsign2 = CreateDreamsign(isUserSide: false);
      var dreamsign3 = CreateDreamsign(isUserSide: false);
      var dreamsign4 = CreateDreamsign(isUserSide: false);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.Add(dreamsign3);
      layout.Add(dreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "Enemy dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "Enemy dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign3, "Enemy dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign4, "Enemy dreamsign 4");
    }

    [UnityTest]
    public IEnumerator FourEnemyDreamsignsPortraitIPhone12AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var dreamsign1 = CreateDreamsign(isUserSide: false);
      var dreamsign2 = CreateDreamsign(isUserSide: false);
      var dreamsign3 = CreateDreamsign(isUserSide: false);
      var dreamsign4 = CreateDreamsign(isUserSide: false);
      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(dreamsign1);
      layout.Add(dreamsign2);
      layout.Add(dreamsign3);
      layout.Add(dreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign1, "Enemy dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign2, "Enemy dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign3, "Enemy dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, dreamsign4, "Enemy dreamsign 4");
    }

    [UnityTest]
    public IEnumerator BothSidesFourDreamsignsLandscape16x9AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var userDreamsign1 = CreateDreamsign(isUserSide: true);
      var userDreamsign2 = CreateDreamsign(isUserSide: true);
      var userDreamsign3 = CreateDreamsign(isUserSide: true);
      var userDreamsign4 = CreateDreamsign(isUserSide: true);
      var enemyDreamsign1 = CreateDreamsign(isUserSide: false);
      var enemyDreamsign2 = CreateDreamsign(isUserSide: false);
      var enemyDreamsign3 = CreateDreamsign(isUserSide: false);
      var enemyDreamsign4 = CreateDreamsign(isUserSide: false);

      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(userDreamsign1);
      layout.Add(userDreamsign2);
      layout.Add(userDreamsign3);
      layout.Add(userDreamsign4);
      layout.Add(enemyDreamsign1);
      layout.Add(enemyDreamsign2);
      layout.Add(enemyDreamsign3);
      layout.Add(enemyDreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign2, "User dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign3, "User dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign4, "User dreamsign 4");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign1, "Enemy dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign2, "Enemy dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign3, "Enemy dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign4, "Enemy dreamsign 4");
    }

    [UnityTest]
    public IEnumerator BothSidesFourDreamsignsPortraitIPhone12AreFullyOnScreen()
    {
      var viewport = CreateViewport(GameViewResolution.ResolutionIPhone12);
      yield return Initialize(viewport);
      var layout = GetStartBattleLayout();

      var identityCard1 = CreateTestCard();
      var identityCard2 = CreateTestCard();
      SetIdentityCardPosition(identityCard1, isUserSide: true);
      SetIdentityCardPosition(identityCard2, isUserSide: false);

      var userDreamsign1 = CreateDreamsign(isUserSide: true);
      var userDreamsign2 = CreateDreamsign(isUserSide: true);
      var userDreamsign3 = CreateDreamsign(isUserSide: true);
      var userDreamsign4 = CreateDreamsign(isUserSide: true);
      var enemyDreamsign1 = CreateDreamsign(isUserSide: false);
      var enemyDreamsign2 = CreateDreamsign(isUserSide: false);
      var enemyDreamsign3 = CreateDreamsign(isUserSide: false);
      var enemyDreamsign4 = CreateDreamsign(isUserSide: false);

      layout.Add(identityCard1);
      layout.Add(identityCard2);
      layout.Add(userDreamsign1);
      layout.Add(userDreamsign2);
      layout.Add(userDreamsign3);
      layout.Add(userDreamsign4);
      layout.Add(enemyDreamsign1);
      layout.Add(enemyDreamsign2);
      layout.Add(enemyDreamsign3);
      layout.Add(enemyDreamsign4);
      layout.ApplyLayout(sequence: null);

      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign1, "User dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign2, "User dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign3, "User dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, userDreamsign4, "User dreamsign 4");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign1, "Enemy dreamsign 1");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign2, "Enemy dreamsign 2");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign3, "Enemy dreamsign 3");
      AssertDreamsignBoxColliderIsOnScreen(viewport, enemyDreamsign4, "Enemy dreamsign 4");
    }

    StartBattleObjectLayout GetStartBattleLayout()
    {
      var layout = Registry.DreamscapeLayout.StartBattleLayout;
      layout.GameContext = GameContext.Interface;
      return layout;
    }

    Card CreateDreamsign(bool isUserSide)
    {
      var dreamsign = CreateTestCard();
      dreamsign.ObjectPosition = new ObjectPosition
      {
        Position = new PositionClass
        {
          StartBattleDisplay = isUserSide
            ? StartBattleDisplayType.UserDreamsigns
            : StartBattleDisplayType.EnemyDreamsigns,
        },
        SortingKey = 0,
      };
      return dreamsign;
    }

    void SetIdentityCardPosition(Card card, bool isUserSide)
    {
      card.ObjectPosition = new ObjectPosition
      {
        Position = new PositionClass
        {
          StartBattleDisplay = isUserSide
            ? StartBattleDisplayType.UserIdentityCard
            : StartBattleDisplayType.EnemyIdentityCard,
        },
        SortingKey = 0,
      };
    }

    void AssertDreamsignBoxColliderIsOnScreen(
      IGameViewport viewport,
      Card dreamsign,
      string description
    )
    {
      var collider = dreamsign.CardCollider;
      var center = collider.center;
      var extents = collider.size * 0.5f;

      var localCorners = new Vector3[8];
      localCorners[0] = center + new Vector3(-extents.x, -extents.y, -extents.z);
      localCorners[1] = center + new Vector3(-extents.x, -extents.y, extents.z);
      localCorners[2] = center + new Vector3(-extents.x, extents.y, -extents.z);
      localCorners[3] = center + new Vector3(-extents.x, extents.y, extents.z);
      localCorners[4] = center + new Vector3(extents.x, -extents.y, -extents.z);
      localCorners[5] = center + new Vector3(extents.x, -extents.y, extents.z);
      localCorners[6] = center + new Vector3(extents.x, extents.y, -extents.z);
      localCorners[7] = center + new Vector3(extents.x, extents.y, extents.z);

      for (var i = 0; i < localCorners.Length; i++)
      {
        var worldCorner = dreamsign.transform.TransformPoint(localCorners[i]);
        AssertPointIsOnScreen(viewport, worldCorner, $"{description} box collider corner {i}");
      }
    }
  }
}
