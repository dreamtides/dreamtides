#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Schema;
using Dreamtides.Services;
using Dreamtides.TestFakes;
using NUnit.Framework;
using UnityEditor;
using UnityEngine;

namespace Dreamtides.Tests.TestUtils
{
  public class TestDisplayable : Displayable { }

  public abstract class DreamtidesUnitTest
  {
    readonly List<GameObject> _createdObjects = new();
    int _testCardCounter;

    Registry? _registry;
    BattleLayout? _portraitLayout;
    BattleLayout? _landscapeLayout;
    GeneratedMainCamera? _mainCamera;
    GameMode _gameMode = GameMode.Quest;
    TestConfiguration? _testConfiguration;
    FakeSoundService? _fakeSoundService;
    FakeActionService? _fakeActionService;

    protected Registry Registry =>
      _registry ?? throw new InvalidOperationException("Registry not initialized");

    protected BattleLayout LandscapeLayout =>
      _landscapeLayout ?? throw new InvalidOperationException("Landscape layout not initialized");

    protected BattleLayout PortraitLayout =>
      _portraitLayout ?? throw new InvalidOperationException("Portrait layout not initialized");

    protected TestConfiguration TestConfiguration =>
      _testConfiguration
      ?? throw new InvalidOperationException("Test configuration not initialized");

    protected FakeSoundService FakeSoundService =>
      _fakeSoundService ?? throw new InvalidOperationException("FakeSoundService not initialized");

    protected FakeActionService FakeActionService =>
      _fakeActionService
      ?? throw new InvalidOperationException("FakeActionService not initialized");

    protected GeneratedMainCamera MainCamera =>
      _mainCamera ?? throw new InvalidOperationException("MainCamera not initialized");

    protected IEnumerator Initialize(IGameViewport? viewport = null)
    {
      var canvas = GeneratedCanvas.Create(_createdObjects);
      _mainCamera = GeneratedMainCamera.Create(_createdObjects);
      _portraitLayout = GeneratedPortraitBattleLayout.Create(_createdObjects, canvas);
      _landscapeLayout = GeneratedLandscapeBattleLayout.Create(_createdObjects, canvas);
      var dreamscapeLayout = GeneratedDreamscapeLayout.Create(_createdObjects);

      var generatedRegistry = GeneratedRegistry.Create(
        _createdObjects,
        canvas,
        _mainCamera,
        _portraitLayout,
        _landscapeLayout,
        dreamscapeLayout
      );
      _registry = generatedRegistry.Registry;
      _fakeSoundService = generatedRegistry.FakeSoundService;
      _fakeActionService = generatedRegistry.FakeActionService;

      GeneratedSites.Create(_createdObjects);
      _testConfiguration = new TestConfiguration(Guid.NewGuid());
      return _registry.RunAwake(
        _gameMode,
        _testConfiguration,
        viewport ?? CreateViewport(GameViewResolution.Resolution16x9)
      );
    }

    protected static FakeViewport CreateViewport(
      GameViewResolution resolution,
      Transform cameraTransform,
      RectTransform canvasRootRect,
      Vector2? safeAreaMinimumAnchor = null,
      Vector2? safeAreaMaximumAnchor = null,
      Rect? canvasPixelRect = null
    )
    {
      var size = resolution.AsVector();
      var rect = canvasPixelRect ?? new Rect(0f, 0f, size.x, size.y);
      return new FakeViewport(
        size,
        cameraTransform,
        60f,
        canvasRootRect,
        rect,
        safeAreaMinimumAnchor,
        safeAreaMaximumAnchor
      );
    }

    protected FakeViewport CreateViewport(
      GameViewResolution resolution,
      Vector2? safeAreaMinimumAnchor = null,
      Vector2? safeAreaMaximumAnchor = null,
      Rect? canvasPixelRect = null,
      RectTransform? canvasRootRect = null
    )
    {
      var camera = CreateGameObject().transform;
      canvasRootRect ??= CreateGameObject().AddComponent<RectTransform>();
      return CreateViewport(
        resolution,
        camera,
        canvasRootRect,
        safeAreaMinimumAnchor,
        safeAreaMaximumAnchor,
        canvasPixelRect
      );
    }

    [TearDown]
    public void TearDown()
    {
      foreach (var createdObject in _createdObjects)
      {
        if (createdObject)
        {
          UnityEngine.Object.DestroyImmediate(createdObject);
        }
      }
    }

    public static void AssertVector3Equal(Vector3 expected, Vector3 actual, float tolerance = 0.01f)
    {
      Assert.That(actual.x, Is.EqualTo(expected.x).Within(tolerance), $"X component mismatch");
      Assert.That(actual.y, Is.EqualTo(expected.y).Within(tolerance), $"Y component mismatch");
      Assert.That(actual.z, Is.EqualTo(expected.z).Within(tolerance), $"Z component mismatch");
    }

    protected GameObject CreateGameObject()
    {
      var gameObject = new GameObject();
      _createdObjects.Add(gameObject);
      return gameObject;
    }

    protected T CreateSceneObject<T>(Action<T>? configure = null)
      where T : Displayable
    {
      var result = CreateGameObject().AddComponent<T>();
      configure?.Invoke(result);
      result.Initialize(Registry, _gameMode, _testConfiguration, fromRegistry: true);
      result.StartFromRegistry();
      return result;
    }

    protected T CreateSceneElement<T>(Action<T>? configure = null)
      where T : SceneElement
    {
      var result = CreateGameObject().AddComponent<T>();
      configure?.Invoke(result);
      result.Initialize(Registry, _gameMode, _testConfiguration);
      return result;
    }

    protected TestDisplayable CreateDisplayable()
    {
      return CreateSceneObject<TestDisplayable>();
    }

    protected FakeViewport CreateViewportForLandscapeLayout(
      GameViewResolution resolution,
      float? battleLayoutYRotation = null
    )
    {
      if (battleLayoutYRotation.HasValue)
      {
        LandscapeLayout.transform.rotation = Quaternion.Euler(0f, battleLayoutYRotation.Value, 0f);
      }

      var cameraPosition = LandscapeLayout.CameraPosition;
      var cameraGo = CreateGameObject();
      cameraGo.transform.position = cameraPosition.position;
      cameraGo.transform.rotation = cameraPosition.rotation;

      var canvasRootRect = CreateGameObject().AddComponent<RectTransform>();
      return CreateViewport(resolution, cameraGo.transform, canvasRootRect);
    }

    protected Card CreateTestCard()
    {
      var prefab = AssetDatabase.LoadAssetAtPath<Card>("Assets/Content/Prefabs/Card.prefab");
      var cardObject = UnityEngine.Object.Instantiate(prefab.gameObject);
      _createdObjects.Add(cardObject);
      var card = cardObject.GetComponent<Card>();
      card._cardView = new CardView
      {
        Id = $"test-card-{_testCardCounter++}",
        Position = new ObjectPosition
        {
          Position = new Position { Enum = PositionEnum.Offscreen },
          SortingKey = 0,
        },
        Prefab = CardPrefab.Character,
        CardFacing = CardFacing.FaceUp,
      };
      card.Initialize(Registry, _gameMode, _testConfiguration, fromRegistry: true);
      card.StartFromRegistry();
      return card;
    }

    protected Card[] CreateTestCards(int count)
    {
      var cards = new Card[count];
      for (var i = 0; i < count; i++)
      {
        cards[i] = CreateTestCard();
      }
      return cards;
    }

    protected static void AssertPointIsOnScreen(
      IGameViewport viewport,
      Vector3 worldPosition,
      string description
    )
    {
      var viewportPoint = viewport.WorldToViewportPoint(worldPosition);
      Assert.That(
        viewportPoint.x,
        Is.GreaterThanOrEqualTo(0f).And.LessThanOrEqualTo(1f),
        $"{description} is off screen horizontally (viewport x={viewportPoint.x})"
      );
      Assert.That(
        viewportPoint.y,
        Is.GreaterThanOrEqualTo(0f).And.LessThanOrEqualTo(1f),
        $"{description} is off screen vertically (viewport y={viewportPoint.y})"
      );
      Assert.That(
        viewportPoint.z,
        Is.GreaterThan(0f),
        $"{description} is behind the camera (viewport z={viewportPoint.z})"
      );
    }

    protected static void AssertCardIsOnScreen(IGameViewport viewport, Card card)
    {
      AssertPointIsOnScreen(viewport, card.GetCardCenter(), $"Card center ({card.name})");
      AssertPointIsOnScreen(
        viewport,
        card.GetCostSpriteWorldPosition(),
        $"Card cost sprite ({card.name})"
      );
    }

    protected static void AssertCardsAreOnScreen(IGameViewport viewport, Card[] cards)
    {
      foreach (var card in cards)
      {
        AssertCardIsOnScreen(viewport, card);
      }
    }

    protected static void AssertCardsAreHorizontallyOrdered(IGameViewport viewport, Card[] cards)
    {
      for (var i = 0; i < cards.Length - 1; i++)
      {
        var screenPos1 = viewport.WorldToScreenPoint(cards[i].GetCardCenter());
        var screenPos2 = viewport.WorldToScreenPoint(cards[i + 1].GetCardCenter());
        Assert.That(
          screenPos2.x,
          Is.GreaterThan(screenPos1.x),
          $"Card {i + 1} should be to the right of card {i} on screen"
        );
      }
    }

    protected static void AssertCardBoxColliderIsOnScreen(
      IGameViewport viewport,
      Card card,
      string cardDescription
    )
    {
      var collider = card.CardCollider;
      var corners = GetBoxColliderWorldCorners(collider);

      for (var i = 0; i < corners.Length; i++)
      {
        AssertPointIsOnScreen(viewport, corners[i], $"{cardDescription} box collider corner {i}");
      }
    }

    /// <summary>
    /// Computes the world-space corners of a BoxCollider without using
    /// BoxCollider.bounds.
    ///
    /// We cannot use BoxCollider.bounds in Edit mode tests because it returns
    /// a world-space axis-aligned bounding box (AABB) that is computed by
    /// Unity's physics engine during the physics simulation step. In Edit mode
    /// tests, physics simulation does not run reliably, causing
    /// non-deterministic behavior where the bounds may remain at the origin
    /// even after the transform has been updated.
    ///
    /// Instead, this method manually computes world corners by:
    /// 1. Using collider.center and collider.size (local-space properties
    ///    that don't depend on physics)
    /// 2. Computing the 8 local-space corners of the box
    /// 3. Using Transform.TransformPoint() to convert each corner to world
    ///    space
    ///
    /// Transform.TransformPoint() is deterministic and works immediately
    /// without waiting for physics updates.
    /// </summary>
    static Vector3[] GetBoxColliderWorldCorners(BoxCollider collider)
    {
      var transform = collider.transform;
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

      var worldCorners = new Vector3[8];
      for (var i = 0; i < 8; i++)
      {
        worldCorners[i] = transform.TransformPoint(localCorners[i]);
      }

      return worldCorners;
    }
  }
}
