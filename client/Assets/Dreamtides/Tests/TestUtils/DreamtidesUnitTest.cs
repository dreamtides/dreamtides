#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using Dreamtides.TestFakes;
using NUnit.Framework;
using UnityEngine;

namespace Dreamtides.Tests.TestUtils
{
  public class TestDisplayable : Displayable { }

  public class TestCard : Displayable
  {
    internal BoxCollider _cardCollider = null!;
    internal Transform _costSpritePosition = null!;

    public BoxCollider CardCollider => _cardCollider;
    public Transform CostSpritePosition => _costSpritePosition;

    public Vector3 GetCardCenter() => transform.TransformPoint(_cardCollider.center);

    public Vector3 GetCostSpriteWorldPosition() => _costSpritePosition.position;
  }

  public abstract class DreamtidesUnitTest
  {
    readonly List<GameObject> _createdObjects = new();

    Registry? _registry;
    BattleLayout? _portraitLayout;
    BattleLayout? _landscapeLayout;
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

    protected IEnumerator Initialize(IGameViewport? viewport = null)
    {
      var canvas = GeneratedCanvas.Create(_createdObjects);
      var mainCamera = GeneratedMainCamera.Create(_createdObjects);
      _portraitLayout = GeneratedPortraitBattleLayout.Create(_createdObjects, canvas);
      _landscapeLayout = GeneratedLandscapeBattleLayout.Create(_createdObjects, canvas);

      var generatedRegistry = GeneratedRegistry.Create(
        _createdObjects,
        canvas,
        mainCamera,
        _portraitLayout,
        _landscapeLayout
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

    protected TestCard CreateTestCard(
      float colliderWidth = 2.5f,
      float colliderHeight = 4f,
      Vector3? costSpriteLocalPosition = null
    )
    {
      var card = CreateSceneObject<TestCard>(c =>
      {
        var collider = c.gameObject.AddComponent<BoxCollider>();
        collider.size = new Vector3(colliderWidth, colliderHeight, 0.1f);
        collider.center = new Vector3(0f, -0.5f, 0f);
        c._cardCollider = collider;

        var costSpriteGo = new GameObject("CostSprite");
        _createdObjects.Add(costSpriteGo);
        costSpriteGo.transform.SetParent(c.transform, worldPositionStays: false);
        costSpriteGo.transform.localPosition =
          costSpriteLocalPosition ?? new Vector3(-1f, 1.5f, 0f);
        c._costSpritePosition = costSpriteGo.transform;
      });
      return card;
    }

    protected TestCard[] CreateTestCards(
      int count,
      float colliderWidth = 2.5f,
      float colliderHeight = 4f
    )
    {
      var cards = new TestCard[count];
      for (var i = 0; i < count; i++)
      {
        cards[i] = CreateTestCard(colliderWidth, colliderHeight);
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

    protected static void AssertCardIsOnScreen(IGameViewport viewport, TestCard card)
    {
      AssertPointIsOnScreen(viewport, card.GetCardCenter(), $"Card center ({card.name})");
      AssertPointIsOnScreen(
        viewport,
        card.GetCostSpriteWorldPosition(),
        $"Card cost sprite ({card.name})"
      );
    }

    protected static void AssertCardsAreOnScreen(IGameViewport viewport, TestCard[] cards)
    {
      foreach (var card in cards)
      {
        AssertCardIsOnScreen(viewport, card);
      }
    }

    protected static void AssertCardsAreHorizontallyOrdered(
      IGameViewport viewport,
      TestCard[] cards
    )
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
  }
}
