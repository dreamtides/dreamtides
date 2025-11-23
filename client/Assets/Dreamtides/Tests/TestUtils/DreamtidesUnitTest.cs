#nullable enable

using System;
using System.Collections;
using System.Collections.Generic;
using Dreamtides.Components;
using Dreamtides.Layout;
using Dreamtides.Services;
using NUnit.Framework;
using UnityEngine;

namespace Dreamtides.Tests.TestUtils
{
  public class TestDisplayable : Displayable { }

  public abstract class DreamtidesUnitTest
  {
    readonly List<GameObject> _createdObjects = new();

    Registry? _registry;
    GameLayout? _portraitLayout;
    GameLayout? _landscapeLayout;
    GameMode _gameMode = GameMode.Quest;
    TestConfiguration? _testConfiguration;

    protected Registry Registry =>
      _registry ?? throw new InvalidOperationException("Registry not initialized");

    protected TestConfiguration TestConfiguration =>
      _testConfiguration
      ?? throw new InvalidOperationException("Test configuration not initialized");

    protected IEnumerator Initialize(IGameViewport? viewport = null)
    {
      _registry = CreateGameObject().AddComponent<Registry>();
      _portraitLayout = CreateGameObject().AddComponent<GameLayout>();
      _landscapeLayout = CreateGameObject().AddComponent<GameLayout>();
      _registry._portraitLayout = _portraitLayout;
      _registry._landscapeLayout = _landscapeLayout;
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
  }
}
