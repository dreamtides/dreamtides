#nullable enable

using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Tests.TestUtils;
using NUnit.Framework;
using UnityEngine;
using UnityEngine.TestTools;
using UnityEngine.UI;

namespace Dreamtides.Tests.Components
{
  [TestFixture]
  public class DreamscapeSiteButtonPositionerTests : DreamtidesUnitTest
  {
    [UnityTest]
    public IEnumerator PositionsDoNotOverlapAcrossResolutions(
      [ValueSource(nameof(CommonResolutions))] GameViewResolution resolution
    )
    {
      var canvasRoot = CreateCanvasRoot();
      var viewport = CreateViewport(
        resolution,
        cameraTransform: CreateGameObject().transform,
        canvasRootRect: canvasRoot,
        canvasPixelRect: CreateCanvasPixelRect(resolution)
      );
      var safeArea = CreateSafeArea(
        canvasRoot,
        viewport.SafeAreaMinimumAnchor,
        viewport.SafeAreaMaximumAnchor
      );
      var positioner = new DreamscapeSiteButtonPositioner(viewport, safeArea);
      var worldPositions = CreateWorldPositions(
        viewport,
        new List<Vector2>
        {
          new Vector2(0.3f, 0.52f),
          new Vector2(0.5f, 0.5f),
          new Vector2(0.7f, 0.48f),
          new Vector2(0.45f, 0.46f),
        }
      );
      var buttons = CreateButtons(safeArea, worldPositions.Count, new Vector2(20f, 20f));

      var positions = positioner.PositionButtons(worldPositions, buttons);

      AssertWithinBounds(positions, buttons, safeArea);
      AssertNoOverlap(positions, buttons, safeArea);
      yield return null;
    }

    [UnityTest]
    public IEnumerator PositionsAreDeterministic()
    {
      var canvasRoot = CreateCanvasRoot();
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        cameraTransform: CreateGameObject().transform,
        canvasRootRect: canvasRoot,
        canvasPixelRect: CreateCanvasPixelRect(GameViewResolution.Resolution16x9)
      );
      var safeArea = CreateSafeArea(
        canvasRoot,
        viewport.SafeAreaMinimumAnchor,
        viewport.SafeAreaMaximumAnchor
      );
      var positioner = new DreamscapeSiteButtonPositioner(viewport, safeArea);
      var worldPositions = CreateWorldPositions(
        viewport,
        new List<Vector2> { new Vector2(0.5f, 0.5f), new Vector2(0.51f, 0.5f) }
      );
      var buttons = CreateButtons(safeArea, worldPositions.Count, new Vector2(20f, 20f));

      var first = positioner.PositionButtons(worldPositions, buttons);
      var second = positioner.PositionButtons(worldPositions, buttons);

      AssertSequencesEqual(first, second);
      yield return null;
    }

    [UnityTest]
    public IEnumerator PositionsAppearAboveProjectedSites()
    {
      var canvasRoot = CreateCanvasRoot();
      var viewport = CreateViewport(
        GameViewResolution.Resolution16x9,
        cameraTransform: CreateGameObject().transform,
        canvasRootRect: canvasRoot,
        canvasPixelRect: CreateCanvasPixelRect(GameViewResolution.Resolution16x9)
      );
      var safeArea = CreateSafeArea(
        canvasRoot,
        viewport.SafeAreaMinimumAnchor,
        viewport.SafeAreaMaximumAnchor
      );
      var positioner = new DreamscapeSiteButtonPositioner(viewport, safeArea);
      var worldPositions = CreateWorldPositions(
        viewport,
        new List<Vector2> { new Vector2(0.25f, 0.25f), new Vector2(0.75f, 0.75f) }
      );
      var buttons = CreateButtons(safeArea, worldPositions.Count, new Vector2(20f, 20f));

      var positions = positioner.PositionButtons(worldPositions, buttons);
      var anchors = GetProjectedAnchors(viewport, canvasRoot, safeArea, worldPositions);
      var halfSizes = GetHalfSizes(safeArea, buttons);
      for (var i = 0; i < positions.Count; i++)
      {
        Assert.That(positions[i].y, Is.GreaterThan(anchors[i].y + halfSizes[i].y - 0.001f));
      }
      yield return null;
    }

    static readonly GameViewResolution[] CommonResolutions =
    {
      GameViewResolution.Resolution16x9,
      GameViewResolution.Resolution21x9,
      GameViewResolution.Resolution3x2,
      GameViewResolution.ResolutionIPhone12,
      GameViewResolution.ResolutionIPhoneSE,
    };

    RectTransform CreateCanvasRoot()
    {
      var canvasRoot = CreateGameObject().AddComponent<RectTransform>();
      canvasRoot.sizeDelta = new Vector2(225f, 400f);
      canvasRoot.anchorMin = new Vector2(0.5f, 0.5f);
      canvasRoot.anchorMax = new Vector2(0.5f, 0.5f);
      canvasRoot.anchoredPosition = Vector2.zero;
      canvasRoot.localScale = Vector3.one;
      var canvas = canvasRoot.gameObject.AddComponent<Canvas>();
      canvas.renderMode = RenderMode.ScreenSpaceOverlay;
      var scaler = canvasRoot.gameObject.AddComponent<CanvasScaler>();
      scaler.uiScaleMode = CanvasScaler.ScaleMode.ScaleWithScreenSize;
      scaler.referenceResolution = new Vector2(225f, 400f);
      scaler.screenMatchMode = CanvasScaler.ScreenMatchMode.MatchWidthOrHeight;
      scaler.matchWidthOrHeight = 1f;
      return canvasRoot;
    }

    RectTransform CreateSafeArea(RectTransform canvasRoot, Vector2 anchorMin, Vector2 anchorMax)
    {
      var safeArea = CreateGameObject().AddComponent<RectTransform>();
      safeArea.SetParent(canvasRoot, worldPositionStays: false);
      safeArea.anchorMin = anchorMin;
      safeArea.anchorMax = anchorMax;
      safeArea.offsetMin = Vector2.zero;
      safeArea.offsetMax = Vector2.zero;
      safeArea.pivot = new Vector2(0.5f, 0.5f);
      safeArea.localScale = Vector3.one;
      return safeArea;
    }

    Rect CreateCanvasPixelRect(GameViewResolution resolution)
    {
      var size = resolution.AsVector();
      return new Rect(0f, 0f, size.x, size.y);
    }

    List<RectTransform> CreateButtons(RectTransform parent, int count, Vector2 size)
    {
      var buttons = new List<RectTransform>(count);
      for (var i = 0; i < count; i++)
      {
        var button = CreateGameObject().AddComponent<RectTransform>();
        button.SetParent(parent, worldPositionStays: false);
        button.sizeDelta = size;
        button.anchorMin = new Vector2(0.5f, 0.5f);
        button.anchorMax = new Vector2(0.5f, 0.5f);
        button.anchoredPosition = Vector2.zero;
        buttons.Add(button);
      }
      return buttons;
    }

    List<Vector3> CreateWorldPositions(
      FakeViewport viewport,
      IReadOnlyList<Vector2> viewportPositions
    )
    {
      var positions = new List<Vector3>(viewportPositions.Count);
      var screenWidth = viewport.ScreenWidth;
      var screenHeight = viewport.ScreenHeight;
      for (var i = 0; i < viewportPositions.Count; i++)
      {
        var viewportPosition = viewportPositions[i];
        var screenPoint = new Vector3(
          viewportPosition.x * screenWidth,
          viewportPosition.y * screenHeight,
          10f
        );
        positions.Add(viewport.ScreenToWorldPoint(screenPoint));
      }
      return positions;
    }

    List<Vector2> GetProjectedAnchors(
      FakeViewport viewport,
      RectTransform canvasRoot,
      RectTransform safeArea,
      IReadOnlyList<Vector3> worldPositions
    )
    {
      var anchors = new List<Vector2>(worldPositions.Count);
      var canvasRect = canvasRoot.rect;
      for (var i = 0; i < worldPositions.Count; i++)
      {
        var viewportPoint = viewport.WorldToViewportPoint(worldPositions[i]);
        var canvasPosition = new Vector2(
          Mathf.Lerp(canvasRect.xMin, canvasRect.xMax, viewportPoint.x),
          Mathf.Lerp(canvasRect.yMin, canvasRect.yMax, viewportPoint.y)
        );
        var worldOnCanvas = canvasRoot.TransformPoint(
          new Vector3(canvasPosition.x, canvasPosition.y, 0f)
        );
        var safeLocal = safeArea.InverseTransformPoint(worldOnCanvas);
        anchors.Add(new Vector2(safeLocal.x, safeLocal.y));
      }
      return anchors;
    }

    List<Vector2> GetHalfSizes(RectTransform parent, IReadOnlyList<RectTransform> buttons)
    {
      var sizes = new List<Vector2>(buttons.Count);
      for (var i = 0; i < buttons.Count; i++)
      {
        var bounds = RectTransformUtility.CalculateRelativeRectTransformBounds(parent, buttons[i]);
        var extents = bounds.extents;
        sizes.Add(new Vector2(extents.x, extents.y));
      }
      return sizes;
    }

    void AssertWithinBounds(
      IReadOnlyList<Vector2> positions,
      IReadOnlyList<RectTransform> buttons,
      RectTransform safeArea
    )
    {
      var halfSizes = GetHalfSizes(safeArea, buttons);
      var rect = safeArea.rect;
      for (var i = 0; i < positions.Count; i++)
      {
        var position = positions[i];
        var halfSize = halfSizes[i];
        Assert.That(position.x, Is.GreaterThanOrEqualTo(rect.xMin + halfSize.x - 0.001f));
        Assert.That(position.x, Is.LessThanOrEqualTo(rect.xMax - halfSize.x + 0.001f));
        Assert.That(position.y, Is.GreaterThanOrEqualTo(rect.yMin + halfSize.y - 0.001f));
        Assert.That(position.y, Is.LessThanOrEqualTo(rect.yMax - halfSize.y + 0.001f));
      }
    }

    void AssertNoOverlap(
      IReadOnlyList<Vector2> positions,
      IReadOnlyList<RectTransform> buttons,
      RectTransform parent
    )
    {
      var halfSizes = GetHalfSizes(parent, buttons);
      for (var i = 0; i < positions.Count; i++)
      {
        for (var j = i + 1; j < positions.Count; j++)
        {
          var dx = Mathf.Abs(positions[i].x - positions[j].x);
          var dy = Mathf.Abs(positions[i].y - positions[j].y);
          var sumX = halfSizes[i].x + halfSizes[j].x;
          var sumY = halfSizes[i].y + halfSizes[j].y;
          var overlaps = dx < sumX - 0.001f && dy < sumY - 0.001f;
          Assert.That(overlaps, Is.False);
        }
      }
    }

    void AssertSequencesEqual(IReadOnlyList<Vector2> expected, IReadOnlyList<Vector2> actual)
    {
      Assert.That(actual.Count, Is.EqualTo(expected.Count));
      for (var i = 0; i < expected.Count; i++)
      {
        Assert.That(actual[i].x, Is.EqualTo(expected[i].x).Within(0.0001f));
        Assert.That(actual[i].y, Is.EqualTo(expected[i].y).Within(0.0001f));
      }
    }
  }
}
