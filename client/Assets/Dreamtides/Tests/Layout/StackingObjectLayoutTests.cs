#nullable enable

using System.Collections;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;

namespace Dreamtides.Tests.Layout
{
  [TestFixture]
  public class StackingObjectLayoutTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator SingleObjectIsAtLayoutPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(GameViewResolution.Resolution16x9);

      var layout = CreateStackingLayout(stackRight: true);
      var displayable = CreateDisplayable();
      layout.Add(displayable);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        Vector3.Distance(displayable.transform.position, layout.transform.position),
        Is.LessThan(0.01f),
        "Single object should be at layout origin"
      );
    }

    [UnityTest]
    public IEnumerator MultipleObjectsStackHorizontallyOnScreen_NoRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 0f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator MultipleObjectsStackHorizontallyOnScreen_90DegreeRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 90f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator MultipleObjectsStackHorizontallyOnScreen_120DegreeRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 120f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator MultipleObjectsStackHorizontallyOnScreen_180DegreeRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 180f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator MultipleObjectsStackHorizontallyOnScreen_270DegreeRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator FiveObjectsStackHorizontally_270DegreeRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator StackLeftAnchorsMaintainsHorizontalOrder_NoRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 0f
      );

      var layout = CreateStackingLayout(stackRight: false);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);

      var lastCardPos = viewport.WorldToScreenPoint(cards[2].GetCardCenter());
      var layoutScreenPos = viewport.WorldToScreenPoint(layout.transform.position);
      Assert.That(
        Mathf.Abs(lastCardPos.x - layoutScreenPos.x),
        Is.LessThan(50f),
        "When stackRight=false, the last card should be near the layout origin"
      );
    }

    [UnityTest]
    public IEnumerator StackLeftAnchorsMaintainsHorizontalOrder_270DegreeRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: false);
      var cards = CreateTestCards(count: 3);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);

      var lastCardPos = viewport.WorldToScreenPoint(cards[2].GetCardCenter());
      var layoutScreenPos = viewport.WorldToScreenPoint(layout.transform.position);
      Assert.That(
        Mathf.Abs(lastCardPos.x - layoutScreenPos.x),
        Is.LessThan(50f),
        "When stackRight=false, the last card should be near the layout origin"
      );
    }

    [UnityTest]
    public IEnumerator ObjectsAreEvenlySpaced()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(GameViewResolution.Resolution16x9);

      var layout = CreateStackingLayout(stackRight: true, offset: 2f);
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());
      var screenPos2 = viewport.WorldToScreenPoint(cards[2].GetCardCenter());
      var screenPos3 = viewport.WorldToScreenPoint(cards[3].GetCardCenter());

      var spacing01 = screenPos1.x - screenPos0.x;
      var spacing12 = screenPos2.x - screenPos1.x;
      var spacing23 = screenPos3.x - screenPos2.x;

      Assert.That(
        spacing12,
        Is.EqualTo(spacing01).Within(spacing01 * 0.1f),
        "Objects should have consistent spacing"
      );
      Assert.That(
        spacing23,
        Is.EqualTo(spacing12).Within(spacing12 * 0.1f),
        "Objects should have consistent spacing"
      );
    }

    [UnityTest]
    public IEnumerator ShrinkOffsetAppliesAboveThreshold()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(GameViewResolution.Resolution16x9);

      var layout = CreateStackingLayout(
        stackRight: true,
        offset: 2f,
        shrinkOffset: 1f,
        shrinkOffsetThreshold: 3
      );

      var cards = CreateTestCards(count: 5);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(cards[0].GetCardCenter());
      var screenPos1 = viewport.WorldToScreenPoint(cards[1].GetCardCenter());

      var layoutBelowThreshold = CreateStackingLayout(
        stackRight: true,
        offset: 2f,
        shrinkOffset: 1f,
        shrinkOffsetThreshold: 10
      );
      var cardsBelowThreshold = CreateTestCards(count: 5);
      foreach (var card in cardsBelowThreshold)
      {
        layoutBelowThreshold.Add(card);
      }
      layoutBelowThreshold.ApplyLayout(sequence: null);

      var belowScreenPos0 = viewport.WorldToScreenPoint(cardsBelowThreshold[0].GetCardCenter());
      var belowScreenPos1 = viewport.WorldToScreenPoint(cardsBelowThreshold[1].GetCardCenter());

      var spacingAboveThreshold = screenPos1.x - screenPos0.x;
      var spacingBelowThreshold = belowScreenPos1.x - belowScreenPos0.x;

      Assert.That(
        spacingAboveThreshold,
        Is.LessThan(spacingBelowThreshold),
        "Spacing should shrink when above threshold"
      );
    }

    [UnityTest]
    public IEnumerator ObjectsAreHorizontallyOrdered_Resolution21x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution21x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution21x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator ObjectsAreHorizontallyOrdered_Resolution32x9()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution32x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution32x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator ObjectsAreHorizontallyOrdered_Resolution16x10()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x10);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x10,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator ObjectsAreHorizontallyOrdered_Resolution3x2()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution3x2);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution3x2,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator ObjectsAreHorizontallyOrdered_Resolution5x4()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution5x4);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution5x4,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true);
      var cards = CreateTestCards(count: 4);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator ObjectRotationMatchesLayoutRotation()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForStackingLayout(GameViewResolution.Resolution16x9, gameLayoutYRotation: 120f);

      var layout = CreateStackingLayout(stackRight: true);
      var card = CreateTestCard();
      layout.Add(card);
      layout.ApplyLayout(sequence: null);

      var layoutEuler = layout.transform.rotation.eulerAngles;
      var cardEuler = card.transform.localEulerAngles;

      Assert.That(
        Mathf.DeltaAngle(cardEuler.x, layoutEuler.x),
        Is.EqualTo(0f).Within(1f),
        "Card X rotation should match layout rotation"
      );
      Assert.That(
        Mathf.DeltaAngle(cardEuler.y, layoutEuler.y),
        Is.EqualTo(0f).Within(1f),
        "Card Y rotation should match layout rotation"
      );
      Assert.That(
        Mathf.DeltaAngle(cardEuler.z, layoutEuler.z),
        Is.EqualTo(0f).Within(1f),
        "Card Z rotation should match layout rotation"
      );
    }

    [UnityTest]
    public IEnumerator ObjectScaleMatchesLayoutScale()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForStackingLayout(GameViewResolution.Resolution16x9);

      var layout = CreateStackingLayout(stackRight: true, scale: 0.75f);
      var displayable = CreateDisplayable();
      layout.Add(displayable);
      layout.ApplyLayout(sequence: null);

      Assert.That(
        displayable.transform.localScale.x,
        Is.EqualTo(0.75f).Within(0.01f),
        "Object scale should match layout scale"
      );
    }

    [UnityTest]
    public IEnumerator ManyObjectsAreStillHorizontallyOrdered()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(
        stackRight: true,
        offset: 1.5f,
        shrinkOffset: 0.75f,
        shrinkOffsetThreshold: 8
      );
      var cards = CreateTestCards(count: 12);
      foreach (var card in cards)
      {
        layout.Add(card);
      }
      layout.ApplyLayout(sequence: null);

      AssertCardsAreHorizontallyOrdered(viewport, cards);
    }

    [UnityTest]
    public IEnumerator PositionsUseLocalRight_WouldHaveDetectedBug()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      viewport = CreateViewportForStackingLayout(
        GameViewResolution.Resolution16x9,
        gameLayoutYRotation: 270f
      );

      var layout = CreateStackingLayout(stackRight: true, offset: 2f);
      var displayables = new TestDisplayable[3];
      for (var i = 0; i < 3; i++)
      {
        displayables[i] = CreateDisplayable();
        layout.Add(displayables[i]);
      }
      layout.ApplyLayout(sequence: null);

      var screenPos0 = viewport.WorldToScreenPoint(displayables[0].transform.position);
      var screenPos1 = viewport.WorldToScreenPoint(displayables[1].transform.position);
      var screenPos2 = viewport.WorldToScreenPoint(displayables[2].transform.position);

      Assert.That(
        screenPos1.x,
        Is.GreaterThan(screenPos0.x),
        "Second object should be to the right of first object on screen when rotated"
      );
      Assert.That(
        screenPos2.x,
        Is.GreaterThan(screenPos1.x),
        "Third object should be to the right of second object on screen when rotated"
      );

      var screenYVariance = Mathf.Max(
        Mathf.Abs(screenPos1.y - screenPos0.y),
        Mathf.Abs(screenPos2.y - screenPos1.y)
      );
      var screenXSpacing = Mathf.Min(screenPos1.x - screenPos0.x, screenPos2.x - screenPos1.x);

      Assert.That(
        screenYVariance,
        Is.LessThan(screenXSpacing * 0.5f),
        "Objects should stack primarily horizontally, not diagonally (bug symptom: stacking down and to the left)"
      );
    }

    [UnityTest]
    public IEnumerator ObjectsStackAlongLocalRightAxis()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForStackingLayout(GameViewResolution.Resolution16x9, gameLayoutYRotation: 90f);

      var layout = CreateStackingLayout(stackRight: true, offset: 2f);
      var displayables = new TestDisplayable[3];
      for (var i = 0; i < 3; i++)
      {
        displayables[i] = CreateDisplayable();
        layout.Add(displayables[i]);
      }
      layout.ApplyLayout(sequence: null);

      var expectedDirection = layout.transform.right.normalized;
      var actualDirection = (
        displayables[1].transform.position - displayables[0].transform.position
      ).normalized;

      Assert.That(
        Vector3.Dot(expectedDirection, actualDirection),
        Is.GreaterThan(0.99f),
        "Objects should stack along the layout's local right axis"
      );
    }

    [UnityTest]
    public IEnumerator EmptyLayoutProducesValidPosition()
    {
      var viewport = CreateViewport(GameViewResolution.Resolution16x9);
      yield return Initialize(viewport);
      CreateViewportForStackingLayout(GameViewResolution.Resolution16x9);

      var layout = CreateStackingLayout(stackRight: true);

      var position = layout.CalculateObjectPosition(index: 0, count: 0);

      Assert.That(
        Vector3.Distance(position, layout.transform.position),
        Is.LessThan(0.01f),
        "Empty layout should return transform position"
      );
    }

    FakeViewport CreateViewportForStackingLayout(
      GameViewResolution resolution,
      float? gameLayoutYRotation = null
    )
    {
      if (gameLayoutYRotation.HasValue)
      {
        LandscapeLayout.transform.rotation = Quaternion.Euler(0f, gameLayoutYRotation.Value, 0f);
      }

      var cameraPosition = LandscapeLayout.CameraPosition;
      var cameraGo = CreateGameObject();
      cameraGo.transform.position = cameraPosition.position;
      cameraGo.transform.rotation = cameraPosition.rotation;

      var canvasRootRect = CreateGameObject().AddComponent<RectTransform>();
      return CreateViewport(resolution, cameraGo.transform, canvasRootRect);
    }

    StackingObjectLayout CreateStackingLayout(
      bool stackRight,
      float offset = 1.5f,
      float shrinkOffset = 0.75f,
      int shrinkOffsetThreshold = 8,
      float scale = 1f
    )
    {
      var go = CreateGameObject();
      go.transform.SetParent(LandscapeLayout.Contents.transform, worldPositionStays: false);
      go.transform.localPosition = new Vector3(0f, 5f, 0f);
      go.transform.localRotation = Quaternion.Euler(75f, 90f, 0f);
      go.transform.localScale = Vector3.one * scale;

      var layout = go.AddComponent<StackingObjectLayout>();
      layout._offset = offset;
      layout._shrinkOffset = shrinkOffset;
      layout._shrinkOffsetThreshold = shrinkOffsetThreshold;
      layout._stackRight = stackRight;
      layout.GameContext = GameContext.Stack;

      return layout;
    }
  }
}
